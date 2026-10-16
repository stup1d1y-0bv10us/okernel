.code16
.section .text
.global stage2_entry

stage2_entry:
    cli

    xorw %ax, %ax
    movw %ax, %ds
    movw %ax, %es
    movw %ax, %ss
    movw $0x7c00, %sp

    sti

    movb $0x0e, %ah
    movb $'S', %al
    int $0x10

    movb $0x0e, %ah
    movb $'2', %al
    int $0x10


    movw $0, 0x0502
    movw $0x5000, %di
    xorl %ebx, %ebx

e820_next:

    movl $0xe820, %eax
    movl $0x534d4150, %edx
    movl $24, %ecx


    movl $1, 20(%di)

    int $0x15

    jc e820_failed

    cmpl $0x534d4150, %eax
    jne e820_failed


    cmpl $20, %ecx
    je e820_entry_valid

    cmpl $24, %ecx
    jne e820_failed

e820_entry_valid:
    incw 0x0502


    addw $24, %di


    testl %ebx, %ebx
    jz e820_success


    cmpw $32, 0x0502
    jae e820_failed

    jmp e820_next

e820_success:

    movw 0x0502, %cx
    cmpw $1, %cx
    jb e820_failed

    cmpw $32, %cx
    ja e820_failed


    movw $0x5000, %si

e820_validate_entry:

    movl 8(%si), %eax
    orl 12(%si), %eax
    jz e820_failed


    cmpl $0, 16(%si)
    je e820_failed


    movl 0(%si), %eax
    movl 4(%si), %edx

    addl 8(%si), %eax
    adcl 12(%si), %edx

    jc e820_failed


    addw $24, %si
    loop e820_validate_entry


    movl $0x49424b4f, 0x0600
    movl $1,          0x0604
    movl $32,         0x0608

    movw 0x0502, %ax
    movzwl %ax, %eax
    movl %eax, 0x060c

    movl $24,         0x0610
    movl $0x5000,     0x0614

    movzbl 0x0500, %eax
    movl %eax, 0x0618

    movl $0,          0x061c


    cmpl $0x49424b4f, 0x0600
    jne e820_failed

    cmpl $1, 0x0604
    jne e820_failed

    cmpl $32, 0x0608
    jne e820_failed

    movw 0x0502, %ax
    movzwl %ax, %eax
    cmpl %eax, 0x060c
    jne e820_failed

    cmpl $24, 0x0610
    jne e820_failed

    cmpl $0x5000, 0x0614
    jne e820_failed

    movzbl 0x0500, %eax
    cmpl %eax, 0x0618
    jne e820_failed

    cmpl $0, 0x061c
    jne e820_failed


    movb $0x0e, %ah
    movb $'M', %al
    int $0x10

    movb $0x0e, %ah
    movb $'V', %al
    int $0x10

    movb $0x0e, %ah
    movb $'R', %al
    int $0x10

    movb $0x0e, %ah
    movb $'B', %al
    int $0x10


#ifdef CHS_FORCE_TEST

    jmp chs_fallback
#else
    movb $0x41, %ah
    movw $0x55aa, %bx
    movb 0x0500, %dl
    int $0x13
    jc chs_fallback
    cmpw $0xaa55, %bx
    jne chs_fallback
    testb $0x01, %cl
    jz chs_fallback
#endif


    movw $0, (dap_lba_hi + 0x8000)

    xorw %cx, %cx
edd_chunk:
    pushw %cx
    movw $8, (dap_count + 0x8000)
    movw %cx, %ax
    shlw $12, %ax
    movw %ax, (dap_offset + 0x8000)
    shlw $3, %cx
    addw $5, %cx
    movw %cx, (dap_lba_lo + 0x8000)

    movb $0x42, %ah
    movb 0x0500, %dl
    movw $(dap + 0x8000), %si
    int $0x13
    jc stage2_disk_read_failed

    popw %cx
    incw %cx
    cmpw $8, %cx
    jb edd_chunk

    jmp kernel_load_done

