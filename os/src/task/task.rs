//! Types related to task management & Functions for completely changing TCB

use super::id::TaskUserRes;
use super::{kstack_alloc, KernelStack, ProcessControlBlock, TaskContext};
use crate::task::current_task;
use crate::trap::TrapContext;
use crate::{mm::PhysPageNum, sync::UPSafeCell};
use alloc::collections::BTreeMap;
use alloc::sync::{Arc, Weak};
use alloc::vec;
use alloc::vec::Vec;
use core::cell::RefMut;

/// Task control block structure
pub struct TaskControlBlock {
    /// immutable
    pub process: Weak<ProcessControlBlock>,
    /// Kernel stack corresponding to PID
    pub kstack: KernelStack,
    /// mutable
    inner: UPSafeCell<TaskControlBlockInner>,
}

impl TaskControlBlock {
    /// Get the mutable reference of the inner TCB
    pub fn inner_exclusive_access(&self) -> RefMut<'_, TaskControlBlockInner> {
        self.inner.exclusive_access()
    }
    /// Get the address of app's page table
    pub fn get_user_token(&self) -> usize {
        let process = self.process.upgrade().unwrap();
        let inner = process.inner_exclusive_access();
        inner.memory_set.token()
    }
}

pub struct TaskControlBlockInner {
    pub res: Option<TaskUserRes>,
    /// The physical page number of the frame where the trap context is placed
    pub trap_cx_ppn: PhysPageNum,
    /// Save task context
    pub task_cx: TaskContext,

    /// Maintain the execution status of the current process
    pub task_status: TaskStatus,
    /// It is set when active exit or execution error occurs
    pub exit_code: Option<i32>,

    // mutex allocation
    pub mutex_hold: BTreeMap<usize, bool>,

    /// semaphore allocation
    pub semaphore_hold: BTreeMap<usize, usize>,

    /// need
    pub need: Vec<isize>,
}

impl TaskControlBlockInner {
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }

    #[allow(unused)]
    fn get_status(&self) -> TaskStatus {
        self.task_status
    }

    #[allow(unused)]
    pub fn need2hold(&mut self, is_mutex: bool) {
        match is_mutex {
            true => {
                self.mutex_hold.insert(self.need[1] as usize, true);
                self.need[1] = -1;
            }
            false => {
                self.semaphore_hold
                    .entry(self.need[0] as usize)
                    .and_modify(|v| *v +=1 )
                    .or_insert(1);
                self.need[0] = -1;
            }
        }
    }

    #[allow(unused)]
    pub fn unhold(&mut self, id: usize, is_mutex: bool) {
        match is_mutex {
            true => {
                self.mutex_hold.insert(id, false);
            }
            false => {
                self.semaphore_hold
                    .entry( id)
                    .and_modify(|v| *v -=1 )
                    .or_insert(0);
            }
        }
    }
}

/// set need
pub fn set_need(id: usize, is_mutex: bool) {
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    inner.need[is_mutex as usize] = id as isize;
}

/// unhold
pub fn unhold(id: usize, is_mutex: bool) {
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    inner.unhold(id, is_mutex);
}
impl TaskControlBlock {
    /// Create a new task
    pub fn new(
        process: Arc<ProcessControlBlock>,
        ustack_base: usize,
        alloc_user_res: bool,
    ) -> Self {
        let res = TaskUserRes::new(Arc::clone(&process), ustack_base, alloc_user_res);
        let trap_cx_ppn = res.trap_cx_ppn();
        let kstack = kstack_alloc();
        let kstack_top = kstack.get_top();
        Self {
            process: Arc::downgrade(&process),
            kstack,
            inner: unsafe {
                UPSafeCell::new(TaskControlBlockInner {
                    res: Some(res),
                    trap_cx_ppn,
                    task_cx: TaskContext::goto_trap_return(kstack_top),
                    task_status: TaskStatus::Ready,
                    exit_code: None,
                    mutex_hold: BTreeMap::new(),
                    semaphore_hold: BTreeMap::new(),
                    need: vec![-1; 2],
                })
            },
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
/// The execution status of the current process
pub enum TaskStatus {
    /// ready to run
    Ready,
    /// running
    Running,
    /// blocked
    Blocked,
}
