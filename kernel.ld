ENTRY(_start)

SECTIONS
{
    . = 0x00010000;

    .text ALIGN(16) :
    {
        KEEP(*(.text._start))
        *(.text .text.*)
    }

    .rodata ALIGN(16) :
    {
        *(.rodata .rodata.*)
    }

    .data ALIGN(16) :
    {
        *(.data .data.*)
    }

    .bss ALIGN(16) (NOLOAD) :
    {
        PROVIDE(_bss_start = .);
        *(COMMON)
        *(.bss .bss.*)
        PROVIDE(_bss_end = .);
    }

    /DISCARD/ :
    {
        *(.eh_frame .eh_frame.*)
        *(.comment)
    }
}