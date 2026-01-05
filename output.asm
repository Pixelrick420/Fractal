BITS 64
section .text
global _start

_start:
    push rbp
    mov rbp, rsp
    sub rsp, 1024
    mov rax, 1
    mov [rbp-8], rax
    mov rax, 1
    mov [rbp-16], rax
    mov rax, 1
    mov [rbp-24], rax
for_start_0:
    mov rax, [rbp-24]
    push rax
    mov rax, 10
    mov rbx, rax
    pop rax
    cmp rax, rbx
    setl al
    movzx rax, al
    cmp rax, 0
    je for_end_0
    mov rax, [rbp-8]
    mov [rbp-32], rax
    mov rax, [rbp-16]
    mov [rbp-8], rax
    mov rax, [rbp-32]
    push rax
    mov rax, [rbp-16]
    mov rbx, rax
    pop rax
    add rax, rbx
    mov [rbp-16], rax
    mov rax, [rbp-24]
    push rax
    mov rax, 1
    mov rbx, rax
    pop rax
    add rax, rbx
    mov [rbp-24], rax
    jmp for_start_0
for_end_0:
    mov rax, [rbp-16]
    mov rdi, rax
    mov rax, 60
    syscall

    ; Default exit with code 0
    mov rax, 60
    xor rdi, rdi
    syscall
