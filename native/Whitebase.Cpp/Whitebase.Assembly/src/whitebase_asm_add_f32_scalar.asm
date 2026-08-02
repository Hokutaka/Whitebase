PUBLIC whitebase_asm_add_f32_scalar

.code

; void whitebase_asm_add_f32_scalar(
;     const float* lhs,   ; RCX
;     const float* rhs,   ; RDX
;     float* output,      ; R8
;     size_t length       ; R9
; );

whitebase_asm_add_f32_scalar PROC
    test r9, r9
    jz scalar_done

    cmp r9, 4
    jb scalar_tail

    ALIGN 16
scalar_loop4:
    movd xmm0, DWORD PTR [rcx]
    movd xmm1, DWORD PTR [rcx + 4]
    movd xmm2, DWORD PTR [rcx + 8]
    movd xmm3, DWORD PTR [rcx + 12]

    addss xmm0, DWORD PTR [rdx]
    addss xmm1, DWORD PTR [rdx + 4]
    addss xmm2, DWORD PTR [rdx + 8]
    addss xmm3, DWORD PTR [rdx + 12]

    movss DWORD PTR [r8],      xmm0
    movss DWORD PTR [r8 + 4],  xmm1
    movss DWORD PTR [r8 + 8],  xmm2
    movss DWORD PTR [r8 + 12], xmm3

    add rcx, 16
    add rdx, 16
    add r8, 16
    sub r9, 4

    cmp r9, 4
    jae scalar_loop4

scalar_tail:
    test r9, r9
    jz scalar_done

scalar_loop1:
    movd xmm0, DWORD PTR [rcx]
    addss xmm0, DWORD PTR [rdx]
    movss DWORD PTR [r8], xmm0

    add rcx, 4
    add rdx, 4
    add r8, 4
    dec r9
    jnz scalar_loop1

scalar_done:
    ret
whitebase_asm_add_f32_scalar ENDP

END
