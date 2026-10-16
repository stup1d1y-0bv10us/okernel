use core::arch::asm;

pub(crate) const KERNEL_CS: u16 = 0x18;

#[repr(align(16))]
#[allow(dead_code)]
struct AlignedDfStack([u8; 8192]);

static mut DF_STACK: AlignedDfStack = AlignedDfStack([0; 8192]);

const DF_STACK_SIZE: u64 = 8192;

#[repr(C, packed)]
struct Tss {
    reserved0: u32,
    rsp: [u64; 3],
    reserved1: u64,
    ist: [u64; 7],
    reserved2: u64,
    reserved3: u16,
    iomap_base: u16,
}

const _: () = assert!(core::mem::size_of::<Tss>() == 104);

static mut TSS: Tss = Tss {
    reserved0: 0,
    rsp: [0, 0, 0],
    reserved1: 0,
    ist: [0; 7],
    reserved2: 0,
    reserved3: 0,
    iomap_base: 0,
};

static mut KERNEL_GDT: [u64; 6] = [0; 6];

const TSS_SELECTOR: u16 = 0x20;

pub(crate) fn df_stack_bounds() -> (u64, u64) {
    (core::ptr::addr_of!(DF_STACK) as u64, DF_STACK_SIZE)
}

pub(crate) fn tss_init() {

    const CODE32: u64 = 0x00cf9a000000ffff;
    const DATA: u64 = 0x00cf92000000ffff;
    const CODE64: u64 = 0x00af9a000000ffff;

    unsafe {
        let tss = core::ptr::addr_of_mut!(TSS);

        core::ptr::addr_of_mut!((*tss).rsp).write_unaligned([0x7c00, 0, 0]);
        let mut ist = [0u64; 7];
        ist[0] = core::ptr::addr_of!(DF_STACK) as u64 + DF_STACK_SIZE;
        core::ptr::addr_of_mut!((*tss).ist).write_unaligned(ist);

        core::ptr::addr_of_mut!((*tss).iomap_base)
            .write_unaligned(core::mem::size_of::<Tss>() as u16);

        let tss_base = core::ptr::addr_of!(TSS) as u64;
        let tss_limit = core::mem::size_of::<Tss>() as u64 - 1;
        let tss_lo = (tss_limit & 0xffff)
            | ((tss_base & 0x00ff_ffff) << 16)
            | (0x89u64 << 40)
            | (((tss_base >> 24) & 0xff) << 56);
        let tss_hi = tss_base >> 32;

        let gdt = &mut *core::ptr::addr_of_mut!(KERNEL_GDT);
        gdt[0] = 0;
        gdt[1] = CODE32;
        gdt[2] = DATA;
        gdt[3] = CODE64;
        gdt[4] = tss_lo;
        gdt[5] = tss_hi;
    }

    let mut gdtr = [0u8; 10];
    gdtr[0..2].copy_from_slice(&47u16.to_le_bytes());
    let base = core::ptr::addr_of!(KERNEL_GDT) as u64;
    gdtr[2..10].copy_from_slice(&base.to_le_bytes());

    unsafe {
        asm!("lgdt [{}]", in(reg) gdtr.as_ptr(), options(nostack, preserves_flags));
        asm!(
            "mov ds, {0:x}",
            "mov es, {0:x}",
            "mov ss, {0:x}",
            in(reg) 0x10u16,
            options(nostack, preserves_flags)
        );
        asm!("ltr {0:x}", in(reg) TSS_SELECTOR, options(nostack, preserves_flags));
    }
}