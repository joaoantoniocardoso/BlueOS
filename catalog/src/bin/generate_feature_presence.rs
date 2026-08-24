fn main() {
    if let Err(err) = catalog_git::feature_presence::run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
