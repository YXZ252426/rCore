[跳转到process.rs 48行](../../os/src/syscall/process.rs#L48)
• 这个例子的核心模型

  sys_get_time 做了三件事：

  1. 通过 token + 用户指针 把用户缓冲区翻译成内核可写的分段切片。
  2. 在内核里构造 TimeVal。
  3. 把 TimeVal 按字节拷贝到这些分段切片（支持跨页）。

  ———

  底层 API 在做什么

  1. translated_byte_buffer(token, ptr, len)
     返回 Vec<&mut [u8]>（教程里写成 Vec<&'static mut [u8]>），每个元素是“一个页内片段”。
     原因：用户虚拟地址连续，但可能跨页，物理上不连续。
  2. core::slice::from_raw_parts(&time_val as *const _ as *const u8, size)
     把 TimeVal 变成 &[u8] 的“字节视图”，不复制内存。
  3. copy_from_slice
     真正发生数据复制：把内核字节写进用户缓冲区片段。

  ———

  围绕“引用”和“消费”的区别

  1. for buffer in buffers.iter_mut()
     buffer 类型是 &mut &mut [u8]（对元素的可变借用）。
     buffers 本身没被消费，循环后还能用。
  2. for buffer in buffers
     等价于 into_iter，buffers 被消费。
     buffer 直接是 &mut [u8]，写起来更直观。
     循环后 buffers 不能再用。
  3. 选型规则
     后面还要用容器就 iter_mut()；不用就 into_iter（for buffer in buffers）。

  ———

  'static 的正确理解（本例重点）

  1. 这里的 &'static mut [u8] 是 unsafe 下的“类型放宽”，不是运行时永久占用。
  2. 真安全依赖外部不变量：短期使用、不泄漏、不并发别名写、页不在使用期被回收重映射。
  3. 所以它是“编译器信你”，不是“系统替你保证”。

  ———

  一句话总结

  这个例子本质是：用页表把用户缓冲区拆成可写片段，再把内核结构体转成字节流分段拷贝；iter_mut 是借用遍历，for in buffers 是消费遍历。