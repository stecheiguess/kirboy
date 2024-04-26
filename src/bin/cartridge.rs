fn main() {
    // Read boot-rom file
    let bytes = std::fs::read("test.gb").unwrap();

    let mut title = String::new();

    for &byte in bytes[0x0134..0x0143].iter() {
        title.push(byte as char);
    }
    for &byte in bytes[0x0144..0x0146].iter() {
        println!("{:?}", byte as char);
    }

    println!("{}", title)
}
