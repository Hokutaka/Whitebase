PUBLIC whitebase_asm_sum_f64_scalar

.code

; double whitebase_asm_sum_f64_scalar(
;     const double* input, ; RCX
;     size_t length        ; RDX
; );
; return: XMM0

whitebase_asm_sum_f64_scalar PROC
    xorpd xmm0, xmm0
    xor rax, rax

    test rdx, rdx
    jz done

loop_start:
    addsd xmm0, QWORD PTR [rcx + rax * 8]
    inc rax
    cmp rax, rdx
    jb loop_start

done:
    ret
whitebase_asm_sum_f64_scalar ENDP

END
