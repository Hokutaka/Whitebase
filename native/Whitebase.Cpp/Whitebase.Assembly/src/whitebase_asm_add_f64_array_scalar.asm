PUBLIC whitebase_asm_add_f64_array_scalar

.code

; void whitebase_asm_add_f64_array_scalar(
;     const double* lhs,  ; RCX
;     const double* rhs,  ; RDX
;     double* output,     ; R8
;     size_t length       ; R9
; );

whitebase_asm_add_f64_array_scalar PROC
    xor rax, rax

    test r9, r9
    jz done

loop_start:
    movsd xmm0, QWORD PTR [rcx + rax * 8]
    addsd xmm0, QWORD PTR [rdx + rax * 8]
    movsd QWORD PTR [r8 + rax * 8], xmm0

    inc rax
    cmp rax, r9
    jb loop_start

done:
    ret
whitebase_asm_add_f64_array_scalar ENDP

END
