use crate::arch::x86_64::io::{inb, outb};

const COM1: u16 = 0x3f8;

pub(crate) fn serial_init() {

    unsafe {
        outb(COM1 + 1, 0x00);
        outb(COM1 + 3, 0x80);
        outb(COM1,     0x03);
        outb(COM1 + 1, 0x00);
        outb(COM1 + 3, 0x03);
        outb(COM1 + 2, 0xc7);
        outb(COM1 + 4, 0x0b);
    }
}

pub(crate) fn serial_write_byte(byte: u8) -> bool {

    for _ in 0..100_000u32 {
        let status = unsafe { inb(COM1 + 5) };

        if status & 0x20 != 0 {
            unsafe {
                outb(COM1, byte);
            }

            return true;
        }
    }

    false
}

pub(crate) fn serial_write_bytes(bytes: &[u8]) {
    for &byte in bytes {
        if !serial_write_byte(byte) {
            break;
        }
    }
}

fn serial_write_hex_digit(nibble: u8) {
    let character = if nibble < 10 {
        b'0' + nibble
    } else {
        b'a' + (nibble - 10)
    };
    serial_write_bytes(&[character]);
}

pub(crate) fn serial_write_hex_u64_full(value: u64) {
    serial_write_bytes(b"0x");
    for shift in (0..16).rev() {
        serial_write_hex_digit(((value >> (shift * 4)) & 0xf) as u8);
    }
}

pub(crate) fn serial_write_hex_u32_full(value: u32) {
    serial_write_bytes(b"0x");
    for shift in (0..8).rev() {
        serial_write_hex_digit(((value >> (shift * 4)) & 0xf) as u8);
    }
}

pub(crate) fn serial_write_hex_u32_minimal(value: u32) {
    let mut started = false;
    for shift in (0..8).rev() {
        let nibble = ((value >> (shift * 4)) & 0xf) as u8;
        if nibble != 0 || shift == 0 {
            started = true;
        }
        if started {
            serial_write_hex_digit(nibble);
        }
    }
}