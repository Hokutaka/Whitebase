bits 64
default rel

section .text

global whitebase_gnu_asm_add_f32_scalar
global whitebase_gnu_asm_add_f64_array_scalar
global whitebase_gnu_asm_add_f64_scalar

; Windows x64 ABI
; rcx = lhs, rdx = rhs, r8 = output, r9 = length
whitebase_gnu_asm_add_f32_scalar:
    xor r10, r10

.loop:
    cmp r10, r9
    jae .done

    movss xmm0, [rcx + r10 * 4]
    addss xmm0, [rdx + r10 * 4]
    movss [r8 + r10 * 4], xmm0

    inc r10
    jmp .loop

.done:
    ret

; Windows x64 ABI
; rcx = lhs, rdx = rhs, r8 = output, r9 = length
whitebase_gnu_asm_add_f64_array_scalar:
    xor r10, r10

.loop:
    cmp r10, r9
    jae .done

    movsd xmm0, [rcx + r10 * 8]
    addsd xmm0, [rdx + r10 * 8]
    movsd [r8 + r10 * 8], xmm0

    inc r10
    jmp .loop

.done:
    ret

; Windows x64 ABI
; xmm0 = lhs, xmm1 = rhs, xmm0 = return value
whitebase_gnu_asm_add_f64_scalar:
    addsd xmm0, xmm1
    ret
