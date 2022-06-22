async fn hello_world() {
    println!("hello from async wasi");
}

fn main() {
    println!("Hello, world!");
    std::thread::sleep(std::time::Duration::from_secs(1));
    println!("Hello again, world!");

    let future = hello_world();
    futures::executor::block_on(future);
}
