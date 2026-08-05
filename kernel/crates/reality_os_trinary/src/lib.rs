//! FSOTB / Metatron trinary ISA interpreter for Reality OS.
//!
//! Opcode registry mirrors monorepo `vendor/trinary_os/isa/fsotb_opcode_registry.json`
//! (27 ops, 25 regs = D_eff ceiling, 6-byte instructions conceptually).
//!
//! v0.3 implements a **kernel-resident** interpreter for the core instruction set
//! used at boot self-test: IMM, MOVT, ADDT, SUBT, NEGT, COLLAPSE, EVAL_PANEL,
//! EMIT, HALT, plus CALL/RET stack and SPAWN/JOIN task markers.

#![no_std]

mod hello_fsotb_bytes;
pub mod fsotb;

pub use fsotb::{run_hello_fsotb, FsotbLoadReport};

use reality_os_scalar::{compute_s, residual_predict, sign_trit, DOMAIN_TABLE};

/// Word width in trits (ABI).
pub const WORD_WIDTH_TRITS: u32 = 27;
/// Register file size (= D_eff ceiling).
pub const REGISTER_COUNT: usize = 25;
/// Cooperative task slots (ABI num_task_slots).
pub const NUM_TASK_SLOTS: usize = 8;
/// Max instructions in a boot program blob.
pub const MAX_PROG_LEN: usize = 64;
/// Call/value stack depth.
pub const STACK_DEPTH: usize = 32;

/// Opcode numbers (fsotb_opcode_registry.json).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    Halt = 0,
    Imm = 1,
    Movt = 2,
    Loadt = 3,
    Storet = 4,
    Addt = 5,
    Subt = 6,
    Mult = 7,
    Negt = 8,
    Min3 = 9,
    Max3 = 10,
    Collapse = 11,
    EvalPanel = 12,
    Consensus = 13,
    PhaseRot = 14,
    Brancht = 15,
    Emit = 16,
    LoadRule = 17,
    ApplyOvr = 18,
    Measure = 19,
    Call = 20,
    Ret = 21,
    PushT = 22,
    PopT = 23,
    Syscall = 24,
    Spawn = 25,
    Join = 26,
}

impl Op {
    pub fn from_u8(v: u8) -> Option<Op> {
        if v > 26 {
            return None;
        }
        Some(unsafe { core::mem::transmute(v) })
    }

    pub fn mnemonic(self) -> &'static str {
        match self {
            Op::Halt => "HALT",
            Op::Imm => "IMM",
            Op::Movt => "MOVT",
            Op::Loadt => "LOADT",
            Op::Storet => "STORET",
            Op::Addt => "ADDT",
            Op::Subt => "SUBT",
            Op::Mult => "MULT",
            Op::Negt => "NEGT",
            Op::Min3 => "MIN3",
            Op::Max3 => "MAX3",
            Op::Collapse => "COLLAPSE",
            Op::EvalPanel => "EVAL_PANEL",
            Op::Consensus => "CONSENSUS",
            Op::PhaseRot => "PHASE_ROT",
            Op::Brancht => "BRANCHT",
            Op::Emit => "EMIT",
            Op::LoadRule => "LOAD_RULE",
            Op::ApplyOvr => "APPLY_OVR",
            Op::Measure => "MEASURE",
            Op::Call => "CALL",
            Op::Ret => "RET",
            Op::PushT => "PUSH_T",
            Op::PopT => "POP_T",
            Op::Syscall => "SYSCALL",
            Op::Spawn => "SPAWN",
            Op::Join => "JOIN",
        }
    }
}

/// Compact instruction: op + 3 register/imm fields (kernel encoding, not wire FSOTB).
#[derive(Clone, Copy)]
pub struct Instr {
    pub op: u8,
    pub a: u8,
    pub b: u8,
    pub c: u8,
    /// Optional signed immediate / panel index / tag.
    pub imm: i32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum VmStatus {
    Ok,
    Halted,
    Error,
}

pub struct Vm {
    pub regs: [i32; REGISTER_COUNT],
    pub pc: usize,
    pub status: VmStatus,
    pub steps: u32,
    pub last_emit_tag: i32,
    pub last_emit_s_bits: u64,
    pub stack: [i32; STACK_DEPTH],
    pub sp: usize,
    pub call_stack: [usize; STACK_DEPTH],
    pub csp: usize,
    pub spawn_count: u32,
    pub join_count: u32,
    pub eval_count: u32,
}

impl Vm {
    pub const fn new() -> Self {
        Self {
            regs: [0; REGISTER_COUNT],
            pc: 0,
            status: VmStatus::Ok,
            steps: 0,
            last_emit_tag: 0,
            last_emit_s_bits: 0,
            stack: [0; STACK_DEPTH],
            sp: 0,
            call_stack: [0; STACK_DEPTH],
            csp: 0,
            spawn_count: 0,
            join_count: 0,
            eval_count: 0,
        }
    }

