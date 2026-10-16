fn write_cell(index: usize, character: u8, attribute: u8) {
    let value = ((attribute as u16) << 8) | character as u16;
    let address = 0xb8000usize + index * 2;

    unsafe {
        core::ptr::write_volatile(address as *mut u16, value);
    }
}

pub(crate) fn write_word(start: usize, word: &[u8; 4], attribute: u8) {
    for (offset, character) in word.iter().copied().enumerate() {
        write_cell(start + offset, character, attribute);
    }
}