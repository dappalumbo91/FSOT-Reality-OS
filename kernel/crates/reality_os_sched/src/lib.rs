//! Cooperative domain scheduler for Reality OS.
//!
//! Each task is a domain interface quantum: compute S once per turn.
//! Round-robin over a ready queue sized for boot demos (not full 530 PCB yet —
//! queue holds indices into DOMAIN_TABLE).

#![no_std]

use reality_os_scalar::{compute_s, residual_predict, sign_trit, DOMAIN_TABLE};

pub const MAX_TASKS: usize = 64;

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
    pub domain_index: u32,
    pub ticks: u32,
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
            last_s_bits: 0,
            last_trit: 0,
            last_c_bits: 0,
        }
    }
}

pub struct Scheduler {
    pub tasks: [Task; MAX_TASKS],
    pub task_count: usize,
    pub current: usize,
    pub switches: u32,
    pub quanta_run: u32,
}

impl Scheduler {
    pub const fn new() -> Self {
        Self {
            tasks: [Task::free(); MAX_TASKS],
            task_count: 0,
            current: 0,
            switches: 0,
            quanta_run: 0,
        }
    }

    /// Enqueue first `n` domains (capped at MAX_TASKS).
    pub fn seed_from_domain_table(&mut self, n: usize) {
        let n = if n > MAX_TASKS { MAX_TASKS } else { n };
        let n = if n > DOMAIN_TABLE.len() {
            DOMAIN_TABLE.len()
        } else {
            n
        };
        let mut i = 0usize;
        while i < n {
            self.tasks[i] = Task {
                state: TaskState::Ready,
                domain_index: i as u32,
                ticks: 0,
                last_s_bits: 0,
                last_trit: 0,
                last_c_bits: 0,
            };
            i += 1;
        }
        self.task_count = n;
        self.current = 0;
    }

    /// Run one cooperative quantum on the current ready task, then advance.
    pub fn run_one_quantum(&mut self) -> bool {
        if self.task_count == 0 {
            return false;
        }
        // find next ready starting at current
        let mut tries = 0usize;
        while tries < self.task_count {
            let idx = (self.current + tries) % self.task_count;
            if self.tasks[idx].state == TaskState::Ready
                || self.tasks[idx].state == TaskState::Running
            {
                self.current = idx;
                break;
            }
            tries += 1;
        }
        if tries >= self.task_count {
            return false;
        }

        let t = &mut self.tasks[self.current];
        t.state = TaskState::Running;
        let di = t.domain_index as usize;
        if di >= DOMAIN_TABLE.len() {
            t.state = TaskState::Done;
            return false;
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
        self.switches += 1;
        self.current = (self.current + 1) % self.task_count;
        true
    }

    /// Run `n` quanta.
    pub fn run_quanta(&mut self, n: u32) -> u32 {
        let mut i = 0u32;
        let mut ran = 0u32;
        while i < n {
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

/// Boot self-test: schedule 32 domains for 64 quanta.
pub fn boot_sched_selftest() -> (bool, u32, u32, u32) {
    let mut sched = Scheduler::new();
    sched.seed_from_domain_table(32);
    let ran = sched.run_quanta(64);
    let ok = sched.task_count == 32 && ran == 64 && sched.switches == 64;
    (ok, sched.task_count as u32, ran, sched.switches)
}
