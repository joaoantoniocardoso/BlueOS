use blueos_catalog::catalog::Catalog;
use blueos_catalog::export::export_json;

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
