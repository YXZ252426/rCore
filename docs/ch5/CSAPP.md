# 程序执行的最小模型（程序视角）

## 1 程序入口并不是 `main`

程序执行流程：

```
ELF entry  
   ↓  
\_start (runtime入口)  
   ↓  
runtime初始化  
   ↓  
main(argc, argv)  
   ↓  
exit
```

因此：

```
CPU执行的第一条指令 ≠ main
```

而是运行时入口 `_start`。

---

# 2 CPU执行程序的基本循环

CPU本质执行模型：

```
while(true):  
    inst = memory\[PC\]  
    execute(inst)  
    PC = next
```

即：

```
Fetch → Decode → Execute
```

最核心状态：

```
PC = 下一条指令地址
```

---

# 3 为什么需要寄存器

如果没有寄存器：

```
a = b + c
```

必须频繁访问内存。

寄存器提供：

```
CPU内部高速存储
```

例如：

```
add x1, x2, x3  
x1 = x2 + x3
```

因此寄存器保存：

```
当前计算状态
```

---

# 4 为什么需要栈

函数调用存在嵌套：

```
main  
 └─ foo  
     └─ bar
```

每次调用需要保存：

```
返回地址  
参数  
局部变量  
保存寄存器
```

因此需要：

```
stack frame
```

结构：

```
| bar frame |  
| foo frame |  
| main frame|
```

---

# 5 为什么栈只需要一个指针

栈是：

```
LIFO
```

CPU只需要维护：

```
SP (stack pointer)
```

操作：

```
push → SP -= size  
pop  → SP += size
```

函数调用天然符合：

```
嵌套结构
```

因此使用栈。

---

# 6 程序执行状态由什么组成

从CPU视角：

```
PC        → 执行到哪条指令  
registers → 当前计算状态  
SP        → 当前函数调用栈
```

---

# 7 为什么 context switch 要保存寄存器

线程切换时：

CPU寄存器会被覆盖。

因此需要保存：

```
PC  
SP  
general registers
```

形成：

```
context
```

切换流程：

```
save(thread A registers)  
load(thread B registers)
```

---

# 8 为什么不用保存栈

栈在内存中：

```
memory
```

只需要恢复：

```
SP
```

即可重新访问原来的栈。

---

# 9 一个线程的本质

线程可以抽象为：

```
thread = (PC + registers + SP)
```

或者更本质：

```
execution state = registers
```

因为：

```
PC / SP 本质也是寄存器
```

---

# 一句话总结

CPU执行程序只需要保存：

```
PC        → 控制流  
registers → 计算状态  
SP        → 调用栈
```

而：

```
thread = CPU execution state
```

操作系统调度线程本质就是：

```
保存 / 恢复寄存器
```

# 1 用户进程虚拟地址空间（典型布局）

（64bit Linux 简化版）

```
高地址  ───────────────────────────  
            Kernel Space  
        (内核映射，用户不可访问)  
──────────────────────────────────  
            Stack  
            ↓ 向低地址增长  
            │  
            │  stack frame  
            │  stack frame  
            │  
            └── SP (当前栈顶)  
  
            Guard Page  
──────────────────────────────────  
            Memory Mapping Area  
            mmap()  
            shared library  
            file mapping  
──────────────────────────────────  
            Heap  
            ↑ 向高地址增长  
            malloc / new  
──────────────────────────────────  
            .bss  
──────────────────────────────────  
            .data  
──────────────────────────────────  
            .rodata  
──────────────────────────────────  
            .text  
            程序代码  
低地址  ───────────────────────────
```

关键点：

```
.text / .data / .bss     固定  
heap                     向高地址增长  
stack                    向低地址增长
```

---

# 2 stack 内部结构

假设当前调用：

```
main  
 └─ foo  
     └─ bar
```

stack 可能是：

```
高地址  
──────────────────  
| main frame      |  
|-----------------|  
| foo frame       |  
|-----------------|  
| bar frame       | ← 当前函数  
|-----------------|  
|                 |  
SP ────────────────  
低地址
```

每个 **stack frame** 通常包含：

```
return address  
saved registers  
local variables  
arguments
```