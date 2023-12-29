fn main() {
    let s = "username: me\r\ntimestamp: today\r\n\r\n\r\n";
    for p in s.split("\r\n") {
        println!("{:?}", p)
    }

    // let (head, tail) = s.split_once("\r\n\r\n").unwrap();
    // println!("{:?}", head);
    // println!("{:?}", tail);
}