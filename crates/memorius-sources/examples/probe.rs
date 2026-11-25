//! Prints one reading per second: `cargo run -p memorius-sources --example probe`.

fn main() {
    let mut probe = memorius_sources::Probe::new();
    println!("title access: {:?}", probe.title_access());
    for _ in 0..5 {
        println!("{:?}", probe.read());
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
