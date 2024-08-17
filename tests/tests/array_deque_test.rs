#[cfg(test)]
mod tests {
    use os_in_rust_common::array_deque::ArrayDeque;

    #[test]
    pub fn test_append() {
        let mut deque = ArrayDeque::new([0; 10]);
        deque.append(1);
        deque.append(2);
        deque.append(3);

        deque.iter().for_each(|e| {
            println!("{}", e);
        })
    }
    #[test]
    pub fn test_push() {
        let mut deque = ArrayDeque::new([0;  10]);
        deque.push(1);
        deque.push(2);
        deque.push(3);

        deque.iter().for_each(|e| {
            println!("{}", e);
        })
    }

    #[test]
    pub fn test_mix() {
        let mut deque = ArrayDeque::new([0;  10]);
        deque.append(1);
        println!("pop {:?}", deque.pop());
        deque.append(2);
        println!("pop {:?}", deque.pop());
        deque.iter().for_each(|e| {
            println!("{}", e);
        })
    }

    #[test]
    pub fn pop_all() {
        let mut deque = ArrayDeque::new([0;  10]);
        deque.append(1);
        deque.append(2);
        deque.append(3);
        println!("pop {:?}", deque.pop_last());
        println!("pop {:?}", deque.pop());
        println!("pop {:?}", deque.pop());
        println!("pop {:?}", deque.pop());
    }

    #[test]
    pub fn test_util() {
        let mut arr = [0; 5];
        let bytes = 'c'.encode_utf8(&mut arr).as_bytes();
        for byte in bytes {
            println!("{}", byte);
        }


    }

}