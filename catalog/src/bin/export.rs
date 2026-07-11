use blueos_catalog::{export_json, Catalog};

fn main() {
    let catalog = Catalog::bootstrap();
    match export_json(&catalog) {
        Ok(json) => println!("{json}"),
        Err(error) => {
            eprintln!("export failed: {error}");
            std::process::exit(1);
        }
    }
}
