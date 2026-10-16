#![no_std]
#![no_main]

use core::arch::asm;
use core::panic::PanicInfo;

mod arch;
mod boot;
mod drivers;
mod memory;

use crate::boot::e820::{e820_diagnostics, frame_planning};
use crate::boot::okbi::{self, BootInfo};
use memory::bootstrap::{allocator_init, allocator_smoke_test};

unsafe extern "C" {
    static mut _bss_start: u8;
    static mut _bss_end: u8;
}

use arch::x86_64::{gdt, idt, timer};
#[cfg(align_test)]
use arch::x86_64::interrupts;
use drivers::serial::{
    serial_init, serial_write_bytes, serial_write_hex_u64_full,
};
use drivers::vga::write_word;

fn zero_bss() {
    unsafe {
        let start = core::ptr::addr_of_mut!(_bss_start);
        let end = core::ptr::addr_of_mut!(_bss_end);
        let mut current = start;
        while current < end {
            core::ptr::write(current, 0u8);
            current = current.add(1);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn _start(boot_info: *const BootInfo) -> ! {

    zero_bss();

    serial_init();
    serial_write_bytes(b"OKernel: Rust entry\r\n");

    let valid_info = okbi::validate(boot_info);

    if let Some(info) = valid_info {
        serial_write_bytes(b"OKernel: OKBI valid\r\n");
        write_word(6, b"RUST", 0x0f);

        gdt::tss_init();
        serial_write_bytes(b"OKernel: TSS ready\r\n");

        idt::idt_init();
        serial_write_bytes(b"OKernel: IDT ready\r\n");

        if e820_diagnostics(info) && frame_planning(info) {
            if allocator_init(info) {
                allocator_smoke_test();
            } else {
                serial_write_bytes(b"OKernel: allocator init FAILED\r\n");
            }
        }

        #[cfg(ud2_test)]
        {
            serial_write_bytes(b"OKernel: UD2 test\r\n");
            unsafe {
                asm!("ud2", options(noreturn));
            }
        }

        #[cfg(df_test)]
        {
            serial_write_bytes(b"OKernel: double-fault test begin\r\n");
            idt::poison_gp_gate_for_df_test();
            unsafe {
                asm!(
                    "mov eax, 0x38",
                    "mov ds, ax",
                    out("eax") _,
                    options(nostack, nomem, preserves_flags)
                );
            }
        }

        timer::timer_init();
        serial_write_bytes(b"OKernel: timer ready\r\n");
        serial_write_bytes(b"OKernel: timer wait begin\r\n");

        let wait_start = timer::ticks();
        loop {
            let observed = timer::ticks();
            if observed.wrapping_sub(wait_start) >= timer::TIMER_TEST_TICKS {
                break;
            }
            unsafe {
                asm!("sti", "hlt", options(nomem, nostack));
            }
        }

        unsafe {
            asm!("cli", options(nomem, nostack));
        }
        let total = timer::ticks();
        serial_write_bytes(b"OKernel: timer ticks=");
        serial_write_hex_u64_full(total);
        serial_write_bytes(b"\r\n");
        serial_write_bytes(b"OKernel: timer valid\r\n");

        #[cfg(align_test)]
        {
            serial_write_bytes(b"OKernel: align test begin\r\n");
            let pre_rsp: u64;
            unsafe {
                asm!("mov {}, rsp", out(reg) pre_rsp, options(nostack, preserves_flags));
            }
            serial_write_bytes(b"OKernel: align test rsp=");
            serial_write_hex_u64_full(pre_rsp);
            serial_write_bytes(b"\r\n");

            if pre_rsp.wrapping_sub(8) % 16 != 8 {
                serial_write_bytes(b"OKernel: align test INVALID setup\r\n");
            } else {
                let ticks_before = timer::ticks();
                unsafe {
                    interrupts::align_test_wait();
                }
                let post_rsp: u64;
                unsafe {
                    asm!("mov {}, rsp", out(reg) post_rsp, options(nostack, preserves_flags));
                }
                let ticks_after = timer::ticks();
                serial_write_bytes(b"OKernel: align test returned rsp=");
                serial_write_hex_u64_full(post_rsp);
                serial_write_bytes(b" ticks=");
                serial_write_hex_u64_full(ticks_after);
                serial_write_bytes(b"\r\n");
                if post_rsp != pre_rsp {
                    serial_write_bytes(b"OKernel: align test FAIL rsp\r\n");
                } else if ticks_after <= ticks_before {
                    serial_write_bytes(b"OKernel: align test FAIL ticks\r\n");
                } else {
                    serial_write_bytes(b"OKernel: align test ok\r\n");
                }
            }
        }

        loop {
            unsafe {
                asm!("sti", "hlt", options(nomem, nostack));
            }
        }
    } else {
        serial_write_bytes(b"OKernel: OKBI INVALID\r\n");
        write_word(6, b"FAIL", 0x0c);
    }

    loop {
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_init();
    serial_write_bytes(b"OKernel: PANIC\r\n");

    loop {
        core::hint::spin_loop();
    }
}