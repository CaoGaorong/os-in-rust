mod test {
    use lazy_static::lazy_static;
    use std::{
        env,
        fs::{self, File, OpenOptions},
        io::{BufReader, Read, Seek, SeekFrom, Write},
    };

    lazy_static! {
        // 当前工作目录的路径
        static ref CURRENT_DIR: String = env::current_dir().unwrap().to_str().unwrap().to_string();
    }

    /**
     * 场景0：测试类测试
     */
    #[test]
    pub fn test() {
        println!("rust up !!!");
        println!("cwd: {:?}", CURRENT_DIR.as_str());
        let s = CURRENT_DIR.to_owned() + "/resource/a.txt";
        println!("cwd: {:?}", s);
    }

    /**
     * 场景1：创建一个文件
     * - 普通创建
     * - 创建一个已存在的文件（create和create_new表现不同）
     */
    #[test]
    pub fn create_file() {
        let path = CURRENT_DIR.to_owned() + "/resource/new_created.txt";
        let mut file = File::create(path.to_owned()).unwrap();
        let mut buffer = [0; 5]; // 设置缓冲区大小为 1024 字节，根据需求调整
        println!("{:?}", file.read(&mut buffer));
        // 使用File::create
        println!("{:?}", File::create(path.to_owned())); // Ok(File { fd: 3, path: "/resource/new_created.txt", read: false, write: true })
        // 重复使用File::create
        println!("{:?}", File::create(path.to_owned())); // Ok(File { fd: 3, path: "/resource/new_created.txt", read: false, write: true })
        // 使用File::create_new创建一个已存在的文件
        println!("{:?}", File::create_new(path.to_owned())); // Err(Os { code: 17, kind: AlreadyExists, message: "File exists" })
    }

    /**
     * 场景2：正常打开一个文件。打开两次得到不同的文件描述符
     */
    #[test]
    pub fn open_file() {
        let path = CURRENT_DIR.to_owned() + "/resource/a.txt";
        // 打开一个文件
        let file1 = File::open(path.to_owned());
        // 再打开这个文件
        let file2 = File::open(path.to_owned());
        println!("{:?}", file1); // Ok(File { fd: 3, path: "/resource/a.txt", read: true, write: false })
        println!("{:?}", file2); // Ok(File { fd: 4, path: "/resource/a.txt", read: true, write: false })
    }

    /**
     * 场景3：打开一个不存在的文件
     */
    #[test]
    pub fn read_not_found() {
        let path = CURRENT_DIR.to_owned() + "/resource/404.txt";
        let file = File::open(path);
        println!("{:?}", file); // Err(Os { code: 2, kind: NotFound, message: "No such file or directory" })
    }
    /**
     * 场景4：打开一个是文件夹的文件
     */
    #[test]
    pub fn read_mismatch() {
        let path = CURRENT_DIR.to_owned() + "/resource/static";
        let file = File::open(path);
        println!("{:?}", file); // Ok(File { fd: 3, path: "/resource/static", read: true, write: false })
    }


    /**
     * 场景5：读取出一个文件的全部内容，直到文件结束
     */
    #[test]
    pub fn read_all_from_file() {
        let path = CURRENT_DIR.to_owned() + "/resource/a.txt";
        let mut file = File::open(path).unwrap();
        let mut buffer = [0; 5]; // 设置缓冲区大小为 1024 字节，根据需求调整

        loop {
            let bytes_read = file.read(&mut buffer).unwrap();

            if bytes_read == 0 {
                break; // 文件读取完毕
            }

            // 在这里处理每次读取的数据，例如打印到控制台
            println!("{}", String::from_utf8_lossy(&buffer[..bytes_read]));
        }
    }

    /**
     * 场景6：从某个文件的偏移量开始读取
     */
    #[test]
    pub fn read_from_offset() {
        let path = CURRENT_DIR.to_owned() + "/resource/a.txt";
        let mut file = File::open(path).unwrap();
        // 指定文件的偏移量，这里从文件开头的2个字节开始读取
        file.seek(std::io::SeekFrom::Start(2))
            .expect("seek file error");
        let mut buffer = [0; 1024]; // 设置缓冲区大小为 1024 字节，根据需求调整

        loop {
            let bytes_read = file.read(&mut buffer).unwrap();

            if bytes_read == 0 {
                break; // 文件读取完毕
            }

            // 在这里处理每次读取的数据，例如打印到控制台
            println!("{}", String::from_utf8_lossy(&buffer[..bytes_read]));
        }
    }

    /**
     * 场景6：从某个文件的偏移量开始读取
     */
    #[test]
    pub fn read_file_infinity() {
        let path = CURRENT_DIR.to_owned() + "/resource/a.txt";
        let mut file = File::open(path).unwrap();
        let mut buffer = [0; 1024]; // 设置缓冲区大小，根据需要调整
        let mut total_bytes_read = 0;

        let mut reset_cnt = 0;
        loop {
            let bytes_read = file.read(&mut buffer).expect("Unable to read from file");
            
            if total_bytes_read >= 100 {
                break;
            }
            if reset_cnt > 3 {
                break;
            }

            if bytes_read == 0 {
                reset_cnt += 1;
                println!("reset file ");
                // 已读到文件末尾，重新定位到文件开头
                file.seek(SeekFrom::Start(0))
                    .expect("Unable to seek to start of file");
                total_bytes_read = 0;
                continue;
            }
            

            // 在这里处理每次读取的数据
            total_bytes_read += bytes_read;

            // 示例：打印读取的数据
            let data = std::str::from_utf8(&buffer[..bytes_read]).expect("Invalid UTF-8 data");
            println!("Read {} bytes: {}", bytes_read, data);
        }
    }

    /**
     * 场景7：写入某个文件
     */
    #[test]
    pub fn write_to_file() {
        let path = CURRENT_DIR.to_owned() + "/resource/a.txt";
        let mut file = File::open(path).unwrap();
        println!("file = {:?}", file);
        let result = file.write_all("hello".as_bytes());
        println!("{:?}", result); // 因为只是open了文件，因此无法写入： Err(Os { code: 9, kind: Uncategorized, message: "Bad file descriptor" })
    }


    


    /**
     * 场景8：追加写入一个文件
     */
    #[test]
    pub fn append_file() {
        let path = CURRENT_DIR.to_owned() + "/resource/a.txt";
        let mut file  = OpenOptions::new().append(true).open(path).expect("failed to open file");
        println!("file = {:?}", file); // File { fd: 3, path: "/resource/a.txt", read: false, write: true }
        let result  = file.write("jjjjj".as_bytes());
        println!("{:?}", result);
    }

    #[test]
    pub fn write_then_read() {
        let path = CURRENT_DIR.to_owned() + "/resource/a.txt";
        let mut file  = OpenOptions::new().append(true).open(path).expect("failed to open file");
        println!("file = {:?}", file); // File { fd: 3, path: "/resource/a.txt", read: false, write: true }
        let result  = file.write("jjjjj".as_bytes());
        file.seek(SeekFrom::Start(0));
        let mut buff = [0; 20];
        let read_res = file.read(&mut buff);
        println!("read res: {:?}", read_res);
        let string = std::str::from_utf8(&buff);
        println!("s: {}", string.unwrap());

    }


    /**
     * 场景9：写入的是一个目录项
     */
    #[test]
    pub fn write_mismatch() {
        let path = CURRENT_DIR.to_owned() + "/resource/static";
        let mut file = OpenOptions::new().append(true).open(path).unwrap();
        println!("file = {:?}", file);
        let result = file.write_all("hello".as_bytes());
        println!("{:?}", result); // Os { code: 21, kind: IsADirectory, message: "Is a directory" }
    }
    
    /**
     * 场景10：指定偏移量，写入文件
     */
    #[test]
    pub fn write_file_by_offset() {
        let path = CURRENT_DIR.to_owned() + "/resource/a.txt";
        let mut file = OpenOptions::new().write(true).open(path).expect("failed to open file");
        println!("file = {:?}", file);
        // 跳过100个字节
        file.seek(SeekFrom::Start(100)).expect("failed to seek file");
        // 写入文件
        file.write("skip byte".as_bytes()).expect("failed to write file");
    }

    /**
     * 删除文件。普通文件
     */
    #[test]
    pub fn delete_file() {
        let path = CURRENT_DIR.to_owned() + "/resource/test.txt";
        println!("{:?}", fs::remove_file(path.to_owned())); // Ok(())
        println!("{:?}", fs::remove_file(path.to_owned())); // Err(Os { code: 2, kind: NotFound, message: "No such file or directory" })
    }

    /**
     * 删除文件。是一个文件夹，但是用了fs::remove_file()函数
     */
    #[test]
    pub fn delete_mismatch() {
        let path = CURRENT_DIR.to_owned() + "/resource/static";
        println!("{:?}", fs::remove_file(path.to_owned())); // Err(Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" })
    }
    
}
