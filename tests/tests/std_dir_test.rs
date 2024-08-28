pub mod test {
    use std::{env, fs};

    use lazy_static::lazy_static;

    
    lazy_static! {
        // 当前工作目录的路径
        static ref CURRENT_DIR: String = env::current_dir().unwrap().to_str().unwrap().to_string();
    }



    /**
     * 场景1：创建一个目录、重复创建一个目录
     */
    #[test]
    pub fn create_dir() {
        let path = CURRENT_DIR.to_owned() + "/resource/static";
        println!("{:?}", fs::create_dir(path.to_owned()));
        println!("{:?}", fs::create_dir_all(path.to_owned())); // Err(Os { code: 17, kind: AlreadyExists, message: "File exists" })
    }

    /**
     * 场景2：递归创建目录
     */
    #[test]
    pub fn create_dir_recursive() {
        let path = CURRENT_DIR.to_owned() + "/resource/static/folder1/folder2/folder3";
        // Err(Os { code: 2, kind: NotFound, message: "No such file or directory" })
        println!("{:?}", fs::create_dir(path.to_owned()));
        // Ok(())
        println!("{:?}", fs::create_dir_all(path.to_owned()));
    }

    /**
     * 场景2：创建一个目录，但是这个名字已经被一个文件用过了
     */
    #[test]
    pub fn create_mistake() {
        let path = CURRENT_DIR.to_owned() + "/resource/a.txt";
        println!("{:?}", fs::create_dir(path.to_owned())); // Err(Os { code: 17, kind: AlreadyExists, message: "File exists" })
    }
    

    /**
     * 场景3：打开一个目录
     */
    #[test]
    pub fn open_dir() {
        let path = CURRENT_DIR.to_owned() + "/resource/static";
        let dir = fs::read_dir(path);
        println!("{:?}", dir); // Ok(ReadDir("/xxx/rust/rust-learning/resource/static"))
    }

    /**
     * 场景4：打开一个目录，但是这个名字却是一个文件
     */
    #[test]
    pub fn open_mismatch() {
        let path = CURRENT_DIR.to_owned() + "/resource/a.txt";
        let dir = fs::read_dir(path);
        println!("{:?}", dir); // Err(Os { code: 20, kind: NotADirectory, message: "Not a directory" })
    }

    

    /**
     * 场景5：读取一个目录中的所有目录项
     */
    #[test]
    pub fn read_entry_from_dir() {
        let path = CURRENT_DIR.to_owned() + "/resource/static";
        let dir = fs::read_dir(path).expect("failed to open dir");
        
        for dir_entry in dir {
            if dir_entry.is_err() {
                println!("failed to read dir entry, {:?}", dir_entry.unwrap_err());
                continue;
            }
            let dir_entry = dir_entry.unwrap();
            println!("file_name: {:?}, file_type:{:?}, path:{:?}", dir_entry.file_name(), dir_entry.file_type(), dir_entry.path());
        }
    }

    /**
     * 场景6：删除一个目录
     */
    #[test]
    pub fn delete_dir() {
        let path = CURRENT_DIR.to_owned() + "/resource/";
        println!("{:?}", fs::remove_dir(path.to_owned())); // Ok(())
        // println!("{:?}", fs::remove_dir(path.to_owned())); // Err(Os { code: 2, kind: NotFound, message: "No such file or directory" })
    }
}