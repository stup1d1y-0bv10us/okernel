#[path = "../../src/memory/mod.rs"]
mod memory;

fn main() {
    println!("range-test harness: run with `cargo test`");
}

#[cfg(test)]
mod tests {
    use super::memory::ranges;

    #[test]
    fn clip_partly_outside_mapped_range() {
        assert_eq!(
            ranges::intersect_ranges(
                0x100000,
                0x7FE0000,
                ranges::IDENTITY_MAP_START,
                ranges::IDENTITY_MAP_END,
            ),
            Some((0x100000, 0x200000))
        );
    }

    #[test]
    fn empty_after_clipping() {
        assert_eq!(
            ranges::intersect_ranges(
                0xFD00000000,
                0x10000000000,
                ranges::IDENTITY_MAP_START,
                ranges::IDENTITY_MAP_END,
            ),
            None
        );
        assert_eq!(ranges::intersect_ranges(0x100000, 0x200000, 0x300000, 0x400000), None);

        assert_eq!(ranges::intersect_ranges(0x100000, 0x200000, 0x200000, 0x300000), None);
    }

    #[test]
    fn inward_alignment() {
        assert_eq!(ranges::align_up(0x100100, 4096), Some(0x101000));
        assert_eq!(ranges::align_down(0x1FFF00, 4096), Some(0x1FF000));

        assert_eq!(ranges::align_up(0x100000, 4096), Some(0x100000));
        assert_eq!(ranges::align_down(0x200000, 4096), Some(0x200000));

        assert_eq!(ranges::align_up(99, 0), None);
        assert_eq!(ranges::align_down(99, 0), None);
    }

    #[test]
    fn frame_conversion() {
        assert_eq!(
            ranges::frames_of_range(0x101000, 0x1FF000),
            Some((0x101, 0x1FF, 0xFE))
        );
        assert_eq!(
            ranges::frames_of_range(0x100000, 0x200000),
            Some((0x100, 0x200, 0x100))
        );

        assert_eq!(ranges::frames_of_range(0x100000, 0x101000), Some((0x100, 0x101, 1)));

        assert_eq!(ranges::frames_of_range(0x100000, 0x100FFF), None);
        assert_eq!(ranges::frames_of_range(0x100000, 0x100000), None);
        assert_eq!(ranges::frames_of_range(0x200000, 0x100000), None);
    }

    #[test]
    fn disabled_type1_is_ineligible() {
        assert!(ranges::descriptor_eligible(1, 1));
        assert!(!ranges::descriptor_eligible(1, 0));

        assert!(ranges::descriptor_eligible(1, 0xFFFFFFFF));
        assert!(!ranges::descriptor_eligible(1, 0xFFFFFFFE));
    }

    #[test]
    fn non_type1_is_ineligible() {
        assert!(!ranges::descriptor_eligible(2, 1));
        assert!(!ranges::descriptor_eligible(5, 1));
        assert!(!ranges::descriptor_eligible(0, 1));
        assert!(!ranges::descriptor_eligible(0xFFFFFFFF, 0xFFFFFFFF));
    }

    #[test]
    fn arithmetic_overflow_fails_closed() {
        assert_eq!(ranges::checked_range_end(0xFFFFFFFFFFFFF000, 0x2000), None);
        assert_eq!(ranges::checked_range_end(0xFFFFFFFFFFFFFFFF, 1), None);
        assert_eq!(ranges::checked_range_end(0x1000, 0), None);
        assert_eq!(ranges::checked_range_end(0x1000, 0x2000), Some(0x3000));
        assert_eq!(ranges::align_up(u64::MAX, 4096), None);
        assert_eq!(ranges::align_up(u64::MAX - 4095, 4096), Some(u64::MAX & !4095));
        assert_eq!(ranges::frames_of_range(u64::MAX - 4094, u64::MAX), None);
    }

    #[test]
    fn exact_boundary_touch_keeps_range() {
        assert_eq!(
            ranges::subtract_reservation(0x100000, 0x200000, 0x000000, 0x100000),
            [Some((0x100000, 0x200000)), None]
        );

        assert_eq!(
            ranges::subtract_reservation(0x100000, 0x200000, 0x200000, 0x300000),
            [Some((0x100000, 0x200000)), None]
        );
    }

    #[test]
    fn interior_reservation_splits_in_two() {
        let [below, above] =
            ranges::subtract_reservation(0x100000, 0x200000, 0x180000, 0x190000);
        assert_eq!(below, Some((0x100000, 0x180000)));
        assert_eq!(above, Some((0x190000, 0x200000)));

        let (a_start, a_end) = below.unwrap();
        let (b_start, b_end) = above.unwrap();
        assert!(a_end <= 0x180000);
        assert!(b_start >= 0x190000);
        assert_eq!(a_end - a_start + (b_end - b_start), 0x200000 - 0x100000 - 0x10000);
    }

