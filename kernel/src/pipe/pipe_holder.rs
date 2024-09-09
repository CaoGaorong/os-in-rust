use crate::{filesystem::FileDescriptor, sys_call};

#[derive(Clone, Copy)]
#[derive(Debug)]
pub enum PipeError {
    PipeExhaust,
    FileDescriptorExhaust,
    PipeNotExist
}


#[derive(Debug)]
pub struct PipeReader {
    fd: FileDescriptor,
}

impl PipeReader {
    pub fn new(fd: FileDescriptor) -> Self {
        Self {
            fd,
        }
    }
    /**
     * 从这个管道中读取数据
     */
    pub fn read(&mut self, buff: &mut[u8]) -> usize {
        sys_call::read(self.fd, buff)
    }

    pub fn get_fd(&self) -> FileDescriptor {
        self.fd
    }
}

#[derive(Debug)]
pub struct PipeWriter {
    fd: FileDescriptor,
}

impl PipeWriter {
    pub fn new(fd: FileDescriptor) -> Self {
        Self {
            fd,
        }
    }

    /**
     * 往管道中写入数据
     */
    pub fn write(&mut self, buff: &[u8]) {
        sys_call::write(self.fd, buff)
    }

    pub fn get_fd(&self) -> FileDescriptor {
        self.fd
    }
}