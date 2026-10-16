core::arch::global_asm!(include_str!("asm/entry.s"));

unsafe extern "C" {
    #[cfg(align_test)]
    pub(crate) fn align_test_wait();
}

#[cfg(align_test)]
core::arch::global_asm!(include_str!("asm/align_test.s"));