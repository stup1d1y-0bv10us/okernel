pub const IDENTITY_MAP_START: u64 = 0x000000;

pub const IDENTITY_MAP_END: u64 = 0x20_0000;

pub const LOW_EXCLUSION_START: u64 = 0x000000;

pub const LOW_EXCLUSION_END: u64 = 0x100000;

pub const PAGE_SIZE: u64 = 4096;

pub fn intersect_ranges(a_start: u64, a_end: u64, b_start: u64, b_end: u64) -> Option<(u64, u64)> {
    if a_start >= a_end || b_start >= b_end {
        return None;
    }
    let start = if a_start > b_start { a_start } else { b_start };
    let end = if a_end < b_end { a_end } else { b_end };
    if start >= end {
        return None;
    }
    Some((start, end))
}

pub fn align_up(value: u64, align: u64) -> Option<u64> {
    let mask = align.checked_sub(1)?;
    value.checked_add(mask).map(|adjusted| adjusted & !mask)
}

pub fn align_down(value: u64, align: u64) -> Option<u64> {
    let mask = align.checked_sub(1)?;
    Some(value & !mask)
}

pub fn frames_of_range(start: u64, end: u64) -> Option<(u64, u64, u64)> {
    if start >= end {
        return None;
    }
    let first_byte = align_up(start, PAGE_SIZE)?;
    let end_byte = align_down(end, PAGE_SIZE)?;
    if first_byte >= end_byte {
        return None;
    }
    let first_frame = first_byte / PAGE_SIZE;
    let end_frame = end_byte / PAGE_SIZE;
    let count = end_frame.checked_sub(first_frame)?;
    if count == 0 {
        return None;
    }
    Some((first_frame, end_frame, count))
}

pub fn checked_range_end(base: u64, length: u64) -> Option<u64> {
    if length == 0 {
        return None;
    }
    base.checked_add(length)
}

pub fn descriptor_eligible(region_type: u32, attributes: u32) -> bool {
    region_type == 1 && attributes & 1 != 0
}

pub fn subtract_reservation(
    cand_start: u64,
    cand_end: u64,
    res_start: u64,
    res_end: u64,
) -> [Option<(u64, u64)>; 2] {
    if cand_start >= cand_end {
        return [None, None];
    }
    if res_start >= res_end {
        return [Some((cand_start, cand_end)), None];
    }
    if cand_end <= res_start || cand_start >= res_end {
        return [Some((cand_start, cand_end)), None];
    }
    let first_end = res_start.min(cand_end);
    let first = if cand_start < first_end {
        Some((cand_start, first_end))
    } else {
        None
    };
    let second_start = res_end.max(cand_start);
    let second = if second_start < cand_end {
        Some((second_start, cand_end))
    } else {
        None
    };
    [first, second]
}