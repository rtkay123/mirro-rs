#[tokio::main]
async fn main() {
    let res = archlinux::archlinux().await.unwrap();
    println!("{res:#?}");
    println!("Hello, world!");
}
