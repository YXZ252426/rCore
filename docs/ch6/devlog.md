## very beginning
```rust
impl File for Stdout {
    fn readable(&self) -> bool {
        false
    }
    fn writable(&self) -> bool {
        true
    }
    fn read(&self, _user_buf: UserBuffer) -> usize {
        panic!("Cannot read from stdout!");
    }
    fn write(&self, user_buf: UserBuffer) -> usize {
        for buffer in user_buf.buffers.iter() {
            print!("{}", core::str::from_utf8(*buffer).unwrap());
        }
        user_buf.len()
    }
}

```

understand the principle that everything is a file

```rust
pub struct TaskControlBlockInner {
    /// The physical page number of the frame where the trap context is placed
    pub trap_cx_ppn: PhysPageNum,

    /// Application data can only appear in areas
    /// where the application address space is lower than base_size
    pub base_size: usize,

    /// Save task context
    pub task_cx: TaskContext,

    /// Maintain the execution status of the current process
    pub task_status: TaskStatus,

    /// Application address space
    pub memory_set: MemorySet,

    /// Parent process of the current process.
    /// Weak will not affect the reference count of the parent
    pub parent: Option<Weak<TaskControlBlock>>,

    /// A vector containing TCBs of all child processes of the current process
    pub children: Vec<Arc<TaskControlBlock>>,

    /// It is set when active exit or execution error occurs
    pub exit_code: i32,
    pub fd_table: Vec<Option<Arc<dyn File + Send + Sync>>>,

    /// Heap bottom
    pub heap_bottom: usize,

    /// Program break
    pub program_brk: usize,
}
```

compare the concept in standard text book and the real implementation in code

for example children will inherit the fd_table

Fd_table is a vector, just like what book says

understand rust polymorphism, dyn

understand trait Send, Sync and smart pointer Arc, 

understand bitflags

understand use lib open sys call

```rust
pub fn main() -> i32 {
    let test_str = "Hello, world!";
    let filea = "filea\0";
    let fd = open(filea, OpenFlags::CREATE | OpenFlags::WRONLY);
    assert!(fd > 0);
    let fd = fd as usize;
    write(fd, test_str.as_bytes());
    close(fd);

    let fd = open(filea, OpenFlags::RDONLY);
    assert!(fd > 0);
    let fd = fd as usize;
    let mut buffer = [0u8; 100];
    let read_len = read(fd, &mut buffer) as usize;
    close(fd);

    assert_eq!(
        test_str,
        core::str::from_utf8(&buffer[..read_len]).unwrap(),
    );
    println!("file_test passed!");
    0
}
```



this file show how the user app achieve file RW, because fd is managed by FCB and it is in sys mode, so the procedure is ask for fd first and get the given fd and trigger another syscall

1. open("filea", CREATE | WRONLY)

   - asks the kernel to open or create a file
   - kernel allocates a **file descriptor fd**
   - kernel returns that integer fd to user space

2. write(fd, test_str.as_bytes())

   - user passes the fd back into the kernel
   - kernel uses that fd to find the **corresponding** opened file object
   - kernel writes "Hello, world!" into the file

3. close(fd)

   - tells kernel to release this open-file entry

   
## 3.13

read DiskInode, Bitmap and DirEntry

what makes me really excited is that i establish the map between the real kernel implement and the theoretical implement in book

for example, the multi level index in DiskInode, every file only take one unique inode, therefore lead to the concept hardlink

also the bitmap, Through nested loops and bitwise operations, enabling precise manipulation down to every single bit.

read efs

> in computer system memory is the first class citizen, we can understand this through the implement of efs and func create
i can understand more about install OS and format fs by comparing the Linux provided API and the implement in rCore
all the disk manipulate is in BlockCache, therefore in memory, take create as an example, it's main function is to construct the efs struct, and only at the very end all the cache are sync in disk
so in disk, it can actually be called as fs image in disk
disk is actually linear, and with the help of superblock, we establish logical data structure in disk

After reading efs.rs, I think this implementation shows that filesystem operations are memory-centered. Most reads and writes go through the block cache, so the program usually modifies cached blocks in memory first
and writes them back to disk later.

In EasyFileSystem::create, the code does more than build the EasyFileSystem struct. It actually formats the filesystem: it clears all blocks, initializes the superblock, creates the root inode, and finally syncs
cached data to disk.

It is also helpful to compare this with Linux concepts. EasyFileSystem::create is closer to formatting a filesystem image, while open is closer to opening an existing filesystem.

The disk itself is just a linear array of blocks. The filesystem gives those blocks meaning by organizing them with the superblock, bitmaps, inode area, and data area.