    fn reg(&self, i: u8) -> i32 {
        let i = (i as usize) % REGISTER_COUNT;
        self.regs[i]
    }

    fn set_reg(&mut self, i: u8, v: i32) {
        let i = (i as usize) % REGISTER_COUNT;
        self.regs[i] = v;
    }

    fn push(&mut self, v: i32) -> bool {
        if self.sp >= STACK_DEPTH {
            return false;
        }
        self.stack[self.sp] = v;
        self.sp += 1;
        true
    }

    fn pop(&mut self) -> Option<i32> {
        if self.sp == 0 {
            return None;
        }
        self.sp -= 1;
        Some(self.stack[self.sp])
    }

    /// Execute one instruction.
    pub fn step(&mut self, program: &[Instr]) -> VmStatus {
        if self.status != VmStatus::Ok {
            return self.status;
        }
        if self.pc >= program.len() {
            self.status = VmStatus::Halted;
            return self.status;
        }
        let ins = program[self.pc];
        self.pc += 1;
        self.steps += 1;

        let op = match Op::from_u8(ins.op) {
            Some(o) => o,
            None => {
                self.status = VmStatus::Error;
                return self.status;
            }
        };

        match op {
            Op::Halt => {
                self.status = VmStatus::Halted;
            }
            Op::Imm => {
                self.set_reg(ins.a, ins.imm);
            }
            Op::Movt => {
                self.set_reg(ins.a, self.reg(ins.b));
            }
            Op::Loadt | Op::Storet => {
                // Memory ops deferred to frame-backed heap; treat as mov for now.
                self.set_reg(ins.a, self.reg(ins.b));
            }
            Op::Addt => {
                self.set_reg(ins.a, self.reg(ins.b).wrapping_add(self.reg(ins.c)));
            }
            Op::Subt => {
                self.set_reg(ins.a, self.reg(ins.b).wrapping_sub(self.reg(ins.c)));
            }
            Op::Mult => {
                self.set_reg(ins.a, self.reg(ins.b).wrapping_mul(self.reg(ins.c)));
            }
            Op::Negt => {
                self.set_reg(ins.a, self.reg(ins.b).wrapping_neg());
            }
            Op::Min3 => {
                let x = self.reg(ins.b);
                let y = self.reg(ins.c);
                self.set_reg(ins.a, if x < y { x } else { y });
            }
            Op::Max3 => {
                let x = self.reg(ins.b);
                let y = self.reg(ins.c);
                self.set_reg(ins.a, if x > y { x } else { y });
            }
            Op::Collapse => {
                // Collapse f64-ish reg value: use sign_trit on reg as i32 scaled
                let v = self.reg(ins.b) as f64;
                self.set_reg(ins.a, sign_trit(v) as i32);
            }
            Op::EvalPanel => {
                // Wire: rd=a, panel=b, tag=imm (hello.fsotb). Domain table panel index = b.
                let idx = (ins.b as usize) % DOMAIN_TABLE.len().max(1);
                let d = &DOMAIN_TABLE[idx];
                let s = compute_s(d.d_eff, d.delta_psi, d.observed, d.hits);
                let scaled = (s * 1_000_000.0) as i32;
                self.set_reg(ins.a, scaled);
                self.last_emit_s_bits = s.to_bits();
                self.last_emit_tag = ins.imm;
                self.eval_count += 1;
            }
            Op::Consensus => {
                let x = self.reg(ins.b);
                let y = self.reg(ins.c);
                // majority-of-two with zero bias
                self.set_reg(ins.a, if x == y { x } else { 0 });
            }
            Op::PhaseRot => {
                // rotate trit: -1->0, 0->1, 1->-1
                let v = self.reg(ins.b);
                let r = if v < 0 {
                    0
                } else if v == 0 {
                    1
                } else {
                    -1
                };
                self.set_reg(ins.a, r);
            }
            Op::Brancht => {
                // if reg a != 0, pc += imm (relative)
                if self.reg(ins.a) != 0 {
                    let delta = ins.imm;
                    if delta >= 0 {
                        self.pc = self.pc.wrapping_add(delta as usize);
                    } else {
                        self.pc = self.pc.wrapping_sub((-delta) as usize);
                    }
                }
            }
            Op::Emit => {
                self.last_emit_tag = ins.imm;
                // pair with last eval S bits already stored
            }
            Op::LoadRule | Op::ApplyOvr | Op::Measure | Op::Syscall => {
                // reserved — no-op success for self-test
            }
            Op::Call => {
                if self.csp >= STACK_DEPTH {
                    self.status = VmStatus::Error;
                } else {
                    self.call_stack[self.csp] = self.pc;
                    self.csp += 1;
                    self.pc = ins.imm as usize;
                }
            }
            Op::Ret => {
                if self.csp == 0 {
                    self.status = VmStatus::Halted;
                } else {
                    self.csp -= 1;
                    self.pc = self.call_stack[self.csp];
                }
            }
            Op::PushT => {
                if !self.push(self.reg(ins.a)) {
                    self.status = VmStatus::Error;
                }
            }
            Op::PopT => match self.pop() {
                Some(v) => self.set_reg(ins.a, v),
                None => self.status = VmStatus::Error,
            },
            Op::Spawn => {
                self.spawn_count += 1;
            }
            Op::Join => {
                self.join_count += 1;
            }
        }
        self.status
    }