    #[test]
    fn reservation_excludes_intersecting_frames() {
        assert_eq!(
            ranges::subtract_reservation(0x100000, 0x200000, 0x000000, 0x1000000),
            [None, None]
        );
        assert_eq!(
            ranges::subtract_reservation(0x100000, 0x101000, 0x100000, 0x101000),
            [None, None]
        );
        assert_eq!(
            ranges::subtract_reservation(0x100000, 0x101000, 0x200000, 0x300000),
            [Some((0x100000, 0x101000)), None]
        );
    }

    #[test]
    fn malformed_inputs_fail_closed() {
        assert_eq!(ranges::intersect_ranges(9, 3, 0, 99), None);
        assert_eq!(ranges::intersect_ranges(5, 5, 0, 99), None);
        assert_eq!(ranges::intersect_ranges(0, 99, 7, 7), None);
        assert_eq!(
            ranges::subtract_reservation(9, 3, 0, 99),
            [None, None]
        );
        assert_eq!(
            ranges::subtract_reservation(5, 9, 7, 7),
            [Some((5, 9)), None]
        );
        assert_eq!(ranges::frames_of_range(0, 4095), None);
    }

    #[test]
    fn reservation_policy_constants() {
        assert_eq!(ranges::IDENTITY_MAP_START, 0x000000);
        assert_eq!(ranges::IDENTITY_MAP_END, 0x20_0000);
        assert_eq!(ranges::LOW_EXCLUSION_START, 0x000000);
        assert_eq!(ranges::LOW_EXCLUSION_END, 0x100000);
        assert_eq!(ranges::PAGE_SIZE, 4096);
    }
}

#[cfg(test)]
mod allocator_tests {
    use super::memory::allocator::FrameAllocator;
    use super::memory::allocator::{MANAGED_END, MANAGED_START};

    fn full_allocator() -> FrameAllocator {
        let mut allocator = FrameAllocator::new();
        assert!(allocator.add_eligible_range(0x100, 0x200));
        assert!(allocator.finish_init());
        allocator
    }

    fn drain_all(allocator: &mut FrameAllocator) -> Vec<u64> {
        let mut frames = Vec::new();
        while let Some(frame) = allocator.alloc() {
            frames.push(frame);
        }
        frames
    }

    #[test]
    fn fresh_allocator_is_unavailable() {
        let mut allocator = FrameAllocator::new();
        assert_eq!(allocator.stats(), (0, 0, 0));
        assert_eq!(allocator.alloc(), None);
        assert!(!allocator.free(0x100000));
        assert_eq!(allocator.stats(), (0, 0, 0));
    }

    #[test]
    fn init_full_range_gives_256_frames() {
        let allocator = full_allocator();
        assert_eq!(allocator.stats(), (256, 0, 256));
    }

    #[test]
    fn disjoint_ranges_leave_gap_ineligible() {
        let mut allocator = FrameAllocator::new();
        assert!(allocator.add_eligible_range(0x100, 0x180));
        assert!(allocator.add_eligible_range(0x1C0, 0x200));
        assert!(allocator.finish_init());
        assert_eq!(allocator.stats(), (192, 0, 192));

        assert!(!allocator.free(0x180000));
        assert!(!allocator.free(0x18F000));
        let frames = drain_all(&mut allocator);
        assert_eq!(frames.len(), 192);
        for frame in &frames {
            assert!(*frame < 0x180000 || *frame >= 0x1C0000);
        }
        assert_eq!(allocator.stats(), (192, 192, 0));
    }

    #[test]
    fn invalid_ranges_rejected() {
        let mut allocator = FrameAllocator::new();
        assert!(!allocator.add_eligible_range(0x600, 0x700));
        assert!(!allocator.add_eligible_range(0x200, 0x300));
        assert!(!allocator.add_eligible_range(0x300, 0x300));
        assert!(!allocator.add_eligible_range(0x400, 0x300));
        assert!(!allocator.add_eligible_range(0x200, 0x600));
        assert_eq!(allocator.stats(), (0, 0, 0));
        assert_eq!(allocator.alloc(), None);
    }

    #[test]
    fn failed_init_exposes_nothing() {
        let mut allocator = FrameAllocator::new();
        assert!(allocator.add_eligible_range(0x100, 0x190));
        assert!(!allocator.add_eligible_range(0x900, 0x999));
        assert_eq!(allocator.stats(), (0, 0, 0));
        assert_eq!(allocator.alloc(), None);
        assert!(allocator.add_eligible_range(0x190, 0x1A0));
        assert!(allocator.finish_init());
        assert_eq!(allocator.stats(), (0xA0, 0, 0xA0));
    }

    #[test]
    fn reinit_refused_with_allocations_intact() {
        let mut allocator = full_allocator();
        let kept = [
            allocator.alloc().unwrap(),
            allocator.alloc().unwrap(),
            allocator.alloc().unwrap(),
        ];
        assert!(!allocator.add_eligible_range(0x100, 0x101));
        assert!(!allocator.finish_init());
        assert_eq!(allocator.stats(), (256, 3, 253));
        for frame in kept {
            assert!(allocator.free(frame));
        }
        assert_eq!(allocator.stats(), (256, 0, 256));
    }

