mod dependencies;
mod solver;
mod user_library;
use std::path::PathBuf;

pub struct RustRunner {
    root_dir: PathBuf,
}

impl RustRunner {
    pub fn new(root_dir: PathBuf) -> RustRunner {
        RustRunner { root_dir }
    }
    fn src_dir(&self) -> PathBuf {
        self.root_dir.join("src")
    }
}

fn cargo_contents() -> String {
    r#"[package]
name = "aocsuite-solution-rust"
version = "0.1.0"
edition = "2024"

[dependencies]
serde_json="1.0.140"
serde = { version = "1.0.219", features = ["derive"]}
"#
    .to_string()
}
