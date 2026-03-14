
[memoy_set::unmap_area](../../os/src/mm/memory_set.rs#L294)
  1. 会不会在 areas.remove(index) 时自动释放 data_frames 里的页帧？
     会。remove 把 MapArea 移出 Vec，这个 MapArea 被 drop 时，里面的 BTreeMap<VPN, FrameTracker> 也会 drop，FrameTracker::drop 会调用 frame_dealloc。
  2. 那是不是可以不在 unmap_one 里 data_frames.remove(&vpn)？
     不能简单这么做。因为 unmap_one 不只用于“整段删除”，还用于 shrink_to 这种“部分取消映射但 area 还活着”的场景。
     如果不 remove，这些页帧会继续被 data_frames 持有，不能及时回收，且 data_frames 与 vpn_range 不一致。
  3. 只靠 areas.remove 还有一个更大问题
     如果你连 page_table.unmap(vpn) 也不做，页表项还在，但物理页可能已回收并复用，会变成悬空映射/严重错误。

  所以当前设计是对的：unmap_one 里同步做两件事


MapArea没有显式Drop，那他的Drop逻辑是什么样的，是递归销毁吗？
• 你这个理解点非常好。

  MapArea 没有手写 Drop 时，Rust 会自动生成“drop glue”（默认析构逻辑），行为是：

  1. MapArea 被销毁时（比如 self.areas.remove(index) 后那个值离开作用域），
  2. 自动递归销毁它的每个字段，
  3. data_frames: BTreeMap<_, FrameTracker> 会销毁所有 value，
  4. 每个 FrameTracker 的 Drop 被调用，执行 frame_dealloc(ppn)。

  所以：即使 MapArea 不写 Drop，data_frames 里的页帧也会被递归回收。

  但注意一件事：
  默认 Drop 只管“对象内字段”，不会自动去清页表项。
  所以 page_table.unmap(vpn) 这种外部状态更新，仍然必须显式做。