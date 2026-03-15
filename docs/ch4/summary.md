难点在于一层一层向上封装，有好几层抽象

从最底层的`frame_allocator`到`MemoryArea`到 `MemorySet`这些内存管理（mm）结构体，到`TaskControlBlock` `TaskManagerInner` `TASKMANNAGER`这些任务管理结构体，甚至之后还要从TCB进化成PCB即进程

分清物理的内存读写与逻辑的资源管理