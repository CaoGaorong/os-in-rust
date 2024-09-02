use crate::{filesystem::{FileDescriptor, FileError, StdFileDescriptor}, pipe::{self, PipeError}, println, shell::shell_util::PathError, sys_call};

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

    // 先把shell输入，按照重定向符(>和>>)分隔
    let redirection = self::split_redirection(cwd, input, buf);
    if redirection.is_err() {
        println!("failed to exec:{}, error:{:?}", input, redirection.unwrap_err());
        return;
    }
    let (input, mut file) = redirection.unwrap();

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
                // 如果最终结果要重定向到某个文件，那么把打印到屏幕的内容写入到文件
                if file.is_some() {
                    let file = file.as_mut();
                    sys_call::set_producer(file.unwrap().get_fd());
                }
                cmd_executor::execute_cmd(cwd, cmd, param, buf);
                // 关闭文件
                if file.is_some() {
                    let file = file.as_mut();
                    file.unwrap().close();
                }
                sys_call::exit(0);
            },
        }
        return;
    }
    // 批量创建管道
    let pipes = self::batch_create_pipe(pipe_cnt);

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
            if idx == 0 {
                let cur_pipe = pipes[idx];
                // 标准输出 重定向到管道的写入
                sys_call::set_producer(cur_pipe);
            // 如果是最后一个循环
            } else if idx == cmd_cnt - 1 {
                let last_pipe = pipes[idx - 1];
                // 标准输入，改为从管道输入
                sys_call::set_consumer(last_pipe);
                // 如果最终结果要重定向到某个文件，那么把打印到屏幕的内容写入到文件
                if file.is_some() {
                    let file = file.as_ref().unwrap();
                    sys_call::set_producer(file.get_fd());
                }
            } else {
                let cur_fd = pipes[idx];
                let last_fd = pipes[idx - 1];
                // 标准输出 重定向到管道的写入
                sys_call::set_producer(cur_fd);
                // 标准输入，改为从管道输入
                sys_call::set_consumer(last_fd);
            }


            // 开始执行命令
            cmd_executor::execute_cmd(cwd, cmd, param, buf);
            
            // 关闭文件
            if idx == cmd_cnt - 1 && file.is_some() {
                let file = file.as_mut();
                file.unwrap().close();
            }

            // 执行完命令，系统调用退出
            sys_call::exit(0);
    };

    // 父进程在这里统一循环等待
    for _ in 0..cmd_cnt {
        sys_call::wait();
    }

    // 释放所有管道
    for pipe_fd in pipes.iter() {
        sys_call::release_pipe(*pipe_fd)
    }

    // 释放内存
    sys_call::free(pipes.as_ptr());
}

#[inline(never)]
fn batch_create_pipe(pipe_cnt: usize) -> &'static mut [FileDescriptor] {
    let pipe_res_size: usize = size_of::<FileDescriptor>() * pipe_cnt;
    let pipe_result_list = unsafe { core::slice::from_raw_parts_mut(sys_call::malloc(pipe_res_size) as *mut _ as *mut FileDescriptor, pipe_cnt) };
    for idx in 0 .. pipe_result_list.len() {
        let pipe = sys_call::pipe(200);
        if pipe.is_err() {
            panic!("failed to create pipe, error:{:?}", pipe.unwrap_err());
        }
        pipe_result_list[idx] = pipe.unwrap();
    }
    pipe_result_list
}

/**
 * 把输入的命令，根据>和>>解析，因为如果使用了>或者>>说明是要把结果重定向到某个文件中
 */
#[inline(never)]
fn split_redirection<'a>(cwd: &str, input: &'a str, buf: &mut [u8]) -> Result<(&'a str, Option<sys_call::File>), FileError> {
    // 如果输入不包含重定向符号，那么不处理
    if !input.contains(">") {
        return Result::Ok((input, Option::None));
    }
    // 如果包含的是>>，那么就是追加写
    if input.contains(">>") {
        let (command, file_name) = input.split_once(">>").unwrap();
        let file_name = file_name.trim();
        let file_name = shell_util::get_abs_path(cwd, file_name, buf);
        if file_name.is_err() {
            println!("failed to parse file_name, error:{:?}", file_name.unwrap_err());
            return Result::Err(FileError::FilePathIllegal);
        }
        let file_name = file_name.unwrap();
        let file = sys_call::OpenOptions::new().append(true).read(true).open(file_name)?;
        return Result::Ok((command, Option::Some(file)));
    }


    // 如果包含的是>符号，那么就是从头写
    let (command, file_name) = input.split_once(">").unwrap();
    let file_name = file_name.trim();
    let file_name = shell_util::get_abs_path(cwd, file_name, buf);
    if file_name.is_err() {
        println!("failed to parse file_name, error:{:?}", file_name.unwrap_err());
        return Result::Err(FileError::FilePathIllegal);
    }
    let file_name = file_name.unwrap();
    let file = sys_call::OpenOptions::new().write(true).open(file_name)?;
    return Result::Ok((command, Option::Some(file)));
}