#[cfg(test)]
mod tests {
    use os_in_rust_common::array_deque::ArrayDeque;

    #[test]
    pub fn test_append() {
        let mut array = [0;  10];
        let mut deque = ArrayDeque::new(&mut array);
        deque.append(1);
        deque.append(2);
        deque.append(3);

        deque.iter().for_each(|e| {
            println!("{}", e);
        })
    }
    #[test]
    pub fn test_push() {
        let mut array = [0;  10];
        let mut deque = ArrayDeque::new(&mut array);
        deque.push(1);
        deque.push(2);
        deque.push(3);

        deque.iter().for_each(|e| {
            println!("{}", e);
        })
    }

    #[test]
    pub fn test_mix() {
        let mut array = [0;  10];
        let mut deque = ArrayDeque::new(&mut array);
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
        let mut array = [0;  10];
        let mut deque = ArrayDeque::new(&mut array);
        deque.append(1);
        deque.append(2);
        println!("pop {:?}", deque.pop());
        println!("pop {:?}", deque.pop());
        println!("pop {:?}", deque.pop());
        println!("pop {:?}", deque.pop());
    }

}