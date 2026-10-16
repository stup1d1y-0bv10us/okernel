use core::sync::atomic::{AtomicU64, Ordering};

use super::pic;
use super::pit::pit_init;

pub(crate) const TIMER_TEST_TICKS: u64 = 10;

static TICKS: AtomicU64 = AtomicU64::new(0);

pub(crate) fn timer_init() {
    pic::pic_init();
    pit_init();
    TICKS.store(0, Ordering::Relaxed);
}

pub(crate) fn ticks() -> u64 {
    TICKS.load(Ordering::Relaxed)
}

#[unsafe(no_mangle)]
pub extern "C" fn irq0_handler() {
    TICKS.fetch_add(1, Ordering::Relaxed);
    pic::eoi_master();
}