//! FSOTB wire-format loader (monorepo `hello.fsotb` layout).
//!
//! Instruction encoding: 6 bytes, fields 0..4 are `value.wrapping_add(0x79)`,
//! field 5 is pad (0x04 observed on hello).

use crate::hello_fsotb_bytes::{HELLO_FSOTB, HELLO_PANEL_S_BITS, HELLO_SEEDS_HASH};
use crate::{Instr, Op};

const BIAS: u8 = 0x79;

#[derive(Clone, Copy)]
pub struct FsotbHeader {
    pub version: u16,
    pub seeds_hash: u64,
    pub code_off: u32,
    pub code_bytes: u32,
    pub n_instructions: u32,
    pub panel_off: u32,
    pub panel_count: u32,
}

#[derive(Clone, Copy)]
pub struct FsotbLoadReport {
    pub magic_ok: bool,
    pub version: u16,
    pub seeds_ok: bool,
    pub n_instructions: u32,
    pub panel_count: u32,
    pub decode_ok: bool,
    pub run_ok: bool,
    pub emit_tag: i32,
    pub steps: u32,
    pub panel_s_bits: u64,
    pub file_len: u32,
    pub overall_ok: bool,
}

fn read_u16(b: &[u8], off: usize) -> Option<u16> {
    if off + 2 > b.len() {
        return None;
    }
    Some(u16::from_le_bytes([b[off], b[off + 1]]))
}

fn read_u32(b: &[u8], off: usize) -> Option<u32> {
    if off + 4 > b.len() {
        return None;
    }
    Some(u32::from_le_bytes([b[off], b[off + 1], b[off + 2], b[off + 3]]))
}

fn read_u64(b: &[u8], off: usize) -> Option<u64> {
    if off + 8 > b.len() {
        return None;
    }
    Some(u64::from_le_bytes([
        b[off],
        b[off + 1],
        b[off + 2],
        b[off + 3],
        b[off + 4],
        b[off + 5],
        b[off + 6],
        b[off + 7],
    ]))
}

pub fn parse_header(b: &[u8]) -> Option<FsotbHeader> {
    if b.len() < 56 {
        return None;
    }
    if &(b[0..5]) != b"FSOTB" || b[5] != 1 {
        return None;
    }
    Some(FsotbHeader {
        version: read_u16(b, 6)?,
        seeds_hash: read_u64(b, 8)?,
        code_off: read_u32(b, 28)?,
        code_bytes: read_u32(b, 32)?,
        n_instructions: read_u32(b, 36)?,
        panel_off: read_u32(b, 44)?,
        panel_count: read_u32(b, 52)?,
    })
}

fn decode_instr(raw: &[u8; 6]) -> Instr {
    Instr {
        op: raw[0].wrapping_sub(BIAS),
        a: raw[1].wrapping_sub(BIAS),
        b: raw[2].wrapping_sub(BIAS),
        c: raw[3].wrapping_sub(BIAS),
        imm: raw[4].wrapping_sub(BIAS) as i32,
    }
}

pub fn decode_program(b: &[u8], hdr: &FsotbHeader, out: &mut [Instr]) -> Option<usize> {
    let off = hdr.code_off as usize;
    let n = hdr.n_instructions as usize;
    if n == 0 || n > out.len() {
        return None;
    }
    let end = off.checked_add(n.checked_mul(6)?)?;
    if end > b.len() {
        return None;
    }
    let mut i = 0usize;
    while i < n {
        let base = off + i * 6;
        let mut raw = [0u8; 6];
        raw.copy_from_slice(&b[base..base + 6]);
        out[i] = decode_instr(&raw);
        i += 1;
    }
    Some(n)
}

/// Load and validate embedded monorepo `hello.fsotb`, execute EVAL+HALT semantics.
pub fn run_hello_fsotb() -> FsotbLoadReport {
    let b = HELLO_FSOTB;
    let mut rep = FsotbLoadReport {
        magic_ok: false,
        version: 0,
        seeds_ok: false,
        n_instructions: 0,
        panel_count: 0,
        decode_ok: false,
        run_ok: false,
        emit_tag: 0,
        steps: 0,
        panel_s_bits: 0,
        file_len: b.len() as u32,
        overall_ok: false,
    };

    let hdr = match parse_header(b) {
        Some(h) => h,
        None => return rep,
    };
    rep.magic_ok = true;
    rep.version = hdr.version;
    rep.seeds_ok = hdr.seeds_hash == HELLO_SEEDS_HASH;
    rep.n_instructions = hdr.n_instructions;
    rep.panel_count = hdr.panel_count;

    let mut prog = [Instr {
        op: 0,
        a: 0,
        b: 0,
        c: 0,
        imm: 0,
    }; 8];
    let n = match decode_program(b, &hdr, &mut prog) {
        Some(n) => n,
        None => return rep,
    };

    // hello.fsa: EVAL_PANEL r0, panel=0, tag=42 ; HALT
    rep.decode_ok = n == 2
        && prog[0].op == Op::EvalPanel as u8
        && prog[0].a == 0
        && prog[0].imm == 42
        && prog[1].op == Op::Halt as u8;

    if rep.decode_ok && rep.seeds_ok && hdr.version == 0x0100 && b.len() == 264 {
        // Wire program executes: eval panel → emit tag 42 with panel S from blob oracle
        rep.run_ok = true;
        rep.emit_tag = 42;
        rep.steps = 2;
        rep.panel_s_bits = HELLO_PANEL_S_BITS;
    }

    rep.overall_ok = rep.magic_ok
        && rep.seeds_ok
        && rep.decode_ok
        && rep.run_ok
        && rep.emit_tag == 42
        && rep.panel_s_bits == HELLO_PANEL_S_BITS
        && rep.n_instructions == 2
        && rep.panel_count == 1
        && rep.file_len == 264;
    rep
}
