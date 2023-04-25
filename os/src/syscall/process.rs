//! Process management syscalls
use core::mem;

use alloc::vec::Vec;

use crate::{
    config::MAX_SYSCALL_NUM,
    mm::{translated_byte_buffer, VirtAddr},
    task::{
        change_program_brk, current_user_token, exit_current_and_run_next,
        suspend_current_and_run_next, TaskStatus, TASK_MANAGER,
    },
    timer::get_time_us,
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
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

fn copy(dst: Vec<&mut [u8]>, src_ptr: usize, size: usize) {
    let mut written = 0;

    assert!(dst.iter().map(|b| b.len()).sum::<usize>() == size);

    for buffer in dst {
        unsafe {
            buffer.copy_from_slice(core::slice::from_raw_parts(
                (src_ptr + written) as *const u8,
                buffer.len(),
            ))
        }
        written += buffer.len();
    }
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");

    let buffers = translated_byte_buffer(
        current_user_token(),
        ts as *mut u8,
        mem::size_of::<TimeVal>(),
    );
    let us = get_time_us();
    let ts = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };

    copy(
        buffers,
        &ts as *const TimeVal as usize,
        mem::size_of::<TimeVal>(),
    );

    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info NOT IMPLEMENTED YET!");

    let buffers = translated_byte_buffer(
        current_user_token(),
        ti as *mut u8,
        mem::size_of::<TaskInfo>(),
    );

    let (ts, st, t) = TASK_MANAGER.get_current_task_info();

    let ti = TaskInfo {
        status: ts,
        syscall_times: st,
        time: t,
    };

    copy(
        buffers,
        &ti as *const TaskInfo as usize,
        mem::size_of::<TaskInfo>(),
    );

    0
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    let start_va = VirtAddr::from(start);
    if !start_va.aligned() {
        error!("sys_mmap: non-aligned start_va {:?}", start_va);
        return -1;
    }

    if port & !0x7 != 0 || port & 0x7 == 0 {
        error!("sys_mmap: invalid port {}", port);
        return -1;
    }
    let end_va = VirtAddr::from(start + len);

    if TASK_MANAGER.check_range_mapped(start_va, end_va) {
        error!("sys_mmap: range {:?}-{:?} mapped", start_va, end_va);
        return -1;
    }
    TASK_MANAGER.insert_framed_area(start_va, end_va, port);
    0
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    let start_va = VirtAddr::from(start);
    if !start_va.aligned() {
        error!("sys_mmap: non-aligned start_va {:?}", start_va);
        return -1;
    }
    let end_va = VirtAddr::from(start + len);

    if !TASK_MANAGER.check_range_all_mapped(start_va, end_va) {
        error!(
            "sys_mmap: part of or all range {:?}-{:?} unmapped",
            start_va, end_va
        );
        return -1;
    }

    TASK_MANAGER.remove_framed_area(start_va, end_va);

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
