一、最核心的问题：为什么要拆成 TaskManager / TaskManagerInner？
结论先行

这是一个 “外壳 + 可变核心”（shell + mutable core）的设计，用来：

绕过 Rust 静态借用检查在内核中的限制

把 并发安全策略 和 调度逻辑 解耦

为以后从 UP → SMP 做结构上的铺垫

二、TaskManager：不可变外壳（逻辑所有者）
pub struct TaskManager {
    num_app: usize,
    inner: UPSafeCell<TaskManagerInner>,
}

1️⃣ 为什么 TaskManager 自身几乎不可变？

注意：
TaskManager 的方法基本都是：

&self


而不是：

&mut self


这在内核里极其重要。

原因：

内核里：

中断

syscall

trap

scheduler

都可能同时引用 TASK_MANAGER

你不可能在这些路径上拿到 &mut TaskManager

所以设计原则是：

TaskManager 作为一个全局单例，只允许“逻辑上可变”，而不是“类型系统上的可变”

这就是 UPSafeCell 存在的意义。

2️⃣ UPSafeCell 在这里承担什么角色？
inner: UPSafeCell<TaskManagerInner>


它表达的不是“数据结构”，而是 并发模型假设：

当前内核是单核（UP），同一时刻只可能有一个执行流在内核态

于是：

用 unsafe 包装

在运行期保证互斥

换取 灵活的可变访问

这是内核常见模式，不是 Rust trick。

三、TaskManagerInner：真正“会变”的东西
pub struct TaskManagerInner {
    tasks: [TaskControlBlock; MAX_APP_NUM],
    current_task: usize,
}

1️⃣ 为什么 tasks 和 current_task 放在一起？

因为它们：

语义上必须原子一致

生命周期强绑定

任何一次调度，都会涉及：

(current_task, tasks[current].status, tasks[next].status)


如果你把它们拆散：

借用会非常复杂

很容易出现“状态已经改了，但 current_task 还没改”的中间态

所以它们被强制放进一个独占可变区域。

2️⃣ 为什么 current_task 不是放在 TaskControlBlock 里？

这是一个常见的“新手直觉错误”。

错误设计：

struct TaskControlBlock {
    is_current: bool,
    ...
}


问题是：

会有 多个 true 的可能

需要 O(n) 扫描

状态一致性无法保证

而现在的设计是：

current_task : usize   ← 全局唯一
tasks[i].task_status   ← 局部状态


不变量非常清晰：

tasks[current_task].status == Running

四、为什么 TaskControlBlock 这样设计？
pub struct TaskControlBlock {
    pub task_status: TaskStatus,
    pub task_cx: TaskContext,
    pub task_trace: [usize; MAX_SYSCALL_NUM],
}

1️⃣ TCB 是“被调度对象”，不是“调度器”

TCB 的设计原则是：

只保存“与单个任务强绑定”的状态

因此：

字段	为什么在 TCB 里
task_status	生命周期状态，属于任务自身
task_cx	CPU 上下文，任务私有
task_trace	syscall 统计，任务私有

而这些 都不属于调度器本身的职责。

2️⃣ 为什么 TaskContext 在 TCB 里，而不是在 TaskManager？

因为：

context switch 是 任务 ↔ 任务

scheduler 只是“撮合者”

你在 __switch 里看到的就是：

__switch(
    &mut current.task_cx,
    &next.task_cx,
)


这是一种非常干净的语义分离。

五、为什么不用“面向对象”的设计？

比如：

impl TaskControlBlock {
    fn run(&mut self) { ... }
}


在用户态可能没问题，但在内核里会带来灾难：

TCB 不应该知道“谁是下一个任务”

调度策略不应该嵌进任务结构

借用关系会变得极其复杂

所以这里是：

TaskManager  → 决定调度
TaskControlBlock → 承载状态


非常传统、非常稳定的 OS 架构。

六、这套设计的“隐藏收益”

这不是 ch3 才用得上的设计，而是为后续演进铺路：

1️⃣ SMP（多核）

UPSafeCell → Mutex / SpinLock

TaskManagerInner 不需要改

2️⃣ 抢占式调度

中断里访问 TASK_MANAGER

依然只需要 &self

3️⃣ 更复杂调度器

把 tasks: [TCB] 换成：

run queue

ready list

RB-tree

外部接口完全不动。