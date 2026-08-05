//! Physical frame allocator from bootloader memory map.
//!
//! Tracks usable frames reported by BIOS/UEFI via `BootInfo.memory_map`.
//! Allocates 4 KiB frame numbers; does not require `map_physical_memory`
//! for bookkeeping (frame indices only). Optional writeback when offset known.

#![no_std]

use bootloader::bootinfo::{MemoryMap, MemoryRegionType};

pub const FRAME_SIZE: u64 = 4096;

/// Snapshot of memory map stats at boot.
#[derive(Clone, Copy)]
pub struct MemMapReport {
    pub regions: u32,
    pub usable_regions: u32,
    pub usable_frames: u64,
    pub total_frames_seen: u64,
    pub allocated: u64,
    pub free_remaining: u64,
}

/// Simple bump allocator over usable regions (first-fit linear).
pub struct FrameAllocator {
    /// Up to 64 usable runs: (start_frame, end_frame exclusive)
    runs: [(u64, u64); 64],
    run_count: usize,
    /// Current run index + next frame to hand out
    cur_run: usize,
    next_frame: u64,
    allocated: u64,
    usable_frames: u64,
    regions: u32,
    usable_regions: u32,
    total_frames_seen: u64,
}

impl FrameAllocator {
    pub const fn empty() -> Self {
        Self {
            runs: [(0, 0); 64],
            run_count: 0,
            cur_run: 0,
            next_frame: 0,
            allocated: 0,
            usable_frames: 0,
            regions: 0,
            usable_regions: 0,
            total_frames_seen: 0,
        }
    }

    /// Build from bootloader memory map.
    pub fn from_memory_map(map: &MemoryMap) -> Self {
        let mut a = Self::empty();
        for region in map.iter() {
            a.regions += 1;
            let start = region.range.start_frame_number;
            let end = region.range.end_frame_number;
            if end > start {
                a.total_frames_seen = a.total_frames_seen.saturating_add(end - start);
            }
            if region.region_type == MemoryRegionType::Usable && end > start {
                if a.run_count < 64 {
                    a.runs[a.run_count] = (start, end);
                    a.run_count += 1;
                }
                a.usable_regions += 1;
                a.usable_frames = a.usable_frames.saturating_add(end - start);
            }
        }
        if a.run_count > 0 {
            a.cur_run = 0;
            a.next_frame = a.runs[0].0;
        }
        a
    }

    /// Allocate one 4 KiB frame; returns physical frame number, or None if exhausted.
    pub fn allocate_frame(&mut self) -> Option<u64> {
        while self.cur_run < self.run_count {
            let (start, end) = self.runs[self.cur_run];
            if self.next_frame < start {
                self.next_frame = start;
            }
            if self.next_frame < end {
                let f = self.next_frame;
                self.next_frame += 1;
                self.allocated += 1;
                return Some(f);
            }
            self.cur_run += 1;
            if self.cur_run < self.run_count {
                self.next_frame = self.runs[self.cur_run].0;
            }
        }
        None
    }

    /// Allocate `n` frames; returns start frame of contiguous run if possible within one region.
    pub fn allocate_frames(&mut self, n: u64) -> Option<u64> {
        if n == 0 {
            return Some(0);
        }
        // try current run
        while self.cur_run < self.run_count {
            let (start, end) = self.runs[self.cur_run];
            if self.next_frame < start {
                self.next_frame = start;
            }
            if self.next_frame + n <= end {
                let f = self.next_frame;
                self.next_frame += n;
                self.allocated += n;
                return Some(f);
            }
            self.cur_run += 1;
            if self.cur_run < self.run_count {
                self.next_frame = self.runs[self.cur_run].0;
            }
        }
        None
    }

    pub fn report(&self) -> MemMapReport {
        MemMapReport {
            regions: self.regions,
            usable_regions: self.usable_regions,
            usable_frames: self.usable_frames,
            total_frames_seen: self.total_frames_seen,
            allocated: self.allocated,
            free_remaining: self.usable_frames.saturating_sub(self.allocated),
        }
    }
}

/// Boot self-test: inventory map + allocate a handful of frames.
pub fn boot_mem_selftest(map: &MemoryMap) -> (bool, MemMapReport, u64) {
    let mut alloc = FrameAllocator::from_memory_map(map);
    let mut first = 0u64;
    let mut got = 0u64;
    // allocate 16 frames
    let mut i = 0u64;
    while i < 16 {
        if let Some(f) = alloc.allocate_frame() {
            if got == 0 {
                first = f;
            }
            got += 1;
        }
        i += 1;
    }
    // allocate a 4-frame block
    let block = alloc.allocate_frames(4);
    let rep = alloc.report();
    let ok = rep.usable_frames > 100
        && rep.usable_regions > 0
        && got == 16
        && block.is_some()
        && rep.allocated >= 20;
    (ok, rep, first)
}
