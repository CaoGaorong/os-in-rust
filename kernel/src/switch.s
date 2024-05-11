.code32

# 函数原型：fn switch_to(cur_task: &mut TaskStruct, task_to_run: &mut TaskStruct);
.global switch_to
switch_to:
   # 保存当前进程 cur_task的上下文
   push esi
   push edi
   push ebx
   push ebp

   # esp + 4 * 4，得到的是switch_to的返回地址，esp + 20得到的是cur_task参数的地址
   mov eax, [esp + 20] # [esp + 20]得到把cur_task作为指针，解引用得到的值。取前4字节，也就是kernel_stack变量
   mov [eax], esp # 把当前esp寄存器的值，存入到kernel_stack变量中

   # 下面是恢复将要运行的任务的上下文
   mov eax, [esp + 24] # esp + 24 得到第二个参数，也就是task_to_run变量。[esp + 24] 是把task_to_run解引用，得到TaskStruct的值，取4字节，也就是kernel_stack变量
   mov esp, [eax] # 把 kernel_stack的变量的值，也就是栈顶的地址，恢复到esp寄存器
   pop ebp
   pop ebx
   pop edi
   pop esi
   ret	