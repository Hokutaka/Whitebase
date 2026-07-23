OPTION CASEMAP:NONE

.code

PUBLIC whitebase_asm_add

whitebase_asm_add PROC
    mov eax, ecx
    add eax, edx
    ret
whitebase_asm_add ENDP

END