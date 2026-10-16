use super::io::outb;

const PIT_CHANNEL0: u16 = 0x40;
const PIT_COMMAND: u16 = 0x43;

const PIT_INPUT_HZ: u32 = 1_193_182;
const PIT_DIVISOR: u16 = 11932;

pub(crate) fn pit_init() {
    debug_assert_pit_divisor();
    unsafe {
        outb(PIT_COMMAND, 0x36);
        outb(PIT_CHANNEL0, (PIT_DIVISOR & 0xff) as u8);
        outb(PIT_CHANNEL0, (PIT_DIVISOR >> 8) as u8);
    }
}

fn debug_assert_pit_divisor() {
    const _: () = assert!(PIT_DIVISOR != 0);
    const _: () = assert!(PIT_INPUT_HZ / PIT_DIVISOR as u32 >= 90);
    const _: () = assert!(PIT_INPUT_HZ / PIT_DIVISOR as u32 <= 110);
}