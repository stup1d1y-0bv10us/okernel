use super::allocator;
use super::ranges;
use crate::boot::e820::{E820Entry, E820_ENTRY_STRIDE, E820_MAX_ENTRIES};
use crate::boot::okbi::BootInfo;
use crate::drivers::serial::{
    serial_write_bytes, serial_write_hex_u32_minimal, serial_write_hex_u64_full,
};

static mut ALLOCATOR: allocator::FrameAllocator = allocator::FrameAllocator::new();

pub(crate) fn allocator_init(info: &BootInfo) -> bool {
    let count = info.e820_count;
    if count == 0 || count > E820_MAX_ENTRIES {
        return false;
    }
    if info.e820_entry_size != E820_ENTRY_STRIDE as u32 || info.e820_address != 0x5000 {
        return false;
    }
    let map_start = info.e820_address as u64;
    let Some(map_bytes) = (count as u64).checked_mul(E820_ENTRY_STRIDE as u64) else {
        return false;
    };
    let Some(map_end) = map_start.checked_add(map_bytes) else {
        return false;
    };
    if map_end > ranges::IDENTITY_MAP_END {
        return false;
    }

    let mut index: u32 = 0;
    while index < count {
        let Some(entry_offset) = (index as u64).checked_mul(E820_ENTRY_STRIDE as u64) else {
            return false;
        };
        let Some(entry_addr) = map_start.checked_add(entry_offset) else {
            return false;
        };

        let entry = unsafe { core::ptr::read(entry_addr as *const E820Entry) };
        let Some(entry_end) = ranges::checked_range_end(entry.base, entry.length) else {
            return false;
        };
        if ranges::descriptor_eligible(entry.region_type, entry.attributes) {
            if let Some((clipped_start, clipped_end)) = ranges::intersect_ranges(
                entry.base,
                entry_end,
                ranges::IDENTITY_MAP_START,
                ranges::IDENTITY_MAP_END,
            ) {
                for piece in ranges::subtract_reservation(
                    clipped_start,
                    clipped_end,
                    ranges::LOW_EXCLUSION_START,
                    ranges::LOW_EXCLUSION_END,
                ) {
                    if let Some((piece_start, piece_end)) = piece {
                        if let Some((first_frame, end_frame, _)) =
                            ranges::frames_of_range(piece_start, piece_end)
                        {
                            let added = unsafe {
                                let allocator =
                                    &mut *core::ptr::addr_of_mut!(ALLOCATOR);
                                allocator.add_eligible_range(first_frame, end_frame)
                            };
                            if !added {
                                return false;
                            }
                        }
                    }
                }
            }
        }
        index += 1;
    }

    unsafe {
        let allocator = &mut *core::ptr::addr_of_mut!(ALLOCATOR);
        if !allocator.finish_init() {
            return false;
        }
    }
    true
}

fn allocator_stats() -> (u32, u32, u32) {
    unsafe {
        let allocator = core::ptr::addr_of!(ALLOCATOR);
        (*allocator).stats()
    }
}

fn frame_alloc() -> Option<u64> {
    unsafe {
        let allocator = core::ptr::addr_of_mut!(ALLOCATOR);
        (*allocator).alloc()
    }
}

fn frame_free(address: u64) -> bool {
    unsafe {
        let allocator = core::ptr::addr_of_mut!(ALLOCATOR);
        (*allocator).free(address)
    }
}

pub(crate) fn allocator_smoke_test() -> bool {
    let (eligible, allocated, free) = allocator_stats();
    serial_write_bytes(b"OKernel: allocator init eligible=0x");
    serial_write_hex_u32_minimal(eligible);
    serial_write_bytes(b" allocated=0x");
    serial_write_hex_u32_minimal(allocated);
    serial_write_bytes(b" free=0x");
    serial_write_hex_u32_minimal(free);
    serial_write_bytes(b"\r\n");

    let Some(frame) = frame_alloc() else {
        serial_write_bytes(b"OKernel: allocator SMOKE FAIL alloc\r\n");
        return false;
    };
    if frame % ranges::PAGE_SIZE != 0
        || frame < allocator::MANAGED_START
        || frame >= allocator::MANAGED_END
    {
        serial_write_bytes(b"OKernel: allocator SMOKE FAIL address\r\n");
        return false;
    }
    serial_write_bytes(b"OKernel: alloc frame=");
    serial_write_hex_u64_full(frame);
    serial_write_bytes(b"\r\n");

    let (eligible2, allocated2, free2) = allocator_stats();
    if eligible2 != eligible || allocated2 != allocated + 1 || free2 != free - 1 {
        serial_write_bytes(b"OKernel: allocator SMOKE FAIL counts\r\n");
        return false;
    }
    if !frame_free(frame) {
        serial_write_bytes(b"OKernel: allocator SMOKE FAIL free\r\n");
        return false;
    }
    let (eligible3, allocated3, free3) = allocator_stats();
    if eligible3 != eligible || allocated3 != allocated || free3 != free {
        serial_write_bytes(b"OKernel: allocator SMOKE FAIL restore\r\n");
        return false;
    }
    if frame_free(frame) {
        serial_write_bytes(b"OKernel: allocator SMOKE FAIL double-free\r\n");
        return false;
    }
    let (eligible4, allocated4, free4) = allocator_stats();
    if eligible4 != eligible || allocated4 != allocated || free4 != free {
        serial_write_bytes(b"OKernel: allocator SMOKE FAIL stable\r\n");
        return false;
    }

    serial_write_bytes(b"OKernel: allocator smoke ok\r\n");
    true
}