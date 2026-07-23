PUBLIC whitebase_asm_add_f32_avx

.code

; void whitebase_asm_add_f32_avx(
;     const float* lhs,   ; RCX
;     const float* rhs,   ; RDX
;     float* output,      ; R8
;     size_t length       ; R9
; );

whitebase_asm_add_f32_avx PROC
    xor rax, rax

    ; AVXで処理できる8要素単位の長さを求める
    mov r10, r9
    and r10, -8

    cmp rax, r10
    jae scalar_tail

avx_loop:
    vmovups ymm0, YMMWORD PTR [rcx + rax * 4]
    vaddps ymm0, ymm0, YMMWORD PTR [rdx + rax * 4]
    vmovups YMMWORD PTR [r8 + rax * 4], ymm0

    add rax, 8
    cmp rax, r10
    jb avx_loop

scalar_tail:
    cmp rax, r9
    jae done

tail_loop:
    vmovss xmm0, DWORD PTR [rcx + rax * 4]
    vaddss xmm0, xmm0, DWORD PTR [rdx + rax * 4]
    vmovss DWORD PTR [r8 + rax * 4], xmm0

    inc rax
    cmp rax, r9
    jb tail_loop

done:
    vzeroupper
    ret
whitebase_asm_add_f32_avx ENDP

END