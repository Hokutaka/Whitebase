PUBLIC whitebase_asm_add_f32_scalar

.code

; void whitebase_asm_add_f32_scalar(
;     const float* lhs,   ; RCX
;     const float* rhs,   ; RDX
;     float* output,      ; R8
;     size_t length       ; R9
; );

whitebase_asm_add_f32_scalar PROC
    xor rax, rax

    test r9, r9
    jz done

loop_start:
    movss xmm0, DWORD PTR [rcx + rax * 4]
    addss xmm0, DWORD PTR [rdx + rax * 4]
    movss DWORD PTR [r8 + rax * 4], xmm0

    inc rax
    cmp rax, r9
    jb loop_start

done:
    ret
whitebase_asm_add_f32_scalar ENDP

END