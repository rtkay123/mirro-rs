use std::fmt::Display;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Filter {
    Https,
    Http,
    Rsync,
    InSync,
}

impl Display for Filter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Filter::Https => "https",
                Filter::Http => "http",
                Filter::Rsync => "rsync",
                Filter::InSync => "in-sync",
            }
        )
    }
}
