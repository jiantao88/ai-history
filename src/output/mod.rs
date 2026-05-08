pub mod human;
pub mod json;
pub mod markdown;
pub mod prompt;

use std::io::IsTerminal;

pub fn is_tty() -> bool {
    std::io::stdout().is_terminal()
}
