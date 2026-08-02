bits 64
default rel

%define CPUID_OSXSAVE_AVX_MASK 0x18000000
%define XCR0_XMM_YMM_MASK     0x6

section .text

global whitebase_asm_add_f32_avx
global whitebase_asm_add_f64_array_avx

; 内部用AVX可用性チェック
;
; 戻り値:
;   eax = 1: CPUとOSがAVXに対応
;   eax = 0: AVXを利用不可
;
; CPUIDはeax, ebx, ecx, edxを破壊する。
; System V AMD64 ABIではrbxがcallee-savedなので保存する。
whitebase_asm_is_avx_available:
    push rbx

    ; CPUID leaf 1が存在するか確認
    xor eax, eax
    cpuid

    cmp eax, 1
    jb .unavailable

    ; CPUID.1:ECX
    ; bit 27 = OSXSAVE
    ; bit 28 = AVX
    mov eax, 1
    cpuid

    mov eax, ecx
    and eax, CPUID_OSXSAVE_AVX_MASK
    cmp eax, CPUID_OSXSAVE_AVX_MASK
    jne .unavailable

    ; XCR0:
    ; bit 1 = XMM状態をOSが保存
    ; bit 2 = YMM状態をOSが保存
    xor ecx, ecx
    xgetbv

    and eax, XCR0_XMM_YMM_MASK
    cmp eax, XCR0_XMM_YMM_MASK
    jne .unavailable

    mov eax, 1
    pop rbx
    ret

.unavailable:
    xor eax, eax
    pop rbx
    ret


; System V AMD64 ABI
; rdi = lhs
; rsi = rhs
; rdx = output
; rcx = length
;
; 戻り値:
; eax = 1: AVX処理を実行
; eax = 0: AVX利用不可。outputは変更しない
whitebase_asm_add_f32_avx:
    ; 可用性チェックのcallをまたぐため、引数を保存
    ;
    ; 関数入口ではrsp % 16 == 8。
    ; 4回pushした後も8なので、さらに8引いて
    ; call直前を16バイト境界に合わせる。
    push rdi
    push rsi
    push rdx
    push rcx
    sub rsp, 8

    call whitebase_asm_is_avx_available

    add rsp, 8
    pop rcx
    pop rdx
    pop rsi
    pop rdi

    test eax, eax
    jz .unavailable

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
    mov eax, 1
    ret

.unavailable:
    xor eax, eax
    ret


; System V AMD64 ABI
; rdi = lhs
; rsi = rhs
; rdx = output
; rcx = length
;
; 戻り値:
; eax = 1: AVX処理を実行
; eax = 0: AVX利用不可。outputは変更しない
whitebase_asm_add_f64_array_avx:
    push rdi
    push rsi
    push rdx
    push rcx
    sub rsp, 8

    call whitebase_asm_is_avx_available

    add rsp, 8
    pop rcx
    pop rdx
    pop rsi
    pop rdi

    test eax, eax
    jz .unavailable

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
    mov eax, 1
    ret

.unavailable:
    xor eax, eax
    ret


section .note.GNU-stack noalloc noexec nowrite progbits
