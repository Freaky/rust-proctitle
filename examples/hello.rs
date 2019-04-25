use proctitle;

fn main() {
    let mut i = 0;
    loop {
        proctitle::set_title(format!("Hello, world, {}", i));
        i += 1;
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}
