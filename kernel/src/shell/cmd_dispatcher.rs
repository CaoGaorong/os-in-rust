use crate::{filesystem::{FileDescriptor, StdFileDescriptor}, pipe::{self, PipeError}, println, sys_call};

use super::{cmd_executor, shell_util};


/**
 * 根据shell的输出，解析和分配命令的执行
 */
#[inline(never)]
pub fn dispatch_cmd(cwd: &str, input: &str, buf: &mut [u8]) {
    if input.trim().is_empty() {
        return;
    }
    unsafe { buf.as_mut_ptr().write_bytes(0, buf.len()) };
    let input = input.trim();

    // 把参数，按照管道分隔
    let cmd_split = input.split("|");
    let cmd_cnt = cmd_split.filter(|s|!s.trim().is_empty()).count();
    // 管道的数量 = 命令的数量 - 1
    let pipe_cnt = cmd_cnt - 1;
    // 没有pipe，直接执行命令
    if pipe_cnt == 0 {
        let fork_res = sys_call::fork();
        match fork_res {
            sys_call::ForkResult::Parent(child_id) => {
                // 阻塞等待子进程退出
                let wait_res = sys_call::wait();
                if wait_res.is_none() {
                    println!("child process does not exit");
                }
                let (chpid, exit_status) = wait_res.unwrap();
                println!("child process exit");
                // println!("child process exit. cur pid:{}, child pid:{}, child status:{:?}", sys_call::get_pid().get_data(), chpid.get_data(), exit_status);
            },
            sys_call::ForkResult::Child => {
                let (cmd, param) = shell_util::parse_cmd(input);
                cmd_executor::execute_cmd(cwd, cmd, param, buf);
                sys_call::exit(0);
            },
        }
        return;
    }
    
    // 保存管道结果
    let pipe_res_size: usize = size_of::<Result<FileDescriptor, PipeError>>() * pipe_cnt;
    // 管道结果数组
    let pipe_result_list = unsafe { core::slice::from_raw_parts_mut(sys_call::malloc(pipe_res_size) as *mut _ as *mut Result<FileDescriptor, PipeError>, pipe_cnt) };
    self::batch_create_pipe(pipe_result_list);

    let cmd_split = input.split("|");
    let cmd_iterator = cmd_split
        // 取出空白
        .map(|s|s.trim())
        // 过滤掉空的命令
        .filter(|s|!s.is_empty());
    // 遍历每个管道隔开的命令
    for (idx, cmd) in cmd_iterator.enumerate() {
            let cmd = cmd.trim();
                // 解析单个命令
            let (cmd, param) = shell_util::parse_cmd(cmd);
            
            let fork_res = sys_call::fork();
            // 如果是父进程，继续下一个循环
            if let sys_call::ForkResult::Parent(_) = fork_res {
                continue;
            }
            // 如果是子进程，那么就执行命令

            // 如果是第一个循环，那么输出的文件描述符重定向
            // if idx == 0 {
            //     // 标准输出 重定向到管道的写入
            //     sys_call::redirect_file_descriptor(FileDescriptor::new(StdFileDescriptor::StdOutputNo as usize), *cur_fd);
            // // 如果是最后一个循环
            // } else if idx == pipe_cnt {
            //     let last_fd = pipe_result_list[idx - 1].as_ref().unwrap();
            //     // 标准输入，改为从管道输入
            //     sys_call::redirect_file_descriptor(FileDescriptor::new(StdFileDescriptor::StdInputNo as usize), *last_fd);
            // } else {
            //     let last_fd = pipe_result_list[idx - 1].as_ref().unwrap();
            //     // 标准输出 重定向到管道的写入
            //     sys_call::redirect_file_descriptor(FileDescriptor::new(StdFileDescriptor::StdOutputNo as usize), *cur_fd);
            //     // 标准输入，改为从管道输入
            //     sys_call::redirect_file_descriptor(FileDescriptor::new(StdFileDescriptor::StdInputNo as usize), *last_fd);
            // }
            // 开始执行命令
            cmd_executor::execute_cmd(cwd, cmd, param, buf)
    };

    // 父进程在这里统一循环等待
    for _ in 0..cmd_cnt {
        sys_call::wait();
    }

}

fn batch_create_pipe(pipe_result_list: &mut [Result<FileDescriptor, PipeError>]) {
    for pipe in pipe_result_list {
        // 把结果放进去
        *pipe = sys_call::pipe(200);
        assert!(pipe.is_ok());
    }
}