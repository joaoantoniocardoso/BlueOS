use catalog_harness::frontend_smoke::frontend_smoke_targets;
use std::io::{self, Write};

use blueos_catalog::catalog::Catalog;

fn main() {
    let catalog = Catalog::bootstrap();
    let targets = frontend_smoke_targets(&catalog);
    let json = serde_json::to_string_pretty(&targets).expect("serialize smoke targets");
    let mut stdout = io::stdout().lock();
    stdout
        .write_all(json.as_bytes())
        .expect("write smoke targets");
    if !json.ends_with('\n') {
        stdout.write_all(b"\n").expect("write newline");
    }
}
