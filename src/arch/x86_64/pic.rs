use super::io::{io_wait, outb};

const PIC_MASTER_CMD: u16 = 0x20;
const PIC_MASTER_DATA: u16 = 0x21;
const PIC_SLAVE_CMD: u16 = 0xa0;
const PIC_SLAVE_DATA: u16 = 0xa1;

pub(crate) fn pic_init() {
    unsafe {
        outb(PIC_MASTER_DATA, 0xff);
        outb(PIC_SLAVE_DATA, 0xff);
        io_wait();
        outb(PIC_MASTER_CMD, 0x11);
        io_wait();
        outb(PIC_SLAVE_CMD, 0x11);
        io_wait();
        outb(PIC_MASTER_DATA, 0x20);
        io_wait();
        outb(PIC_SLAVE_DATA, 0x28);
        io_wait();
        outb(PIC_MASTER_DATA, 0x04);
        io_wait();
        outb(PIC_SLAVE_DATA, 0x02);
        io_wait();
        outb(PIC_MASTER_DATA, 0x01);
        io_wait();
        outb(PIC_SLAVE_DATA, 0x01);
        io_wait();
        outb(PIC_MASTER_DATA, 0xfe);
        outb(PIC_SLAVE_DATA, 0xff);
    }
}

pub(crate) fn eoi_master() {

    unsafe {
        outb(PIC_MASTER_CMD, 0x20);
    }
}