bits 64
default rel

section .text

global whitebase_asm_add_f32_scalar
global whitebase_asm_add_f64_array_scalar
global whitebase_asm_add_f64_scalar

; System V AMD64 ABI
; rdi = lhs, rsi = rhs, rdx = output, rcx = length
whitebase_asm_add_f32_scalar:
    xor r8, r8

.loop:
    cmp r8, rcx
    jae .done

    movss xmm0, [rdi + r8 * 4]
    addss xmm0, [rsi + r8 * 4]
    movss [rdx + r8 * 4], xmm0

    inc r8
    jmp .loop

.done:
    ret

; System V AMD64 ABI
; rdi = lhs, rsi = rhs, rdx = output, rcx = length
whitebase_asm_add_f64_array_scalar:
    xor r8, r8

.loop:
    cmp r8, rcx
    jae .done

    movsd xmm0, [rdi + r8 * 8]
    addsd xmm0, [rsi + r8 * 8]
    movsd [rdx + r8 * 8], xmm0

    inc r8
    jmp .loop

.done:
    ret

; System V AMD64 ABI
; xmm0 = lhs, xmm1 = rhs, xmm0 = return value
whitebase_asm_add_f64_scalar:
    addsd xmm0, xmm1
    ret

section .note.GNU-stack noalloc noexec nowrite progbits
