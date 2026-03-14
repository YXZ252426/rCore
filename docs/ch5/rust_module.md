Rust Module 笔记

  1. 基本概念

  - crate 是编译单元（一个 main.rs/lib.rs）。
  - module 是 crate 内命名空间。
  - 文件夹本身不是模块，必须被 mod xxx; 引入后才是模块。

  2. mod / use / pub use

  - mod xxx;：声明子模块，默认私有。
  - use xxx::A;：仅当前模块可用，不对外暴露。
  - pub use xxx::A;：重导出，外部可通过当前模块路径访问 A。
  - 典型门面层：子模块私有，pub use 暴露公共 API。

  3. 可见性级别

  - 默认私有：仅当前模块及其子模块可见。
  - pub(super)：仅父模块可见。
  - pub(crate)：整个当前 crate 可见。
  - pub：对外部 crate 也可见。
  - 同一文件里同时有 pub(super) 和 pub(crate) 很正常，表示不同封装边界。

  4. super 和 crate 路径

  - 两者并存是正常写法，语义重点不同。

  5. 兄弟模块关系

  - 在同一个 mod.rs 里声明的 mod context; mod id; ... 彼此是兄弟模块。
  - 兄弟互引通常走 super::...（经父模块）或 crate::...（全路径）。

  6. 为什么跳到 mod.rs

  - mod task; 的解析规则是：找 task.rs 或 task/mod.rs。
  - 若项目采用目录式布局，IDE 跳转到 task/mod.rs 是语言规则结果。
  - mod.rs 是历史常用布局，仍然合法。

  7. 结合你的例子

  - task/ 虽然是文件夹，但被 pub mod task; 声明后就是模块。
  - task/mod.rs 里 pub use context::TaskContext; 这类写法是在聚合并对外导出接口。
  
  8. 兄弟模块访问规则（易错）

  - 同一父模块下的子模块不是天然“完全互通”。
  - 默认私有项（不加 pub）通常不能被兄弟模块直接访问。
  - 需要通过可见性暴露：pub(super)、pub(crate) 或 pub。
  - pub(super) 常用于“只在父模块这一层及其子树内共享”。