chs_fallback:

    movb $0x08, %ah
    movb 0x0500, %dl
    int $0x13
    jc stage2_disk_read_failed
    andb $0x3f, %cl
    jz stage2_disk_read_failed
    movb %cl, (chs_spt + 0x8000)
    movb %dh, %al
    incw %ax
    movw %ax, (chs_heads + 0x8000)

    movw $5, (chs_lba + 0x8000)
    movw $0, (chs_dest + 0x8000)
    movb $64, (chs_remaining + 0x8000)

chs_chunk:
    movb (chs_spt + 0x8000), %bl
    movw (chs_lba + 0x8000), %ax
    divb %bl
    movb %ah, %bh
    movb %bl, %al
    subb %bh, %al
    movb (chs_remaining + 0x8000), %ah
    cmpb %ah, %al
    jbe chs_have_count
    movb %ah, %al
chs_have_count:
    movb %al, (chs_count + 0x8000)

    movw (chs_lba + 0x8000), %ax
    divb (chs_spt + 0x8000)
    xorb %ah, %ah
    xorw %dx, %dx
    divw (chs_heads + 0x8000)
    movb %al, %ch
    movb %ah, %dh
    movb %bh, %al
    incb %al
    movb %al, %cl

    movw $0x1000, %ax
    movw %ax, %es
    movw (chs_dest + 0x8000), %bx
    movb (chs_count + 0x8000), %al
    movb $0x02, %ah
    movb 0x0500, %dl
    int $0x13
    jc stage2_disk_read_failed

    xorw %cx, %cx
    movb (chs_count + 0x8000), %cl
    movw %cx, %si
    addw %cx, (chs_lba + 0x8000)
    shlw $9, %cx
    addw %cx, (chs_dest + 0x8000)
    movb (chs_remaining + 0x8000), %al
    xorb %ah, %ah
    subw %si, %ax
    movb %al, (chs_remaining + 0x8000)
    testb %al, %al
    jnz chs_chunk

kernel_load_done:


    movw $0x1000, %ax
    movw %ax, %ds
    xorw %si, %si

    cmpl $0x41574155, (%si)
    jne stage2_disk_read_failed

    cmpl $0x41554156, 4(%si)
    jne stage2_disk_read_failed


    cmpl $0x34364b4f, 0x7ff8
    jne stage2_disk_read_failed

    cmpl $0x21434553, 0x7ffc
    jne stage2_disk_read_failed


    xorw %ax, %ax
    movw %ax, %ds


    movb $0x0e, %ah
    movb $'I', %al
    int $0x10


    cli

    lgdt (gdt_descriptor + 0x8000)

    mov %cr0, %eax
    orl $0x1, %eax
    mov %eax, %cr0

    ljmp $0x08, $(protected_mode_entry + 0x8000)

stage2_disk_read_failed:
    movb $0x0e, %ah
    movb $'F', %al
    int $0x10

    jmp stage2_halt

e820_failed:
    movb $0x0e, %ah
    movb $'E', %al
    int $0x10

stage2_halt:
    cli
    hlt
    jmp stage2_halt

.code32
protected_mode_entry:

    movw $0x10, %ax
    movw %ax, %ds
    movw %ax, %es
    movw %ax, %ss


    movl $0x7c00, %esp


    movl $0x000b8000, %edi
    movb $'P', %al
    movb $0x0f, %ah
    movw %ax, (%edi)


    xorl %eax, %eax
    movl $0x1000, %edi
    movl $3072, %ecx
    cld
    rep stosl


    movl $0x2003, 0x1000


    movl $0x3003, 0x2000


    movl $0x83, 0x3000


    cmpl $0x2003, 0x1000
    jne stage2_pm_table_fail

    cmpl $0x3003, 0x2000
    jne stage2_pm_table_fail

    cmpl $0x83, 0x3000
    jne stage2_pm_table_fail


    movl $0x000b8002, %edi
    movb $'T', %al
    movb $0x0f, %ah
    movw %ax, (%edi)


    movl %cr4, %eax
    orl $0x20, %eax
    movl %eax, %cr4


    movl $0x1000, %eax
    movl %eax, %cr3


    movl $0xC0000080, %ecx
    rdmsr
    orl $0x100, %eax
    wrmsr


    movl %cr0, %eax
    orl $0x80000000, %eax
    movl %eax, %cr0


    ljmp $0x18, $(long_mode_entry + 0x8000)

