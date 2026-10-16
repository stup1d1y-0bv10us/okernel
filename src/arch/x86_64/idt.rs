use core::arch::asm;

use super::gdt::KERNEL_CS;

#[repr(C)]
#[derive(Clone, Copy)]
struct IdtEntry {
    offset_lo: u16,
    selector: u16,
    options: u16,
    offset_mid: u16,
    offset_hi: u32,
    reserved: u32,
}

const IDT_EMPTY: IdtEntry = IdtEntry {
    offset_lo: 0,
    selector: 0,
    options: 0,
    offset_mid: 0,
    offset_hi: 0,
    reserved: 0,
};

const IDT_OPTIONS: u16 = 0x8e00;

const IDT_COUNT: usize = 33;

const IRQ0_VECTOR: usize = 32;

static mut IDT: [IdtEntry; IDT_COUNT] = [IDT_EMPTY; IDT_COUNT];

unsafe extern "C" {
    fn exc_stub_0();
    fn exc_stub_1();
    fn exc_stub_2();
    fn exc_stub_3();
    fn exc_stub_4();
    fn exc_stub_5();
    fn exc_stub_6();
    fn exc_stub_7();
    fn exc_stub_8();
    fn exc_stub_9();
    fn exc_stub_10();
    fn exc_stub_11();
    fn exc_stub_12();
    fn exc_stub_13();
    fn exc_stub_14();
    fn exc_stub_15();
    fn exc_stub_16();
    fn exc_stub_17();
    fn exc_stub_18();
    fn exc_stub_19();
    fn exc_stub_20();
    fn exc_stub_21();
    fn exc_stub_22();
    fn exc_stub_23();
    fn exc_stub_24();
    fn exc_stub_25();
    fn exc_stub_26();
    fn exc_stub_27();
    fn exc_stub_28();
    fn exc_stub_29();
    fn exc_stub_30();
    fn exc_stub_31();
    fn irq0_entry();
}

fn stub_address(stub: unsafe extern "C" fn()) -> u64 {
    stub as u64
}

fn set_idt_entry(index: usize, handler: u64, ist: u8) {
    let entry = IdtEntry {
        offset_lo: (handler & 0xffff) as u16,
        selector: KERNEL_CS,
        options: IDT_OPTIONS | (ist as u16 & 7),
        offset_mid: ((handler >> 16) & 0xffff) as u16,
        offset_hi: (handler >> 32) as u32,
        reserved: 0,
    };

    unsafe {
        (*core::ptr::addr_of_mut!(IDT))[index] = entry;
    }
}

pub(crate) fn idt_init() {
    const STUBS: [unsafe extern "C" fn(); 32] = [
        exc_stub_0, exc_stub_1, exc_stub_2, exc_stub_3, exc_stub_4, exc_stub_5,
        exc_stub_6, exc_stub_7, exc_stub_8, exc_stub_9, exc_stub_10,
        exc_stub_11, exc_stub_12, exc_stub_13, exc_stub_14, exc_stub_15,
        exc_stub_16, exc_stub_17, exc_stub_18, exc_stub_19, exc_stub_20,
        exc_stub_21, exc_stub_22, exc_stub_23, exc_stub_24, exc_stub_25,
        exc_stub_26, exc_stub_27, exc_stub_28, exc_stub_29, exc_stub_30,
        exc_stub_31,
    ];
    for (index, stub) in STUBS.iter().copied().enumerate() {
        set_idt_entry(index, stub_address(stub), 0);
    }

    set_idt_entry(8, stub_address(exc_stub_8), 1);

    set_idt_entry(IRQ0_VECTOR, stub_address(irq0_entry), 0);

    let mut idtr = [0u8; 10];
    let limit = (IDT_COUNT * 16 - 1) as u16;
    idtr[0..2].copy_from_slice(&limit.to_le_bytes());
    let base = core::ptr::addr_of!(IDT) as u64;
    idtr[2..10].copy_from_slice(&base.to_le_bytes());

    unsafe {
        asm!("lidt [{}]", in(reg) idtr.as_ptr(), options(nostack, preserves_flags));
    }
}

#[cfg(df_test)]
pub(crate) fn poison_gp_gate_for_df_test() {

    unsafe {
        (*core::ptr::addr_of_mut!(IDT))[13].options &= !0x8000u16;
    }
}