bits 64
default rel

section .text

global whitebase_asm_add_f32_avx
global whitebase_asm_add_f64_array_avx

; System V AMD64 ABI
; rdi = lhs, rsi = rhs, rdx = output, rcx = length
whitebase_asm_add_f32_avx:
    xor r8, r8

    mov r9, rcx
    and r9, -8

.avx_loop:
    cmp r8, r9
    jae .scalar_tail

    vmovups ymm0, [rdi + r8 * 4]
    vaddps ymm0, ymm0, [rsi + r8 * 4]
    vmovups [rdx + r8 * 4], ymm0

    add r8, 8
    jmp .avx_loop

.scalar_tail:
    cmp r8, rcx
    jae .done

.tail_loop:
    vmovss xmm0, [rdi + r8 * 4]
    vaddss xmm0, xmm0, [rsi + r8 * 4]
    vmovss [rdx + r8 * 4], xmm0

    inc r8
    cmp r8, rcx
    jb .tail_loop

.done:
    vzeroupper
    ret

; System V AMD64 ABI
; rdi = lhs, rsi = rhs, rdx = output, rcx = length
whitebase_asm_add_f64_array_avx:
    xor r8, r8

    mov r9, rcx
    and r9, -4

.avx_loop:
    cmp r8, r9
    jae .scalar_tail

    vmovupd ymm0, [rdi + r8 * 8]
    vaddpd ymm0, ymm0, [rsi + r8 * 8]
    vmovupd [rdx + r8 * 8], ymm0

    add r8, 4
    jmp .avx_loop

.scalar_tail:
    cmp r8, rcx
    jae .done

.tail_loop:
    vmovsd xmm0, [rdi + r8 * 8]
    vaddsd xmm0, xmm0, [rsi + r8 * 8]
    vmovsd [rdx + r8 * 8], xmm0

    inc r8
    cmp r8, rcx
    jb .tail_loop

.done:
    vzeroupper
    ret

section .note.GNU-stack noalloc noexec nowrite progbits
