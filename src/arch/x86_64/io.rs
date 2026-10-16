#[inline(always)]
pub(crate) unsafe fn outb(port: u16, value: u8) {

    unsafe {
        core::arch::asm!(
            "out dx, al",
            in("dx") port,
            in("al") value,
            options(nomem, nostack, preserves_flags)
        );
    }
}

#[inline(always)]
pub(crate) unsafe fn inb(port: u16) -> u8 {
    let value: u8;

    unsafe {
        core::arch::asm!(
            "in al, dx",
            in("dx") port,
            out("al") value,
            options(nomem, nostack, preserves_flags)
        );
    }

    value
}

pub(crate) fn io_wait() {

    unsafe {
        outb(0x80, 0);
    }
}