use core::time::Duration;
extern crate alloc;
use alloc::collections::VecDeque;
use axprocess::{
    futex::FUTEX_WAIT_TASK,
    process::{current_process, current_task, yield_now_task},
};
use axtask::monolithic_task::run_queue::WAIT_FOR_EXIT;
use memory_addr::VirtAddr;

use super::{flags::FutexFlags, syscall_id::ErrorNo};

// / Futex requeue操作
// /
// / 首先唤醒src_addr对应的futex变量的等待队列中，至多wake_num个任务
// /
// / 若原队列中的任务数大于wake_num，则将多余的任务移动到dst_addr对应的futex变量的等待队列中
// /
// / 移动的任务数目至多为move_num
// /
// / 不考虑检查操作
pub fn futex_requeue(wake_num: u32, move_num: usize, src_addr: VirtAddr, dst_addr: VirtAddr) {
    let mut futex_wait_task = FUTEX_WAIT_TASK.lock();
    if !futex_wait_task.contains_key(&src_addr) {
        return;
    }
    let src_wait_task = futex_wait_task.get_mut(&src_addr).unwrap();
    for _ in 0..wake_num {
        if let Some(task) = src_wait_task.pop_front() {
            WAIT_FOR_EXIT.notify_task(false, &task);
        } else {
            break;
        }
    }

    if !src_wait_task.is_empty() {
        let move_num = move_num.min(src_wait_task.len());

        let mut temp_move_task = src_wait_task.drain(..move_num).collect::<VecDeque<_>>();
        let dst_wait_task = if futex_wait_task.contains_key(&dst_addr) {
            futex_wait_task.get_mut(&dst_addr).unwrap()
        } else {
            futex_wait_task.insert(dst_addr, VecDeque::new());
            futex_wait_task.get_mut(&dst_addr).unwrap()
        };
        dst_wait_task.append(&mut temp_move_task);
    }
}

pub fn futex(
    vaddr: VirtAddr,
    futex_op: i32,
    val: u32,
    timeout: usize,
    vaddr2: VirtAddr,
    _val3: u32,
) -> Result<usize, ErrorNo> {
    let flag = FutexFlags::new(futex_op);
    let current_task = current_task();
    match flag {
        FutexFlags::WAIT => {
            // info!("wait");
            let process = current_process();
            let inner = process.inner.lock();
            let mut memory_set = inner.memory_set.lock();
            if memory_set.manual_alloc_for_lazy(vaddr).is_ok() {
                let real_futex_val = unsafe { (vaddr.as_usize() as *const u32).read_volatile() };
                if real_futex_val != val {
                    return Err(ErrorNo::EAGAIN);
                }
                let mut futex_wait_task = FUTEX_WAIT_TASK.lock();
                let wait_list = if futex_wait_task.contains_key(&vaddr) {
                    futex_wait_task.get_mut(&vaddr).unwrap()
                } else {
                    futex_wait_task.insert(vaddr, VecDeque::new());
                    futex_wait_task.get_mut(&vaddr).unwrap()
                };
                wait_list.push_back(current_task.as_task_ref().clone());
                // // 输出每一个键值对应的vec的长度
                drop(futex_wait_task);
                drop(memory_set);
                drop(inner);
                // debug!("ready wait!");
                // info!("time out: {}", timeout as u64);
                WAIT_FOR_EXIT.wait_timeout(Duration::from_nanos(timeout as u64));
                return Ok(0);
            } else {
                return Err(ErrorNo::EFAULT);
            }
        }
        FutexFlags::WAKE => {
            // info!("wake!");
            // // 当前任务释放了锁，所以不需要再次释放
            let mut futex_wait_task = FUTEX_WAIT_TASK.lock();
            if futex_wait_task.contains_key(&vaddr) {
                let wait_list = futex_wait_task.get_mut(&vaddr).unwrap();
                // info!("now task: {}", wait_list.len());
                if let Some(task) = wait_list.pop_front() {
                    // 唤醒一个正在等待的任务
                    drop(futex_wait_task);
                    WAIT_FOR_EXIT.notify_task(false, &task);
                }
            }
            yield_now_task();
            return Ok(val as usize);
        }
        FutexFlags::REQUEUE => {
            // 此时timeout相当于val2，即是move_num
            futex_requeue(val, timeout, vaddr, vaddr2);
            return Ok(0);
        }
        _ => {
            return Err(ErrorNo::EINVAL);
            // return Ok(0);
        }
    }
}