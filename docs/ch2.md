# 第二章：实现批处理系统

这一章实现了一个批处理系统，他现在依然是一个一次性的机器（状态有限，很快终止），一次在内存里装载进多个任务，而且结构是高度固定的,他的功能就是依次完成内存中的多个任务然后结束

```
_num_app:
    .quad 7
    .quad app_0_start
    .quad app_1_start
    .quad app_2_start
    .quad app_3_start
    .quad app_4_start
    .quad app_5_start
    .quad app_6_start
    .quad app_6_end
```

为什么需要lazy_init呢，因为这个结构体在编译时期是无法完成装配的，到底有多少个任务只有在链接的时候才知道？

 - lazy_static! 把 static ref APP_MANAGER: … = …; 展开成一个实际的 static（底层是 lazy_static::__lazy_static_internal::Lazy<T>）和一个关联的 getter 函数。编译后 APP_MANAGER 不是直接的 AppManager 实例，而是一个在第一次用到时才调用你的初始化闭包、然后把返回值保存
   起来的对象。
  - 第一次访问 APP_MANAGER 时会：
    1. 触发 Lazy::force，用 Once（类似 pthread_once）保证初始化只执行一次；
    2. 运行你提供的 unsafe 块，把 _num_app 的符号地址读出、形成切片、拷贝到本地数组并构造 AppManager；
    3. 把这个 AppManager 放到 UPSafeCell 里，并把结果（&'static UPSafeCell<AppManager>）保存。
  - 后续再访问就直接返回已经初始化好的 &'static UPSafeCell<AppManager>，跳过计算。
  - 对比直接 static mut APP_MANAGER: AppManager = …;：那要求你使用 const 表达式，必须在编译期确定值，不能依赖外部符号；也得手动保证 unsafe 的互斥性。lazy_static! 的“真实行为”就是在运行期第一次访问时执行你原本写在宏体里的计算，并且只执行一次、然后缓存结果，用户写
    法仍然像访问普通 static ref。



 循环的隐式触发

  - rust_main() 在初始化后只调用一次 batch::run_next_app()（main.rs:90-92），但这个函数的末尾通过 __restore 跳转到用户态，并不会返回到内核；也就是说，第一次调用就把 CPU 交给 app_0，后续的“循环”来自用户态再次陷
    入内核。
  - 每个用户程序要么通过 sys_exit 主动退出，要么因 page fault/illegal instruction 被 trap 捕获。无论哪种情况，最终都再次调用 run_next_app()：sys_exit() 直接调用（syscall/process.rs:4-8），trap 处理里的
    StoreFault/IllegalInstruction 分支也直接跳到 run_next_app()（trap/mod.rs:40-55）。

  怎么跑完所有 app

  - run_next_app() 会用 APP_MANAGER 读当前 current_app、加载它、然后 move_to_next_app() 让索引加 1（batch.rs:134-141）。加载时如果 current_app >= num_app，load_app() 会调用 exit_success() 让 QEMU 退出
    （batch.rs:100-118）。
  - 每次用户态重新进入内核（由系统调用或异常触发）就会再调用 run_next_app()；由于 current_app 在上一次调用中已经递增，下次加载的就是下一个应用。最终会依次跑完 0..num_app，再遇到越界就结束。

  所以“循环”不是在 run_next_app() 里显式写的 loop，而是由每个应用退出/崩溃后重新进入内核、再重新调用 run_next_app() 这个机制自然形成的。