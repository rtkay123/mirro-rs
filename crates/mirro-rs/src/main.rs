mod tui;

#[tokio::main]
async fn main() {
    let _ = tui::start().await;
    //    let res = archlinux::archlinux().await.unwrap();
}
