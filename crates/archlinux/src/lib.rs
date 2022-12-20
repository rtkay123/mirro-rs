#[cfg(test)]
mod tests;

pub mod response;

const ARCHLINUX_MIRRORS: &str = "https://archlinux.org/mirrors/status/json/";
const LOCAL_SOURCE: &str = include_str!("../../../sample/archlinux.json");

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}
