use proctitle;

fn main() {
    proctitle::set_title("Hello, world");
    println!("Title set, go look while I gobble down CPU");
    loop {}
}
