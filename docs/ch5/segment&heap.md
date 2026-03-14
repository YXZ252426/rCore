# 程序内存逻辑段总结

程序运行时，虚拟地址空间通常可分为 `text / rodata / data / bss / heap / stack` 等区域，不同区域负责存放不同类型的数据。

## 1\. text（代码段）

**作用**：存放程序的机器指令。  
**存什么**：函数体编译后的指令。  
**特点**：

-   通常只读、可执行
    
-   可被多个进程共享
    
-   防止程序修改自身代码
    

例：`main()`、`printf()` 的指令都在 text。

---

## 2\. rodata（只读数据段）

**作用**：存放只读常量。  
**存什么**：

-   字符串字面量，如 `"hello"`
    
-   `const` 全局变量
    
-   常量表、跳转表、浮点常量等
    

**特点**：

-   只读，不可写
    
-   常与 text 一样可共享
    
-   运行时写入会导致异常
    

例：

```
C

char \*s \= "abc";
```

`s` 在栈上，`"abc"` 在 rodata。

---

## 3\. data（已初始化数据段）

**作用**：存放已初始化的全局变量、静态变量。  
**存什么**：

```
C

int g \= 10;  
static int s \= 20;
```

**特点**：

-   程序加载时即存在
    
-   生命周期贯穿整个进程
    
-   可读可写
    

---

## 4\. bss（未初始化数据段）

**作用**：存放未初始化或零初始化的全局变量、静态变量。  
**存什么**：

```
C

int g;  
static int s;  
int x \= 0;
```

**特点**：

-   加载时由系统清零
    
-   可执行文件中不必真正存一堆 0
    
-   节省文件体积
    

---

## 5\. heap（堆）

**作用**：存放运行时动态分配的内存。  
**存什么**：

-   `malloc/new`
    
-   Rust `Box/Vec/String` 的动态部分
    

**特点**：

-   生命周期由程序控制
    
-   大小可变
    
-   分配释放灵活，但管理复杂
    
-   易出现泄漏、碎片、悬垂指针
    

---

## 6\. stack（栈）

**作用**：存放函数调用过程中的临时数据。  
**存什么**：

-   局部变量
    
-   函数参数
    
-   返回地址
    
-   保存的寄存器现场
    

**特点**：

-   自动分配、自动回收
    
-   速度快
    
-   空间较小
    
-   递归过深或大局部数组会栈溢出
    

---

## 7\. 总表

| 段 | 存储内容 | 生命周期 | 可写性 |
| --- | --- | --- | --- |
| text | 机器指令 | 整个进程期 | 否 |
| rodata | 只读常量 | 整个进程期 | 否 |
| data | 已初始化全局/静态变量 | 整个进程期 | 是 |
| bss | 未初始化全局/静态变量 | 整个进程期 | 是 |
| heap | 动态分配内存 | 程序控制 | 是 |
| stack | 局部变量/调用现场 | 函数调用期间 | 是 |

---

# 这段内核堆代码总结

代码本质：**用一块静态预留内存充当内核堆，再让堆分配器管理它。**

```
Rust

static HEAP\_ALLOCATOR: LockedHeap \= LockedHeap::empty();  
  
#\[alloc\_error\_handler\]  
pub fn handle\_alloc\_error(layout: core::alloc::Layout) -> ! {  
    panic!("Heap allocation error, layout = {:?}", layout);  
}  
  
static mut HEAP\_SPACE: \[u8; KERNEL\_HEAP\_SIZE\] \= \[0; KERNEL\_HEAP\_SIZE\];  
  
pub fn init\_heap() {  
    unsafe {  
        HEAP\_ALLOCATOR  
            .lock()  
            .init(HEAP\_SPACE.as\_ptr() as usize, KERNEL\_HEAP\_SIZE);  
    }  
}
```

---

## 1\. HEAP\_ALLOCATOR

`LockedHeap` 是**堆分配器实例**。  
`empty()` 表示一开始它还没有接管任何内存区域。  
`Locked` 表示带锁，适合内核并发访问。

---

## 2\. HEAP\_SPACE

```
Rust

static mut HEAP\_SPACE: \[u8; KERNEL\_HEAP\_SIZE\] \= \[0; KERNEL\_HEAP\_SIZE\];
```

这是**真正作为内核堆底层存储的原始内存池**。  
本质上它是一个**静态大数组**，通常因为全 0 初始化而落在 `.bss`。

注意：

-   它不是“堆上分配出来的内存”
    
-   而是“先静态保留，再逻辑上当堆使用”
    

---

## 3\. init\_heap()

```
Rust

HEAP\_ALLOCATOR.lock().init(start, size)
```

作用：把 `HEAP_SPACE` 的首地址和大小交给分配器。  
其中：

-   `start = HEAP_SPACE.as_ptr() as usize`
    
-   `size = KERNEL_HEAP_SIZE`
    

从这之后，allocator 就开始在这段连续区域内做动态分配。

---

## 4\. 堆的起始地址是什么

内核堆起始地址就是：

```
Rust

HEAP\_SPACE.as\_ptr() as usize
```

也就是 `HEAP_SPACE` 这个静态数组在内存中的首地址。

这个地址不是手写常数，而是由：

-   编译器布局
    
-   linker script
    
-   内核映像装载地址
    

共同决定的。

---

## 5\. alloc\_error\_handler

当 `Box/Vec/String` 等堆分配失败时，会调用：

```
Rust

#\[alloc\_error\_handler\]
```

这里的策略是直接 `panic`。

---

## 6\. 本质理解

这段代码实现的是：

**静态内存池 + 堆分配器 = 内核堆**

更准确地说：

-   `HEAP_SPACE`：原始内存来源
    
-   `LockedHeap`：管理这块内存的分配器
    
-   `Box/Vec/String`：这块堆的使用者
    

---

## 7\. 一句话总结

这不是“系统自动给内核生成了一块堆”，而是：

**在内核镜像的静态区域中预留 `HEAP_SPACE`，再把它的首地址交给 `LockedHeap` 管理，于是这块静态内存被逻辑上变成了内核堆。**

---

## 8\. 和用户堆的区别

这里初始化的是**内核堆**，不是用户进程堆。

-   内核堆：给内核里的动态对象用
    
-   用户堆：给用户程序 `malloc/new` 用，通常依赖进程地址空间、`brk/mmap` 等机制
    

所以这段代码只解决了**内核自身动态分配**的问题。

---