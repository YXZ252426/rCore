# user-app是如何被加载的

首先user-app是一个个elf，这个elf在物理内存上是连续的

首先在build.rs从文件目录中逐个加载其名字

build.rs最终会生产一个link_app.S，这个可以看作一个“条约”，其用 .incbin 把每个 *.elf 原样嵌进内核镜像的数据段，并导出符号：
      - _num_app
      - app_i_start / app_i_end
      - _app_names

这个符号只是链接的一个标识符，本质是一个物理内存的地址以及背后蕴含的数据

.quad yyy
      - 在当前位置写入一个 8 字节（64-bit）数据。

因此.quad app_1_start就代表在这个标识符背后是一个u8的app_1的起始地址，这样就实现了逻辑划分和物理地址的映射

通过这两个例子也可以很好体现逻辑划分和物理地址的映射

这已经是其底层，涉及到对逐个物理地址操作,可以说是直接面向硬件，面向内存编程了

```rust
    let num_app_ptr = _num_app as usize as *const usize;
    let num_app = get_num_app();
    let app_start = unsafe { core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1) };
    assert!(app_id < num_app);
    unsafe {
        core::slice::from_raw_parts(
            app_start[app_id] as *const u8,
            app_start[app_id + 1] - app_start[app_id],
        )
    }
```

```rust
        unsafe {
            for _ in 0..num_app {
                let mut end = start;
                while end.read_volatile() != b'\0' {
                    end = end.add(1);
                }
                let slice = core::slice::from_raw_parts(start, end as usize - start as usize);
                let str = core::str::from_utf8(slice).unwrap();
                v.push(str);
                start = end.add(1);
            }
        }
        v
```

随后在loade app之后就回会交给TCB，建立虚拟内存，分页机制，并且把elf实际分配到内存空间，再更高的，建立进程！！

自此，就实现了从0，1的二进制数与地址，到进程的抽象！

从完全的混沌，建立起可调度，可隔离，可管理的秩序