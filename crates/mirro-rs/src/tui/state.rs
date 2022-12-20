#[cfg(feature = "archlinux")]
use archlinux::ArchLinux;

use anyhow::Result;

#[derive(Default)]
pub struct App {
    pub show_popup: bool,
    #[cfg(feature = "archlinux")]
    pub mirrors: Option<ArchLinux>,
}

impl App {
    #[cfg(feature = "archlinux")]
    pub fn new() -> Self {
        Self {
            show_popup: true,
            mirrors: None,
        }
    }

    #[cfg(feature = "archlinux")]
    pub async fn initialise(&mut self) -> Result<()> {
        println!("starting");
        use anyhow::bail;
        match archlinux::archlinux().await {
            Ok(mirrors) => self.mirrors = Some(mirrors),
            Err(e) => bail!("{e}"),
        }
        println!("done");
        Ok(())
    }
}
