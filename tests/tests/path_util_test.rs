mod tests {
    use kernel::shell::shell_util;


    #[test]
    pub fn abs_path_test() {
        let mut abs_path = [0; 100];
        let result = shell_util::get_abs_path("/Users/jackson", ".../code/abc", &mut abs_path);
        println!("res:{:?}" , result);
    }


    #[test]
    pub fn tests1() {
        let path = "/Users/jackson";
        let mut parts = path.split("/"); // 将路径分割成数组形式的片段
        while let Option::Some(entry) = parts.next() {
            // 检查迭代器是否还有下一个元素，并将其值取出进行解构
            println!("entry: {}", entry); // 输出entry的值
        }
    }


    #[test]
    fn test_split_once() {

        let s = "ls -alh -s".split_once(" ");
        println!("s: {:?}", s);
    }
}