stage2_pm_table_fail:
    movl $0x000b8002, %edi
    movb $'E', %al
    movb $0x0c, %ah
    movw %ax, (%edi)

stage2_pm_halt:
    hlt
    jmp stage2_pm_halt

.align 8
gdt:
    .quad 0x0000000000000000
    .quad 0x00cf9a000000ffff
    .quad 0x00cf92000000ffff
    .quad 0x00af9a000000ffff

gdt_end:

gdt_descriptor:
    .word gdt_end - gdt - 1
    .long gdt + 0x8000


dap:
    .byte 0x10
    .byte 0x00
    .word 0x0008
    .word 0x0000
    .word 0x1000
    .long 0x00000000
    .long 0x00000000
    .set dap_count, dap + 2
    .set dap_offset, dap + 4
    .set dap_lba_lo, dap + 8
    .set dap_lba_hi, dap + 12
chs_spt:
    .byte 0x00
chs_heads:
    .word 0x0000
chs_lba:
    .word 0x0000
chs_dest:
    .word 0x0000
chs_count:
    .byte 0x00
chs_remaining:
    .byte 0x00

.code64
long_mode_entry:

    movq $0x7c00, %rsp


    movq $0x000b8004, %rdi
    movb $'6', %al
    movb $0x0f, %ah
    movw %ax, (%rdi)


    movq $0x000b8006, %rdi
    movb $'4', %al
    movb $0x0f, %ah
    movw %ax, (%rdi)


    movq $0x12345678, %rax
    shlq $32, %rax
    movl $0x9abcdef0, %ecx
    orq %rcx, %rax


    movq %rax, %rdx


    pushq %rax
    xorq %rax, %rax
    popq %rax


    cmpq %rdx, %rax
    jne long_mode_stack_fail


    cmpq $0x7c00, %rsp
    jne long_mode_stack_fail


    movq $0x000b8008, %rdi
    movb $'S', %al
    movb $0x0f, %ah
    movw %ax, (%rdi)


    movq $0x0600, %rdi

    cmpl $0x49424b4f, 0(%rdi)
    jne long_mode_bootinfo_fail

    cmpl $1, 4(%rdi)
    jne long_mode_bootinfo_fail

    cmpl $32, 8(%rdi)
    jne long_mode_bootinfo_fail


    cmpl $1, 12(%rdi)
    jb long_mode_bootinfo_fail

    cmpl $32, 12(%rdi)
    ja long_mode_bootinfo_fail


    cmpl $24, 16(%rdi)
    jne long_mode_bootinfo_fail

    cmpl $0x5000, 20(%rdi)
    jne long_mode_bootinfo_fail


    movq $0x0500, %rsi
    movzbl (%rsi), %eax
    cmpl %eax, 24(%rdi)
    jne long_mode_bootinfo_fail


    cmpl $0, 28(%rdi)
    jne long_mode_bootinfo_fail


    movq $0x000b800a, %rsi
    movb $'B', %al
    movb $0x0f, %ah
    movw %ax, (%rsi)


    movq $0x0600, %rdi


    movq $0x7c00, %rsp
    subq $8, %rsp


    xorq %rbp, %rbp
    cld


    movq $0x0600, %rdi


    movl $0x00010000, %eax
    jmp *%rax

long_mode_bootinfo_fail:
    movq $0x000b800a, %rsi
    movb $'E', %al
    movb $0x0c, %ah
    movw %ax, (%rsi)
    jmp long_mode_halt

long_mode_stack_fail:

    movq $0x000b8008, %rdi
    movb $'F', %al
    movb $0x0c, %ah
    movw %ax, (%rdi)

    jmp long_mode_halt

long_mode_halt:
    hlt
    jmp long_mode_halt