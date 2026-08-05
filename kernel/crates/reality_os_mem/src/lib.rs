//! Physical frame allocator + heap on mapped physical memory.
//!
//! Requires bootloader feature `map_physical_memory` so
//! `BootInfo.physical_memory_offset` maps phys frames:
//! `virt = phys + offset`.

#![no_std]

use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use core::ptr;
use core::sync::atomic::{AtomicU64, Ordering};

pub const FRAME_SIZE: u64 = 4096;
const FRAME_MARKER: u64 = 0xF507_FEED_CAFE_BABE;

/// Snapshot of memory map + heap stats at boot.
#[derive(Clone, Copy)]
pub struct MemMapReport {
    pub regions: u32,
    pub usable_regions: u32,
    pub usable_frames: u64,
    pub total_frames_seen: u64,
    pub allocated: u64,
    pub free_remaining: u64,
    pub phys_offset: u64,
    pub heap_base_phys: u64,
    pub heap_bytes: u64,
    pub heap_used: u64,
    pub heap_write_ok: bool,
}

/// Simple bump allocator over usable regions (first-fit linear).
pub struct FrameAllocator {
    runs: [(u64, u64); 64],
    run_count: usize,
    cur_run: usize,
    next_frame: u64,
    allocated: u64,
    usable_frames: u64,
    regions: u32,
    usable_regions: u32,
    total_frames_seen: u64,
    phys_offset: u64,
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
            phys_offset: 0,
        }
    }

    pub fn from_memory_map(map: &MemoryMap, phys_offset: u64) -> Self {
        let mut a = Self::empty();
        a.phys_offset = phys_offset;
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

    pub fn allocate_frames(&mut self, n: u64) -> Option<u64> {
        if n == 0 {
            return Some(0);
        }
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

    #[inline]
    pub fn frame_to_virt(&self, frame: u64) -> u64 {
        frame * FRAME_SIZE + self.phys_offset
    }

    pub fn report_base(&self) -> MemMapReport {
        MemMapReport {
            regions: self.regions,
            usable_regions: self.usable_regions,
            usable_frames: self.usable_frames,
            total_frames_seen: self.total_frames_seen,
            allocated: self.allocated,
            free_remaining: self.usable_frames.saturating_sub(self.allocated),
            phys_offset: self.phys_offset,
            heap_base_phys: 0,
            heap_bytes: 0,
            heap_used: 0,
            heap_write_ok: false,
        }
    }
}

/// Bump heap carved from allocated physical frames (virt-mapped).
pub struct KernelHeap {
    base_virt: u64,
    base_phys: u64,
    capacity: u64,
    used: AtomicU64,
}

impl KernelHeap {
    pub const fn empty() -> Self {
        Self {
            base_virt: 0,
            base_phys: 0,
            capacity: 0,
            used: AtomicU64::new(0),
        }
    }

    pub fn from_frames(alloc: &FrameAllocator, start_frame: u64, n_frames: u64) -> Self {
        let base_phys = start_frame * FRAME_SIZE;
        let base_virt = alloc.frame_to_virt(start_frame);
        let capacity = n_frames * FRAME_SIZE;
        unsafe {
            let p = base_virt as *mut u8;
            ptr::write_bytes(p, 0, capacity as usize);
        }
        Self {
            base_virt,
            base_phys,
            capacity,
            used: AtomicU64::new(0),
        }
    }

    pub fn alloc(&self, size: usize, align: usize) -> Option<*mut u8> {
        if size == 0 || self.capacity == 0 {
            return None;
        }
        let align = align.max(8) as u64;
        let size = size as u64;
        loop {
            let cur = self.used.load(Ordering::Relaxed);
            let aligned = (cur + align - 1) & !(align - 1);
            let next = match aligned.checked_add(size) {
                Some(n) => n,
                None => return None,
            };
            if next > self.capacity {
                return None;
            }
            if self
                .used
                .compare_exchange(cur, next, Ordering::SeqCst, Ordering::Relaxed)
                .is_ok()
            {
                return Some((self.base_virt + aligned) as *mut u8);
            }
        }
    }

    pub fn used_bytes(&self) -> u64 {
        self.used.load(Ordering::Relaxed)
    }

    pub fn capacity_bytes(&self) -> u64 {
        self.capacity
    }

    pub fn base_phys(&self) -> u64 {
        self.base_phys
    }

    pub fn write_readback_ok(&self) -> bool {
        let p = match self.alloc(64, 8) {
            Some(p) => p,
            None => return false,
        };
        unsafe {
            for i in 0..64 {
                ptr::write_volatile(p.add(i), 0xA5 ^ (i as u8));
            }
            for i in 0..64 {
                if ptr::read_volatile(p.add(i)) != (0xA5 ^ (i as u8)) {
                    return false;
                }
            }
        }
        true
    }
}

/// Boot self-test: map offset, allocate frames, build heap, write/read.
pub fn boot_mem_selftest(map: &MemoryMap, phys_offset: u64) -> (bool, MemMapReport, u64) {
    let mut alloc = FrameAllocator::from_memory_map(map, phys_offset);
    let mut first = 0u64;
    let mut got = 0u64;
    let mut touch_ok = true;
    let mut i = 0u64;
    while i < 16 {
        if let Some(f) = alloc.allocate_frame() {
            if got == 0 {
                first = f;
            }
            if phys_offset != 0 {
                let v = alloc.frame_to_virt(f) as *mut u64;
                unsafe {
                    ptr::write_volatile(v, FRAME_MARKER);
                    if ptr::read_volatile(v) != FRAME_MARKER {
                        touch_ok = false;
                    }
                }
            }
            got += 1;
        }
        i += 1;
    }

    let heap_frames = 32u64;
    let heap_start = alloc.allocate_frames(heap_frames);
    let mut heap_ok = false;
    let mut heap_base = 0u64;
    let mut heap_bytes = 0u64;
    let mut heap_used = 0u64;
    if let Some(sf) = heap_start {
        heap_base = sf * FRAME_SIZE;
        heap_bytes = heap_frames * FRAME_SIZE;
        if phys_offset != 0 {
            let heap = KernelHeap::from_frames(&alloc, sf, heap_frames);
            heap_ok = heap.write_readback_ok();
            heap_used = heap.used_bytes();
            heap_base = heap.base_phys();
            heap_bytes = heap.capacity_bytes();
        }
    }

    let mut rep = alloc.report_base();
    rep.heap_base_phys = heap_base;
    rep.heap_bytes = heap_bytes;
    rep.heap_used = heap_used;
    rep.heap_write_ok = heap_ok;

    let ok = rep.usable_frames > 100
        && rep.usable_regions > 0
        && got == 16
        && heap_start.is_some()
        && rep.allocated >= 48
        && touch_ok
        && (phys_offset == 0 || heap_ok);
    (ok, rep, first)
}
