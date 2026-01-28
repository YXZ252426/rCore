下面是**整理成「可当学习笔记」的 RISC-V 用户态 → 内核态 → 返回/切换 的完整 trap 流程总结**。  
我会刻意保持**结构化、抽象稳定、偏规范**，方便你以后反复对照源码。

---

# RISC-V Trap / Syscall 完整流程笔记

> 适用背景：
> 
> -   RISC-V
>     
> -   U → S trap
>     
> -   `stvec = Direct`
>     
> -   教学 OS / rCore 风格
>     
> -   单核、非抢占式
>     

---

## 一、核心概念速览

### 1\. 什么是 trap

**trap 是硬件主导的控制流转移机制**，包括：

-   同步异常（`ecall`、page fault、illegal instruction）
    
-   异步中断（timer / external）
    

在 RISC-V 中：

-   syscall（`ecall`）只是**一种同步异常**
    

---

### 2\. trap 相关关键 CSR

| CSR | 作用 |
| --- | --- |
| `stvec` | trap 统一入口 |
| `sepc` | trap 发生时的 PC |
| `scause` | trap 原因 |
| `stval` | 附加信息 |
| `sstatus` | 特权级 / 中断状态 |
| `sscratch` | 临时保存用户栈指针 |

---

## 二、trap 总体结构（高层）

```text
用户态(U)
│
│ ecall / fault / interrupt
▼
硬件 trap
│  保存 sepc/scause
│  U → S
▼
__alltraps（汇编）
│  保存寄存器
│  构造 TrapContext
│  call trap_handler
▼
trap_handler（Rust）
│  分发 syscall / exception
│  决定“下一次恢复哪个上下文”
▼
__restore（汇编）
│  恢复寄存器
│  sret
▼
用户态(U)
```

---

## 三、详细流程（时间顺序）

### Step 1：用户态触发 trap

用户程序执行：

```asm
ecall
```

或发生异常（page fault / illegal）。

---

### Step 2：硬件自动行为（不执行任何软件指令）

CPU 自动完成：

1.  `sepc ← 当前 PC`
    
2.  `scause ← trap 原因`
    
3.  `sstatus`：
    
    -   `SPP ← 0`（来自 U）
        
    -   `SPIE ← SIE`
        
    -   `SIE ← 0`
        
4.  特权级切换：`U → S`
    
5.  `PC ← stvec`
    

---

## 四、汇编入口：`__alltraps`

### Step 3：切换到内核栈

```asm
csrrw sp, sscratch, sp
```

-   `sp` ← 内核栈
    
-   `sscratch` ← 用户栈
    

---

### Step 4：在内核栈构造 TrapContext

```asm
addi sp, sp, -34*8
```

保存内容包括：

-   通用寄存器 `x1–x31`
    
-   用户栈指针（来自 `sscratch`）
    
-   `sstatus`
    
-   `sepc`
    

此时：

> **内核栈上的一块内存 = 当前用户程序的完整执行现场**

---

### Step 5：进入 Rust 处理逻辑

```asm
mv a0, sp
call trap_handler
```

-   `a0 = &mut TrapContext`
    
-   `ra` 保存返回地址（即 `__restore`）
    

---

## 五、Rust 层：`trap_handler`

```rust
pub fn trap_handler(cx: &mut TrapContext) -> &mut TrapContext
```

### trap\_handler 的语义

> **决定：下一次 `sret` 要恢复哪个 TrapContext**

---

### 1️⃣ syscall（`UserEnvCall`）

```rust
cx.sepc += 4;              // 跳过 ecall
cx.x[10] = syscall(...);   // 返回值
return cx;
```

结果：

-   返回 **当前应用**
    
-   程序继续执行
    

---

### 2️⃣ sys\_exit（自愿退出）

```rust
pub fn sys_exit(...) -> ! {
    run_next_app()
}
```

特点：

-   `-> !`：永不返回
    
-   在 syscall 内直接切换到下一个应用
    
-   **不会再回到 trap\_handler 的正常返回路径**
    

---

### 3️⃣ 非法异常（被动退出）

```rust
PageFault | IllegalInstruction => {
    run_next_app();
}
```

结果：

-   当前应用被销毁
    
-   切换到下一个应用
    

---

## 六、为什么 trap\_handler 返回后一定会执行 `__restore`

### 核心原因（非常关键）

```asm
call trap_handler
__restore:
```

-   `call` 把 `__restore` 地址写入 `ra`
    
-   `trap_handler` 的 `ret`：
    
    ```asm
    jalr x0, ra, 0
    ```
    
-   **返回地址就是 `__restore`**
    

👉 这是**顺序控制流**，不是约定、不是魔法。

---

## 七、汇编返回路径：`__restore`

### Step 6：选择要恢复的上下文

```asm
mv sp, a0
```

-   `a0` = trap\_handler 返回的 `&TrapContext`
    
-   可以是：
    
    -   当前应用
        
    -   下一个应用
        

---

### Step 7：恢复 CPU 状态

```asm
csrw sstatus, ...
csrw sepc, ...
csrw sscratch, ...
ld x1, ...
...
```

---

### Step 8：释放 TrapContext

```asm
addi sp, sp, 34*8
```

---

### Step 9：恢复用户栈并返回用户态

```asm
csrrw sp, sscratch, sp
sret
```

`sret` 的硬件语义：

-   特权级：`S → U`
    
-   `PC ← sepc`
    
-   `SIE ← SPIE`
    

---

## 八、两条关键执行路径对比

### ① 普通 syscall

```text
ecall
→ trap_handler
→ return cx
→ __restore
→ sret
→ 原用户程序继续执行
```

---

### ② sys\_exit / 异常终止

```text
ecall / fault
→ trap_handler
→ run_next_app
→ __restore
→ sret
→ 下一个用户程序
```

---

## 九、设计要点总结（考试/面试级）

1.  **trap 不是函数调用，是硬件控制流**
    
2.  `stvec` 只设置一次，但每次 trap 都会跳
    
3.  `TrapContext` 是“可恢复的世界”
    
4.  trap\_handler 的返回值决定“下一个世界”
    
5.  `sys_exit -> !` 是为了保证**永不回到用户态**
    
6.  `__restore + sret` 是**唯一合法的返回路径**
    

---

## 十、一句话终极总结

> **RISC-V 的 trap 机制本质是：硬件接管控制流 → 软件保存/选择上下文 → 硬件用 `sret` 精确恢复到某个用户世界，而 `trap_handler` 的唯一职责是决定“恢复谁”。**
