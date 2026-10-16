use super::ranges;

pub const MANAGED_START: u64 = 0x100000;
pub const MANAGED_END: u64 = 0x200000;

pub const MANAGED_FIRST_FRAME: u64 = 0x100;
pub const MANAGED_END_FRAME: u64 = 0x200;

pub const BITMAP_WORDS: usize = 4;

pub struct FrameAllocator {

    eligible: [u64; BITMAP_WORDS],

    allocated: [u64; BITMAP_WORDS],

    initialized: bool,
}

impl FrameAllocator {

    pub const fn new() -> Self {
        Self {
            eligible: [0; BITMAP_WORDS],
            allocated: [0; BITMAP_WORDS],
            initialized: false,
        }
    }

    pub fn add_eligible_range(&mut self, first_frame: u64, end_frame: u64) -> bool {
        if self.initialized {
            return false;
        }
        if first_frame < MANAGED_FIRST_FRAME
            || end_frame > MANAGED_END_FRAME
            || first_frame >= end_frame
        {
            return false;
        }
        let mut frame = first_frame;
        while frame < end_frame {
            let Some((word, mask)) = frame_to_bit(frame) else {
                return false;
            };
            self.eligible[word] |= mask;
            frame += 1;
        }
        true
    }

    pub fn finish_init(&mut self) -> bool {
        if self.initialized {
            return false;
        }
        self.initialized = true;
        true
    }

    pub fn alloc(&mut self) -> Option<u64> {
        if !self.initialized {
            return None;
        }
        let mut word = 0;
        while word < BITMAP_WORDS {
            let free = self.eligible[word] & !self.allocated[word];
            if free != 0 {
                let bit = free.trailing_zeros() as usize;
                self.allocated[word] |= 1u64 << bit;
                let frame = MANAGED_FIRST_FRAME + word as u64 * 64 + bit as u64;
                return Some(frame * ranges::PAGE_SIZE);
            }
            word += 1;
        }
        None
    }

    pub fn free(&mut self, address: u64) -> bool {
        if address % ranges::PAGE_SIZE != 0 {
            return false;
        }
        if address < MANAGED_START || address >= MANAGED_END {
            return false;
        }
        let Some(offset) = address.checked_sub(MANAGED_START) else {
            return false;
        };
        let frame = MANAGED_FIRST_FRAME + offset / ranges::PAGE_SIZE;
        let Some((word, mask)) = frame_to_bit(frame) else {
            return false;
        };
        if !self.initialized {
            return false;
        }
        if self.eligible[word] & mask == 0 {
            return false;
        }
        if self.allocated[word] & mask == 0 {
            return false;
        }
        self.allocated[word] &= !mask;
        true
    }

    pub fn stats(&self) -> (u32, u32, u32) {
        if !self.initialized {
            return (0, 0, 0);
        }
        let mut eligible = 0u32;
        let mut allocated = 0u32;
        let mut word = 0;
        while word < BITMAP_WORDS {
            eligible += self.eligible[word].count_ones();
            allocated += self.allocated[word].count_ones();
            word += 1;
        }
        (eligible, allocated, eligible - allocated)
    }
}

fn frame_to_bit(frame: u64) -> Option<(usize, u64)> {
    if frame < MANAGED_FIRST_FRAME || frame >= MANAGED_END_FRAME {
        return None;
    }
    let bit = (frame - MANAGED_FIRST_FRAME) as usize;
    Some((bit / 64, 1u64 << (bit % 64)))
}