[processor::run_task](../../os/src/task/processor.rs)
• 这里手动 drop 的核心目的：提前释放锁/借用，避免把独占访问带进 __switch 之后的运行阶段。

  在这段里有两个典型 guard：

  1. processor = PROCESSOR.exclusive_access()
  2. task_inner = task.inner_exclusive_access()

  它们是 RAII guard，不手动 drop 的话会等到作用域结束才释放。
  但这里在作用域内就调用了 __switch(...)，切走后可能很久才回来。若不提前释放，会导致：

  - 长时间持有 PROCESSOR / task_inner 的独占访问
  - 其他路径（如 trap/schedule）再访问这些结构时卡住或冲突（死锁/借用失败）

  所以代码在 __switch 前显式：

  - drop(task_inner)：释放 TCB 内部独占访问
  - drop(processor)：释放处理器全局独占访问

  本质就是：切上下文前清空所有 guard，不把锁跨上下文切换带走。