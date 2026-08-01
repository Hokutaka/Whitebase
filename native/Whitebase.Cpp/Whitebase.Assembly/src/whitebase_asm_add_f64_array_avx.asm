PUBLIC whitebase_asm_add_f64_array_avx

.code

; void whitebase_asm_add_f64_array_avx(
;     const double* lhs,  ; RCX
;     const double* rhs,  ; RDX
;     double* output,     ; R8
;     size_t length       ; R9
; );

whitebase_asm_add_f64_array_avx PROC
    xor rax, rax

    ; AVXで処理できる4要素単位の長さを求める
    mov r10, r9
    and r10, -4

    cmp rax, r10
    jae scalar_tail

avx_loop:
    vmovupd ymm0, YMMWORD PTR [rcx + rax * 8]
    vaddpd ymm0, ymm0, YMMWORD PTR [rdx + rax * 8]
    vmovupd YMMWORD PTR [r8 + rax * 8], ymm0

    add rax, 4
    cmp rax, r10
    jb avx_loop

scalar_tail:
    cmp rax, r9
    jae done

tail_loop:
    vmovsd xmm0, QWORD PTR [rcx + rax * 8]
    vaddsd xmm0, xmm0, QWORD PTR [rdx + rax * 8]
    vmovsd QWORD PTR [r8 + rax * 8], xmm0

    inc rax
    cmp rax, r9
    jb tail_loop

done:
    vzeroupper
    ret
whitebase_asm_add_f64_array_avx ENDP

END
