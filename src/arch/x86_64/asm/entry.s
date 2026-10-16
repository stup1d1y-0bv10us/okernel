
.global exc_stub_0
 exc_stub_0: push 0
 push 0
 jmp exc_common

.global exc_stub_1
 exc_stub_1: push 0
 push 1
 jmp exc_common

.global exc_stub_2
 exc_stub_2: push 0
 push 2
 jmp exc_common

.global exc_stub_3
 exc_stub_3: push 0
 push 3
 jmp exc_common

.global exc_stub_4
 exc_stub_4: push 0
 push 4
 jmp exc_common

.global exc_stub_5
 exc_stub_5: push 0
 push 5
 jmp exc_common

.global exc_stub_6
 exc_stub_6: push 0
 push 6
 jmp exc_common

.global exc_stub_7
 exc_stub_7: push 0
 push 7
 jmp exc_common

.global exc_stub_8
 exc_stub_8: push 8
 jmp exc_common

.global exc_stub_9
 exc_stub_9: push 0
 push 9
 jmp exc_common

.global exc_stub_10
 exc_stub_10: push 10
 jmp exc_common

.global exc_stub_11
 exc_stub_11: push 11
 jmp exc_common

.global exc_stub_12
 exc_stub_12: push 12
 jmp exc_common

.global exc_stub_13
 exc_stub_13: push 13
 jmp exc_common

.global exc_stub_14
 exc_stub_14: push 14
 jmp exc_common

.global exc_stub_15
 exc_stub_15: push 0
 push 15
 jmp exc_common

.global exc_stub_16
 exc_stub_16: push 0
 push 16
 jmp exc_common

.global exc_stub_17
 exc_stub_17: push 17
 jmp exc_common

.global exc_stub_18
 exc_stub_18: push 0
 push 18
 jmp exc_common

.global exc_stub_19
 exc_stub_19: push 0
 push 19
 jmp exc_common

.global exc_stub_20
 exc_stub_20: push 0
 push 20
 jmp exc_common

.global exc_stub_21
 exc_stub_21: push 21
 jmp exc_common

.global exc_stub_22
 exc_stub_22: push 0
 push 22
 jmp exc_common

.global exc_stub_23
 exc_stub_23: push 0
 push 23
 jmp exc_common

.global exc_stub_24
 exc_stub_24: push 0
 push 24
 jmp exc_common

.global exc_stub_25
 exc_stub_25: push 0
 push 25
 jmp exc_common

.global exc_stub_26
 exc_stub_26: push 0
 push 26
 jmp exc_common

.global exc_stub_27
 exc_stub_27: push 0
 push 27
 jmp exc_common

.global exc_stub_28
 exc_stub_28: push 0
 push 28
 jmp exc_common

.global exc_stub_29
 exc_stub_29: push 29
 jmp exc_common

.global exc_stub_30
 exc_stub_30: push 30
 jmp exc_common

.global exc_stub_31
 exc_stub_31: push 0
 push 31
 jmp exc_common

exc_common:
push r15
 push r14
 push r13
 push r12
push r11
 push r10
 push r9
 push r8
push rbp
 push rdi
 push rsi
 push rdx
push rcx
 push rbx
 push rax
mov rdi, rsp
and rsp, -16
call exception_handler
ud2


.global irq0_entry
 irq0_entry: push 0
 push 32
 jmp irq_common

irq_common:
push r15
 push r14
 push r13
 push r12
push r11
 push r10
 push r9
 push r8
push rbp
 push rdi
 push rsi
 push rdx
push rcx
 push rbx
 push rax
mov rbx, rsp
cld
and rsp, -16
call irq0_handler
mov rsp, rbx
pop rax
 pop rbx
 pop rcx
 pop rdx
pop rsi
 pop rdi
 pop rbp
 pop r8
pop r9
 pop r10
 pop r11
 pop r12
pop r13
 pop r14
 pop r15
add rsp, 16
iretq