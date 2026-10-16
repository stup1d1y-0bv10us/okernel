use core::arch::asm;

use super::gdt::df_stack_bounds;
use crate::drivers::serial::{
    serial_write_bytes, serial_write_hex_u32_minimal, serial_write_hex_u64_full,
};

#[repr(C)]
pub struct CpuFrame {
    rax: u64,
    rbx: u64,
    rcx: u64,
    rdx: u64,
    rsi: u64,
    rdi: u64,
    rbp: u64,
    r8: u64,
    r9: u64,
    r10: u64,
    r11: u64,
    r12: u64,
    r13: u64,
    r14: u64,
    r15: u64,
    vector: u64,
    error_code: u64,
    rip: u64,
    cs: u64,
    rflags: u64,
}

const _: () = assert!(core::mem::size_of::<CpuFrame>() == 20 * 8);

#[unsafe(no_mangle)]
pub extern "C" fn exception_handler(frame: *const CpuFrame) -> ! {

    let frame = unsafe { &*frame };

    serial_write_bytes(b"OKernel: EXCEPTION\r\n");
    serial_write_bytes(b"vec=");
    serial_write_hex_u64_full(frame.vector);
    serial_write_bytes(b" err=");
    serial_write_hex_u64_full(frame.error_code);
    serial_write_bytes(b"\r\n");
    serial_write_bytes(b"rip=");
    serial_write_hex_u64_full(frame.rip);
    serial_write_bytes(b" cs=");
    serial_write_hex_u64_full(frame.cs);
    serial_write_bytes(b" flags=");
    serial_write_hex_u64_full(frame.rflags);
    serial_write_bytes(b"\r\n");

    let frame_addr = frame as *const CpuFrame as u64;
    let (stack_base, stack_size) = df_stack_bounds();
    let frame_end = frame_addr.checked_add(core::mem::size_of::<CpuFrame>() as u64);
    let in_ist = match frame_end {
        Some(end) => frame_addr >= stack_base && end <= stack_base + stack_size,
        None => false,
    };
    serial_write_bytes(b"ist=");
    serial_write_hex_u32_minimal(in_ist as u32);
    serial_write_bytes(b"\r\n");

    let cr2: u64;

    unsafe {
        asm!("mov {}, cr2", out(reg) cr2, options(nostack, preserves_flags));
    }
    serial_write_bytes(b"cr2=");
    serial_write_hex_u64_full(cr2);
    serial_write_bytes(b"\r\n");

    unsafe {
        asm!("cli", options(nomem, nostack));
    }
    loop {
        core::hint::spin_loop();
    }
}