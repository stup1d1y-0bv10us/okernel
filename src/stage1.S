.code16
.section .text
.global boot_entry

boot_entry:
    cli

    xorw %ax, %ax
    movw %ax, %ds
    movw %ax, %es
    movw %ax, %ss
    movw $0x7c00, %sp

    movb %dl, 0x0500

    sti

    movw $0x8000, %bx
    movb $0x02, %ah
    movb $0x04, %al
    movb $0x00, %ch
    movb $0x02, %cl
    movb $0x00, %dh
    movb 0x0500, %dl
    int $0x13

    jc disk_read_failed

    ljmp $0x0000, $0x8000

disk_read_failed:
    movb $0x0e, %ah
    movb $'E', %al
    int $0x10

disk_halt:
    cli
    hlt
    jmp disk_halt

.org 510
.word 0xaa55