bits 64
default rel

%define CPUID_OSXSAVE_AVX_MASK 0x18000000
%define XCR0_XMM_YMM_MASK     0x6
%define CALL_FRAME_SIZE       72
%define AVX_CACHE_UNKNOWN     -1

section .data
align 4
whitebase_gnu_asm_avx_available_cache:
    dd AVX_CACHE_UNKNOWN

section .text

global whitebase_gnu_asm_add_f32_avx
global whitebase_gnu_asm_add_f64_array_avx
global whitebase_gnu_asm_sum_f64_avx

; Internal AVX availability check.
;
; Return value:
;   eax = 1: CPU and OS support AVX
;   eax = 0: AVX is unavailable
;
; CPUID destroys eax, ebx, ecx and edx. RBX is nonvolatile in the
; Windows x64 ABI, so preserve it here.
whitebase_gnu_asm_is_avx_available:
    mov eax, [rel whitebase_gnu_asm_avx_available_cache]
    test eax, eax
    jns .cached

    push rbx

    xor eax, eax
    cpuid

    cmp eax, 1
    jb .unavailable

    mov eax, 1
    cpuid

    mov eax, ecx
    and eax, CPUID_OSXSAVE_AVX_MASK
    cmp eax, CPUID_OSXSAVE_AVX_MASK
    jne .unavailable

    xor ecx, ecx
    xgetbv

    and eax, XCR0_XMM_YMM_MASK
    cmp eax, XCR0_XMM_YMM_MASK
    jne .unavailable

    mov eax, 1
    mov [rel whitebase_gnu_asm_avx_available_cache], eax
    pop rbx
    ret

.unavailable:
    xor eax, eax
    mov [rel whitebase_gnu_asm_avx_available_cache], eax
    pop rbx
    ret

.cached:
    ret

; Windows x64 ABI
; rcx = lhs, rdx = rhs, r8 = output, r9 = length
;
; Return value:
; eax = 1: AVX operation executed
; eax = 0: AVX unavailable; output is not modified
whitebase_gnu_asm_add_f32_avx:
    ; Reserve 32 bytes of shadow space, save four arguments and align RSP
    ; to a 16-byte boundary before calling the availability helper.
    sub rsp, CALL_FRAME_SIZE
    mov [rsp + 32], rcx
    mov [rsp + 40], rdx
    mov [rsp + 48], r8
    mov [rsp + 56], r9

    call whitebase_gnu_asm_is_avx_available

    mov rcx, [rsp + 32]
    mov rdx, [rsp + 40]
    mov r8, [rsp + 48]
    mov r9, [rsp + 56]
    add rsp, CALL_FRAME_SIZE

    test eax, eax
    jz .unavailable

    xor r10, r10
    mov r11, r9
    and r11, -8

.avx_loop:
    cmp r10, r11
    jae .scalar_tail

    vmovups ymm0, [rcx + r10 * 4]
    vaddps ymm0, ymm0, [rdx + r10 * 4]
    vmovups [r8 + r10 * 4], ymm0

    add r10, 8
    jmp .avx_loop

.scalar_tail:
    cmp r10, r9
    jae .done

.tail_loop:
    vmovss xmm0, [rcx + r10 * 4]
    vaddss xmm0, xmm0, [rdx + r10 * 4]
    vmovss [r8 + r10 * 4], xmm0

    inc r10
    cmp r10, r9
    jb .tail_loop

.done:
    vzeroupper
    mov eax, 1
    ret

.unavailable:
    xor eax, eax
    ret

; Windows x64 ABI
; rcx = lhs, rdx = rhs, r8 = output, r9 = length
;
; Return value:
; eax = 1: AVX operation executed
; eax = 0: AVX unavailable; output is not modified
whitebase_gnu_asm_add_f64_array_avx:
    sub rsp, CALL_FRAME_SIZE
    mov [rsp + 32], rcx
    mov [rsp + 40], rdx
    mov [rsp + 48], r8
    mov [rsp + 56], r9

    call whitebase_gnu_asm_is_avx_available

    mov rcx, [rsp + 32]
    mov rdx, [rsp + 40]
    mov r8, [rsp + 48]
    mov r9, [rsp + 56]
    add rsp, CALL_FRAME_SIZE

    test eax, eax
    jz .unavailable

    xor r10, r10
    mov r11, r9
    and r11, -4

.avx_loop:
    cmp r10, r11
    jae .scalar_tail

    vmovupd ymm0, [rcx + r10 * 8]
    vaddpd ymm0, ymm0, [rdx + r10 * 8]
    vmovupd [r8 + r10 * 8], ymm0

    add r10, 4
    jmp .avx_loop

.scalar_tail:
    cmp r10, r9
    jae .done

.tail_loop:
    vmovsd xmm0, [rcx + r10 * 8]
    vaddsd xmm0, xmm0, [rdx + r10 * 8]
    vmovsd [r8 + r10 * 8], xmm0

    inc r10
    cmp r10, r9
    jb .tail_loop

.done:
    vzeroupper
    mov eax, 1
    ret

.unavailable:
    xor eax, eax
    ret

; Windows x64 ABI
; rcx = input, rdx = length, r8 = output
;
; Return value:
; eax = 1: AVX operation executed and output was written
; eax = 0: AVX unavailable; output is not modified
whitebase_gnu_asm_sum_f64_avx:
    ; Reserve shadow space, save three arguments and align RSP before call.
    sub rsp, CALL_FRAME_SIZE
    mov [rsp + 32], rcx
    mov [rsp + 40], rdx
    mov [rsp + 48], r8

    call whitebase_gnu_asm_is_avx_available

    mov rcx, [rsp + 32]
    mov rdx, [rsp + 40]
    mov r8, [rsp + 48]
    add rsp, CALL_FRAME_SIZE

    test eax, eax
    jz .unavailable

    vxorpd ymm0, ymm0, ymm0
    xor r9, r9
    mov r10, rdx
    and r10, -4

.avx_loop:
    cmp r9, r10
    jae .reduce_vector

    vaddpd ymm0, ymm0, [rcx + r9 * 8]
    add r9, 4
    jmp .avx_loop

.reduce_vector:
    vextractf128 xmm1, ymm0, 1
    vaddpd xmm0, xmm0, xmm1
    vhaddpd xmm0, xmm0, xmm0

.scalar_tail:
    cmp r9, rdx
    jae .done

.tail_loop:
    vaddsd xmm0, xmm0, [rcx + r9 * 8]
    inc r9
    cmp r9, rdx
    jb .tail_loop

.done:
    vmovsd [r8], xmm0
    vzeroupper
    mov eax, 1
    ret

.unavailable:
    xor eax, eax
    ret
