use std::fmt::Display;

use crate::cli::Protocol;

//#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, ValueEnum, Deserialize)]
//#[serde(rename_all = "lowercase")]
//pub enum Protocol {
//    Https,
//    Http,
//    Rsync,
//    #[value(skip)]
//    InSync,
//    #[value(skip)]
//    Ipv4,
//    #[value(skip)]
//    Ipv6,
//    #[value(skip)]
//    Isos,
//}

impl From<archlinux::Protocol> for Protocol {
    fn from(value: archlinux::Protocol) -> Self {
        match value {
            archlinux::Protocol::Rsync => Self::Rsync,
            archlinux::Protocol::Http => Self::Http,
            archlinux::Protocol::Https => Self::Https,
            archlinux::Protocol::Ftp => Self::Ftp,
        }
    }
}

impl Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Protocol::Https => "https",
                Protocol::Http => "http",
                Protocol::Rsync => "rsync",
                Protocol::Ftp => "ftp",
                Protocol::InSync => "in-sync",
                Protocol::Ipv4 => "ipv4",
                Protocol::Ipv6 => "ipv6",
                Protocol::Isos => "isos",
            }
        )
    }
}