    pub fn run(&mut self, program: &[Instr], max_steps: u32) -> VmStatus {
        let mut n = 0u32;
        while self.status == VmStatus::Ok && n < max_steps {
            self.step(program);
            n += 1;
        }
        self.status
    }
}

/// Boot self-test program:
/// IMM r1, 3; IMM r2, 4; ADDT r0,r1,r2; EVAL_PANEL r3, domain=0; EMIT tag=42; HALT
pub fn boot_selftest_program() -> [Instr; 6] {
    [
        Instr {
            op: Op::Imm as u8,
            a: 1,
            b: 0,
            c: 0,
            imm: 3,
        },
        Instr {
            op: Op::Imm as u8,
            a: 2,
            b: 0,
            c: 0,
            imm: 4,
        },
        Instr {
            op: Op::Addt as u8,
            a: 0,
            b: 1,
            c: 2,
            imm: 0,
        },
        Instr {
            op: Op::EvalPanel as u8,
            a: 3,
            b: 0,
            c: 0,
            imm: 0,
        },
        Instr {
            op: Op::Emit as u8,
            a: 0,
            b: 0,
            c: 0,
            imm: 42,
        },
        Instr {
            op: Op::Halt as u8,
            a: 0,
            b: 0,
            c: 0,
            imm: 0,
        },
    ]
}

/// Run boot self-test; returns (ok, steps, r0, emit_tag, eval_count).
pub fn run_boot_selftest() -> (bool, u32, i32, i32, u32) {
    let prog = boot_selftest_program();
    let mut vm = Vm::new();
    let st = vm.run(&prog, 32);
    let ok = st == VmStatus::Halted && vm.regs[0] == 7 && vm.last_emit_tag == 42 && vm.eval_count >= 1;
    (ok, vm.steps, vm.regs[0], vm.last_emit_tag, vm.eval_count)
}

/// Opcode registry completeness check (0..=26 all present).
pub fn opcode_registry_ok() -> bool {
    let mut i = 0u8;
    while i <= 26 {
        if Op::from_u8(i).is_none() {
            return false;
        }
        i += 1;
    }
    REGISTER_COUNT == 25 && WORD_WIDTH_TRITS == 27 && NUM_TASK_SLOTS == 8
}

/// Residual micro-demo: residual_predict on domain 0.
pub fn residual_demo_ok() -> bool {
    if DOMAIN_TABLE.is_empty() {
        return false;
    }
    let d = &DOMAIN_TABLE[0];
    let s = compute_s(d.d_eff, d.delta_psi, d.observed, d.hits);
    let c = residual_predict(1.0, s, d.factor);
    c.is_finite() && c > 0.0
}
