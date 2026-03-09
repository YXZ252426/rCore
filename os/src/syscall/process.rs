//! Process management syscalls

use crate::{config::PAGE_SIZE, mm::{PageTable, PhysAddr, VirtAddr, translated_byte_buffer}, 
            task::{TASK_MANAGER, change_program_brk, current_user_token, exit_current_and_run_next, suspend_current_and_run_next, push_maparea, unmap_area}, 
            timer::get_time_us};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    let token = current_user_token();
    let size = core::mem::size_of::<TimeVal>();

    let mut buffers = translated_byte_buffer(token, ts as *const u8, size);
    let us = get_time_us();
    let time_val = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000
    };
    let time_val_bytes = unsafe {
        core::slice::from_raw_parts(
            &time_val as *const _ as *const u8,
            size
        )
    };

    let mut bytes_written = 0;
    for buffer in buffers.iter_mut() {
        let len = buffer.len().min(time_val_bytes.len() - bytes_written);
        if len == 0 {
            break;
        }
        buffer[..len].copy_from_slice(&time_val_bytes[bytes_written..bytes_written + len]);
        bytes_written += len;
    }
    0
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    let page_table = PageTable::from_token(current_user_token());
    let viraddr = VirtAddr::from(id);
    let pte = match page_table.translate(viraddr.floor()) {
        Some(pte) => pte,
        None => return -1,
    };

    match trace_request{
        0 => {
            if pte.is_user() && pte.readable() {
                let physaddr: PhysAddr = pte.ppn().into();
                let addr = physaddr.0 | viraddr.page_offset();
                let raw_ptr = addr as *const u8;
                unsafe {
                    *raw_ptr as isize
                }
            }else {
                -1
            }
        },
        1 => {
            if pte.is_user() && pte.writable() {
                let physaddr: PhysAddr = pte.ppn().into();
                let addr = physaddr.0 | viraddr.page_offset();
                let raw_ptr = addr as *mut u8;
                unsafe {
                    *raw_ptr = data as u8;
                    0
                }
            }else {
                -1
            }
        },
        2 => {
            TASK_MANAGER.get_task_trace(id)
        },
        _ => {
            panic!("sys_trace: Invalid trace_request!");
        },
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, prot: usize) -> isize {
    trace!("kernel: sys_mmap");
    if prot & !0x7 != 0 || prot & 0x7 == 0 {return -1;}
    if !VirtAddr::from(start).aligned() {return -1;}

    let len = (len + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
    if !push_maparea(start, start + len, prot << 1) {return -1};
    0
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap");
    if !VirtAddr::from(start).aligned() {return -1;}
    let len = (len + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
    if !unmap_area(start, start + len) {return -1;}
    0
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
