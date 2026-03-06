 物理页生命周期笔记（基于你这套实现）
[MapArea::{map, unmmap}](../../os/src/mm/memory_set.rs#L333)
  1. 分配
     MapType::Framed 在 map_one 里调用 frame_alloc() 拿到一个物理页帧，包装成 FrameTracker。
  2. 建立所有权
     拿到的 FrameTracker 会被放进 data_frames（vpn -> FrameTracker），此时 MapArea 持有这个页帧的所有权。
  3. 建立映射
     随后 page_table.map(vpn, ppn, flags) 建立页表项，形成 VPN -> PPN 映射关系。
  4. 使用阶段
     用户/内核通过虚拟地址访问该页；物理页是否“活着”取决于是否仍被某个 FrameTracker 持有。
  5. 解除映射（单页）
     unmap_one 对 Framed 先 data_frames.remove(&vpn)，把 FrameTracker 从容器移除。
     如果返回的值未被接收，会立即离开作用域并 drop。
  6. 回收触发
     FrameTracker::drop 调用 frame_dealloc(ppn)，物理页归还给帧分配器，进入可复用状态。
     然后 page_table.unmap(vpn) 清掉页表项。
  7. 整段释放
     MapArea 被整体移除或销毁时，data_frames 中剩余的全部 FrameTracker 会随容器析构而逐个 drop，完成批量回收。

  关键结论

  - 这套设计是典型 RAII：FrameTracker 的所有权决定物理页生命周期。
  - “从 data_frames 移除且无其他持有者” == “触发 drop 回收页帧”。

  常见风险检查

  1. unmap 时漏掉 remove 会造成页帧泄漏。
  2. 重复 remove 同一页可能导致逻辑错误（虽然 None 不会 drop）。
  3. Identical 映射不走 FrameTracker，不应走这套回收路径。