    #[test]
    fn first_alloc_returns_base() {
        let mut allocator = full_allocator();
        assert_eq!(allocator.alloc(), Some(0x100000));
    }

    #[test]
    fn drain_returns_every_frame_once() {
        let mut allocator = full_allocator();
        let mut frames = drain_all(&mut allocator);
        assert_eq!(frames.len(), 256);
        for frame in &frames {
            assert_eq!(*frame % 4096, 0);
            assert!(*frame >= MANAGED_START && *frame < MANAGED_END);
        }
        frames.sort_unstable();
        let expected: Vec<u64> = (0x100..0x200).map(|f| f * 4096).collect();
        assert_eq!(frames, expected);
        let (eligible, allocated, free) = allocator.stats();
        assert_eq!((eligible, allocated, free), (256, 256, 0));
        assert_eq!(eligible - allocated, free);
    }

    #[test]
    fn allocations_stay_in_domain() {
        let mut allocator = full_allocator();
        for _ in 0..256 {
            let frame = allocator.alloc().unwrap();
            assert_eq!(frame % 4096, 0);
            assert!(frame >= MANAGED_START && frame < MANAGED_END);
            let index = ((frame - MANAGED_START) / 4096) as usize;
            assert!(index < 256);
        }
        assert_eq!(allocator.alloc(), None);
    }

    #[test]
    fn over_allocate_fails_cleanly() {
        let mut allocator = full_allocator();
        for _ in 0..256 {
            assert!(allocator.alloc().is_some());
        }
        let drained = allocator.stats();
        assert_eq!(drained, (256, 256, 0));
        assert_eq!(allocator.alloc(), None);
        assert_eq!(allocator.stats(), drained);
    }

    #[test]
    fn freed_frame_reused_without_duplication() {
        let mut allocator = full_allocator();
        let mut held = Vec::new();
        for _ in 0..4 {
            held.push(allocator.alloc().unwrap());
        }
        let victim = held[1];
        assert!(allocator.free(victim));
        assert_eq!(allocator.alloc(), Some(victim));
        let mut rest = drain_all(&mut allocator);
        let mut all = held;
        all.extend(rest.drain(..));
        all.sort_unstable();
        let expected: Vec<u64> = (0x100..0x200).map(|f| f * 4096).collect();
        assert_eq!(all, expected);
    }

    #[test]
    fn free_valid_frame() {
        let mut allocator = full_allocator();
        let frame = allocator.alloc().unwrap();
        assert_eq!(allocator.stats(), (256, 1, 255));
        assert!(allocator.free(frame));
        assert_eq!(allocator.stats(), (256, 0, 256));
    }

    #[test]
    fn free_unaligned_rejected() {
        let mut allocator = full_allocator();
        let before = allocator.stats();
        assert!(!allocator.free(0x100001));
        assert!(!allocator.free(0x100FFF));
        assert!(!allocator.free(0x1FFFFF));
        assert_eq!(allocator.stats(), before);
    }

    #[test]
    fn free_out_of_range_rejected() {
        let mut allocator = full_allocator();
        let before = allocator.stats();
        assert!(!allocator.free(0xFF000));
        assert!(!allocator.free(0x200000));
        assert!(!allocator.free(0x10000000000));
        assert_eq!(allocator.stats(), before);
    }

    #[test]
    fn free_gap_frame_rejected() {
        let mut allocator = FrameAllocator::new();
        assert!(allocator.add_eligible_range(0x100, 0x180));
        assert!(allocator.add_eligible_range(0x1C0, 0x200));
        assert!(allocator.finish_init());
        let before = allocator.stats();
        assert!(!allocator.free(0x180000));
        assert!(!allocator.free(0x1BFFF0));
        assert_eq!(allocator.stats(), before);
    }

    #[test]
    fn free_never_allocated_rejected() {
        let mut allocator = full_allocator();
        let before = allocator.stats();
        assert!(!allocator.free(0x100000));
        assert!(!allocator.free(0x1FF000));
        assert_eq!(allocator.stats(), before);
    }

    #[test]
    fn double_free_rejected() {
        let mut allocator = full_allocator();
        let frame = allocator.alloc().unwrap();
        assert!(allocator.free(frame));
        assert!(!allocator.free(frame));
        assert_eq!(allocator.stats(), (256, 0, 256));
    }

    #[test]
    fn rejected_frees_change_nothing() {
        let mut allocator = full_allocator();
        let frame = allocator.alloc().unwrap();
        assert!(allocator.free(frame));
        let settled = allocator.stats();
        assert!(!allocator.free(frame));
        assert!(!allocator.free(0x100001));
        assert!(!allocator.free(0xFF000));
        assert!(!allocator.free(0x200000));
        assert!(!allocator.free(0x123456));
        assert_eq!(allocator.stats(), settled);
        assert_eq!(settled, (256, 0, 256));
    }
}