fn main() {
    if let Err(error) = aocsuite_tui::run() {
        eprintln!("encountered error: {error}");
        std::process::exit(1);
    }
}
