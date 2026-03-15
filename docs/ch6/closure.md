# Closure Note (for this read example)

  pub fn read<T, V>(&self, offset: usize, f: impl FnOnce(&T) -> V) -> V {
      f(self.get_ref(offset))
  }

  get_block_cache(self.indirect1 as usize, Arc::clone(block_device))
      .lock()
      .read(0, |indirect_block: &IndirectBlock| {
          indirect_block[inner_id - INODE_DIRECT_COUNT]
      })

  1. read is generic over T and V.
  2. At this call site, type inference sets T = IndirectBlock.
  3. self.get_ref(offset) returns &T, so here it becomes &IndirectBlock.
  4. The closure receives &IndirectBlock, reads one entry, and returns it.
  5. That returned entry is V, and read returns the same V.

  Why FnOnce:

  1. read invokes the closure exactly one time.
  2. FnOnce is the least restrictive bound for one-shot callbacks.
  3. Closures that are Fn/FnMut still work, because they also satisfy a one-time call requirement.

  Key point: this closure only reads data (&IndirectBlock), it does not modify the block.

## another example
```rust
let mut apps: Vec<_> = read_dir("../user/build/elf/")
          .unwrap()
          .into_iter()
          .map(|dir_entry| {
              let mut name_with_ext = dir_entry.unwrap().file_name().into_string().unwrap();
              name_with_ext.drain(name_with_ext.find('.').unwrap()..name_with_ext.len());
              name_with_ext
          })
          .collect();
```
note that map change one iterator into another iterator and collect change the mapped iterator into Vec in this example

• 这段代码的作用是：读取 ../user/build/elf/ 目录下的文件名，去掉扩展名，收集成一个 Vec<String>。

  逐段看：

  let mut apps: Vec<_> = read_dir("../user/build/elf/")

  - read_dir(...) 返回目录迭代器（每项是 Result<DirEntry, _>）。

  - Vec<_> 让编译器自动推断元素类型（这里最终是 String）。

    .unwrap()

  - 如果目录读取失败，直接 panic。

  - 把目录项变成可迭代流。

    .map(|dir_entry| {
        let mut name_with_ext = dir_entry.unwrap().file_name().into_string().unwrap();

  - dir_entry.unwrap()：目录项读取失败就 panic。

  - file_name() 取文件名（如 initproc.elf）。

  - into_string().unwrap()：把 OsString 转为 String，失败也 panic（非 UTF-8 会失败）。

      name_with_ext.drain(name_with_ext.find('.').unwrap()..name_with_ext.len());

  - 找到第一个 . 的位置。

  - 从这个位置到末尾全部删除，相当于去掉扩展名。

  - 例如 initproc.elf 变成 initproc。

  - 如果文件名里没有 .，find(...).unwrap() 会 panic。

      name_with_ext
    })
    .collect();

  - map 后每项是去扩展名后的 String。

  - collect() 收集成 Vec<String>，赋给 apps。

  补一句：这段代码依赖多个 unwrap()，所以对输入格式要求很强，遇到异常文件名会直接崩。


› 以及我对迭代器和mao，collect等api不熟悉


• 那就把这段当成“迭代器流水线”来理解：

  read_dir(...) -> unwrap -> into_iter -> map -> collect

  核心思路：先拿一批数据，再一项项加工，最后收集成容器。

    1. read_dir("../user/build/elf/")

  - 返回“目录项集合”（严格说是一个可迭代对象）。
  - 每个元素是 Result<DirEntry, Error>。

    2. .unwrap()

  - 把 Result 里成功值拿出来，失败就 panic。

    3. .into_iter()

  - 变成迭代器，后面就能链式调用 map/filter/...。
  - 可以理解为“准备开始一条一条处理”。

    4. .map(|dir_entry| { ... })

  - map = “映射/转换”：输入一个元素，输出一个新元素。
  - 这里把 DirEntry 转成“去扩展名后的文件名字符串”。
  - 所以 map 后，元素类型从目录项变成 String。

    5. .collect()

  - 把迭代器里处理完的结果收集到容器里。
  - 由于左边写了 let mut apps: Vec<_>，所以收集成 Vec<String>。

  ———

  你可以先记住这套模板：

  let out: Vec<_> = input
      .into_iter()
      .map(|x| transform(x))
      .collect();

  - into_iter：开始迭代
  - map：逐个变换
  - collect：装回容器

  ———