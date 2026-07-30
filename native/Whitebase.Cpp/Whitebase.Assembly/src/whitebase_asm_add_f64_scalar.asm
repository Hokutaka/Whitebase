OPTION CASEMAP:NONE

.code

PUBLIC whitebase_asm_add_f64_scalar

; double whitebase_asm_add_f64_scalar(
;     double lhs,   ; XMM0
;     double rhs    ; XMM1
; );
; return value      ; XMM0

whitebase_asm_add_f64_scalar PROC
    addsd xmm0, xmm1
    ret
whitebase_asm_add_f64_scalar ENDP

END
