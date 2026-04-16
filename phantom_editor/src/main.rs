fn main() {
    let args: Vec<String> = std::env::args().collect();
    println!("path: {}", args.get(1).unwrap());
}
