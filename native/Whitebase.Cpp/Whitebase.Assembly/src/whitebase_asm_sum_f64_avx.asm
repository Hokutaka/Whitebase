PUBLIC whitebase_asm_sum_f64_avx

.code

; double whitebase_asm_sum_f64_avx(
;     const double* input, ; RCX
;     size_t length        ; RDX
; );
; return: XMM0
; AVX availability is checked by the Rust adapter before this function is called.

whitebase_asm_sum_f64_avx PROC
    vxorpd ymm0, ymm0, ymm0
    xor rax, rax

    mov r8, rdx
    and r8, -4

    cmp rax, r8
    jae reduce_vector

avx_loop:
    vaddpd ymm0, ymm0, YMMWORD PTR [rcx + rax * 8]
    add rax, 4
    cmp rax, r8
    jb avx_loop

reduce_vector:
    vextractf128 xmm1, ymm0, 1
    vaddpd xmm0, xmm0, xmm1
    vhaddpd xmm0, xmm0, xmm0

    cmp rax, rdx
    jae done

tail_loop:
    vaddsd xmm0, xmm0, QWORD PTR [rcx + rax * 8]
    inc rax
    cmp rax, rdx
    jb tail_loop

done:
    vzeroupper
    ret
whitebase_asm_sum_f64_avx ENDP

END
