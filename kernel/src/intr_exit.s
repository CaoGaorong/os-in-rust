.code32

.global intr_exit
intr_exit:
    add esp, 4 #  跳过返回地址
    mov esp, [esp] # esp直接访问得到的是第一个参数，把参数作为新的栈顶
    jmp exit

exit:
    popad 
    pop gs
    pop fs
    pop es
    pop ds
    iretd