use super::okbi::BootInfo;
use crate::drivers::serial::{
    serial_write_bytes, serial_write_hex_u32_full, serial_write_hex_u32_minimal,
    serial_write_hex_u64_full,
};
use crate::memory::ranges;

#[repr(C)]
pub(crate) struct E820Entry {
    pub(crate) base: u64,
    pub(crate) length: u64,
    pub(crate) region_type: u32,
    pub(crate) attributes: u32,
}

const _: () = assert!(core::mem::size_of::<E820Entry>() == 24);

pub(crate) const E820_MAX_ENTRIES: u32 = 32;

pub(crate) const E820_ENTRY_STRIDE: usize = 24;

const IDENTITY_MAP_LIMIT: usize = 0x20_0000;

pub(crate) fn e820_diagnostics(info: &BootInfo) -> bool {
    let count = info.e820_count;
    if count == 0 || count > E820_MAX_ENTRIES {
        serial_write_bytes(b"OKernel: E820 INVALID count\r\n");
        return false;
    }

    if info.e820_entry_size != E820_ENTRY_STRIDE as u32 {
        serial_write_bytes(b"OKernel: E820 INVALID stride\r\n");
        return false;
    }

    if info.e820_address != 0x5000 {
        serial_write_bytes(b"OKernel: E820 INVALID address\r\n");
        return false;
    }

    let Some(total) = (count as usize).checked_mul(E820_ENTRY_STRIDE) else {
        serial_write_bytes(b"OKernel: E820 INVALID range\r\n");
        return false;
    };
    let start = info.e820_address as usize;
    let Some(end) = start.checked_add(total) else {
        serial_write_bytes(b"OKernel: E820 INVALID range\r\n");
        return false;
    };
    if end > IDENTITY_MAP_LIMIT {
        serial_write_bytes(b"OKernel: E820 INVALID range\r\n");
        return false;
    }

    serial_write_bytes(b"OKernel: E820 begin\r\n");

    let mut prev_end: Option<u64> = None;

    for index in 0..count {
        let address = start + (index as usize) * E820_ENTRY_STRIDE;

        let entry = unsafe { core::ptr::read(address as *const E820Entry) };

        let Some(end) = ranges::checked_range_end(entry.base, entry.length) else {
            serial_write_bytes(b"OKernel: E820 INVALID entry ");
            serial_write_hex_u32_minimal(index);
            serial_write_bytes(b"\r\n");
            return false;
        };

        if entry.region_type == 0 {
            serial_write_bytes(b"OKernel: E820 INVALID type ");
            serial_write_hex_u32_minimal(index);
            serial_write_bytes(b"\r\n");
            return false;
        }

        if let Some(prev) = prev_end {
            if entry.base < prev {
                serial_write_bytes(b"OKernel: E820 INVALID order ");
                serial_write_hex_u32_minimal(index);
                serial_write_bytes(b"\r\n");
                return false;
            }
        }
        prev_end = Some(end);

        let enabled = entry.attributes & 1 != 0;
        let eligible = entry.region_type == 1 && enabled;

        serial_write_bytes(b"E820[");
        serial_write_hex_u32_minimal(index);
        serial_write_bytes(b"]: base=");
        serial_write_hex_u64_full(entry.base);
        serial_write_bytes(b" len=");
        serial_write_hex_u64_full(entry.length);
        serial_write_bytes(b" type=");
        serial_write_hex_u32_full(entry.region_type);
        serial_write_bytes(b" attr=");
        serial_write_hex_u32_full(entry.attributes);
        serial_write_bytes(b" enabled=");
        serial_write_hex_u32_minimal(enabled as u32);
        serial_write_bytes(b" eligible=");
        serial_write_hex_u32_minimal(eligible as u32);
        serial_write_bytes(b"\r\n");
    }

    serial_write_bytes(b"OKernel: E820 valid\r\n");
    true
}

pub(crate) fn frame_planning(info: &BootInfo) -> bool {

    let count = info.e820_count;
    if count == 0 || count > E820_MAX_ENTRIES {
        serial_write_bytes(b"OKernel: frame planning INVALID count\r\n");
        return false;
    }
    if info.e820_entry_size != E820_ENTRY_STRIDE as u32 || info.e820_address != 0x5000 {
        serial_write_bytes(b"OKernel: frame planning INVALID header\r\n");
        return false;
    }
    let map_start = info.e820_address as u64;
    let Some(map_bytes) = (count as u64).checked_mul(E820_ENTRY_STRIDE as u64) else {
        serial_write_bytes(b"OKernel: frame planning INVALID range\r\n");
        return false;
    };
    let Some(map_end) = map_start.checked_add(map_bytes) else {
        serial_write_bytes(b"OKernel: frame planning INVALID range\r\n");
        return false;
    };
    if map_end > ranges::IDENTITY_MAP_END {
        serial_write_bytes(b"OKernel: frame planning INVALID range\r\n");
        return false;
    }

    serial_write_bytes(b"OKernel: frame planning begin\r\n");

    let mut reported: u32 = 0;
    let mut index: u32 = 0;
    while index < count {
        let Some(entry_offset) = (index as u64).checked_mul(E820_ENTRY_STRIDE as u64) else {
            serial_write_bytes(b"OKernel: frame planning INVALID range\r\n");
            return false;
        };
        let Some(entry_addr) = map_start.checked_add(entry_offset) else {
            serial_write_bytes(b"OKernel: frame planning INVALID range\r\n");
            return false;
        };

        let entry = unsafe { core::ptr::read(entry_addr as *const E820Entry) };

        let Some(entry_end) = ranges::checked_range_end(entry.base, entry.length) else {
            serial_write_bytes(b"OKernel: frame planning INVALID entry ");
            serial_write_hex_u32_minimal(index);
            serial_write_bytes(b"\r\n");
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
                        if let Some((first_frame, end_frame, frame_count)) =
                            ranges::frames_of_range(piece_start, piece_end)
                        {
                            serial_write_bytes(b"Candidate[");
                            serial_write_hex_u32_minimal(reported);
                            serial_write_bytes(b"]: start=");
                            serial_write_hex_u64_full(piece_start);
                            serial_write_bytes(b" end=");
                            serial_write_hex_u64_full(piece_end);
                            serial_write_bytes(b" first_frame=");
                            serial_write_hex_u64_full(first_frame);
                            serial_write_bytes(b" end_frame=");
                            serial_write_hex_u64_full(end_frame);
                            serial_write_bytes(b" count=0x");
                            serial_write_hex_u32_minimal(frame_count as u32);
                            serial_write_bytes(b"\r\n");
                            reported += 1;
                        }
                    }
                }
            }
        }

        index += 1;
    }

    serial_write_bytes(b"OKernel: frame planning valid\r\n");
    true
}