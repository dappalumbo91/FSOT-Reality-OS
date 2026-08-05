//! Domain scheduler: ready-queue over **all** DOMAIN_TABLE entries + tick preemption.
//!
//! - Ready queue holds domain indices (u16), capacity ≥ 530.
//! - Each quantum computes S + residual for one domain.
//! - Preemption: after `quantum_ticks` timer ticks (PIT), force switch.

#![no_std]

use reality_os_scalar::{compute_s, residual_predict, sign_trit, DOMAIN_COUNT, DOMAIN_TABLE};

/// Must cover full domain table (530).
pub const MAX_TASKS: usize = 640;
pub const READY_CAP: usize = 640;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Free,
    Ready,
    Running,
    Done,
}

#[derive(Clone, Copy)]
pub struct Task {
    pub state: TaskState,
    pub domain_index: u16,
    pub ticks: u32,
    pub preempts: u32,
    pub last_s_bits: u64,
    pub last_trit: i8,
    pub last_c_bits: u64,
}

impl Task {
    pub const fn free() -> Self {
        Self {
            state: TaskState::Free,
            domain_index: 0,
            ticks: 0,
            preempts: 0,
            last_s_bits: 0,
            last_trit: 0,
            last_c_bits: 0,
        }
    }
}

pub struct Scheduler {
    pub tasks: [Task; MAX_TASKS],
    pub task_count: usize,
    /// Circular ready queue of task slot indices.
    ready: [u16; READY_CAP],
    ready_head: usize,
    ready_tail: usize,
    ready_len: usize,
    pub current: Option<u16>,
    pub switches: u32,
    pub quanta_run: u32,
    pub preempt_count: u32,
    /// Ticks allowed per quantum before forced preemption.
    pub quantum_ticks: u32,
    ticks_in_quantum: u32,
}

impl Scheduler {
    pub const fn new() -> Self {
        Self {
            tasks: [Task::free(); MAX_TASKS],
            task_count: 0,
            ready: [0; READY_CAP],
            ready_head: 0,
            ready_tail: 0,
            ready_len: 0,
            current: None,
            switches: 0,
            quanta_run: 0,
            preempt_count: 0,
            quantum_ticks: 1,
            ticks_in_quantum: 0,
        }
    }

    fn enqueue(&mut self, task_slot: u16) -> bool {
        if self.ready_len >= READY_CAP {
            return false;
        }
        self.ready[self.ready_tail] = task_slot;
        self.ready_tail = (self.ready_tail + 1) % READY_CAP;
        self.ready_len += 1;
        true
    }

    fn dequeue(&mut self) -> Option<u16> {
        if self.ready_len == 0 {
            return None;
        }
        let t = self.ready[self.ready_head];
        self.ready_head = (self.ready_head + 1) % READY_CAP;
        self.ready_len -= 1;
        Some(t)
    }

    /// Enqueue **every** domain in DOMAIN_TABLE as a ready task.
    pub fn seed_all_domains(&mut self) -> usize {
        let n = DOMAIN_TABLE.len().min(MAX_TASKS).min(DOMAIN_COUNT);
        let mut i = 0usize;
        while i < n {
            self.tasks[i] = Task {
                state: TaskState::Ready,
                domain_index: i as u16,
                ticks: 0,
                preempts: 0,
                last_s_bits: 0,
                last_trit: 0,
                last_c_bits: 0,
            };
            let _ = self.enqueue(i as u16);
            i += 1;
        }
        self.task_count = n;
        self.current = None;
        self.ticks_in_quantum = 0;
        n
    }

    fn run_slot(&mut self, slot: u16) {
        let t = &mut self.tasks[slot as usize];
        t.state = TaskState::Running;
        let di = t.domain_index as usize;
        if di >= DOMAIN_TABLE.len() {
            t.state = TaskState::Done;
            return;
        }
        let d = &DOMAIN_TABLE[di];
        let s = compute_s(d.d_eff, d.delta_psi, d.observed, d.hits);
        let c = residual_predict(1.0, s, d.factor);
        t.last_s_bits = s.to_bits();
        t.last_c_bits = c.to_bits();
        t.last_trit = sign_trit(s);
        t.ticks += 1;
        t.state = TaskState::Ready;
        self.quanta_run += 1;
    }

    /// One cooperative quantum: dequeue → run → requeue.
    pub fn run_one_quantum(&mut self) -> bool {
        let slot = match self.dequeue() {
            Some(s) => s,
            None => {
                // refill from ready tasks if queue empty but tasks remain Ready
                let mut i = 0usize;
                let mut any = false;
                while i < self.task_count {
                    if self.tasks[i].state == TaskState::Ready {
                        let _ = self.enqueue(i as u16);
                        any = true;
                    }
                    i += 1;
                }
                if !any {
                    return false;
                }
                match self.dequeue() {
                    Some(s) => s,
                    None => return false,
                }
            }
        };
        self.current = Some(slot);
        self.run_slot(slot);
        let _ = self.enqueue(slot);
        self.switches += 1;
        self.ticks_in_quantum = 0;
        true
    }

    /// Timer tick: if quantum budget exhausted, count preemption and switch.
    pub fn on_timer_tick(&mut self) {
        self.ticks_in_quantum = self.ticks_in_quantum.saturating_add(1);
        if self.ticks_in_quantum >= self.quantum_ticks {
            self.preempt_count = self.preempt_count.saturating_add(1);
            if let Some(slot) = self.current {
                if (slot as usize) < self.task_count {
                    self.tasks[slot as usize].preempts =
                        self.tasks[slot as usize].preempts.saturating_add(1);
                }
            }
            // force quantum boundary
            let _ = self.run_one_quantum();
            self.ticks_in_quantum = 0;
        }
    }

    /// Run until `n` quanta or queue drains; inject `timer_every` fake ticks.
    pub fn run_with_preemption(&mut self, n: u32, timer_every: u32) -> u32 {
        let mut ran = 0u32;
        let mut i = 0u32;
        while i < n {
            if timer_every > 0 && (i % timer_every == 0) {
                // multiple ticks to force preempt path
                self.on_timer_tick();
            }
            if self.run_one_quantum() {
                ran += 1;
            } else {
                break;
            }
            i += 1;
        }
        ran
    }
}

/// Boot self-test: all domains enqueued; run ≥ DOMAIN_COUNT quanta with preemption.
pub fn boot_sched_selftest() -> (bool, u32, u32, u32, u32) {
    let mut sched = Scheduler::new();
    sched.quantum_ticks = 1;
    let n = sched.seed_all_domains() as u32;
    // at least one full sweep + extra for preemption
    let target = n.saturating_mul(2).max(64);
    let ran = sched.run_with_preemption(target, 1);
    let ok = n as usize == DOMAIN_COUNT.min(MAX_TASKS)
        && n >= 500
        && ran >= n
        && sched.preempt_count > 0
        && sched.switches >= n;
    (ok, n, ran, sched.switches, sched.preempt_count)
}
