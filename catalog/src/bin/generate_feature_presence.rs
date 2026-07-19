fn main() {
    if let Err(err) = blueos_catalog::tools::feature_presence::run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
