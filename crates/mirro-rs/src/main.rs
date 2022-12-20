mod tui;

#[tokio::main]
async fn main() {
    let _ = tui::start();
    //    let res = archlinux::archlinux().await.unwrap();
}
