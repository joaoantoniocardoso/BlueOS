use blueos_catalog::{calibration_smoke_targets, Catalog};
use std::io::{self, Write};

fn main() {
    let catalog = Catalog::bootstrap();
    let targets = calibration_smoke_targets(&catalog);
    let json = serde_json::to_string_pretty(&targets).expect("serialize smoke targets");
    let mut stdout = io::stdout().lock();
    stdout
        .write_all(json.as_bytes())
        .expect("write smoke targets");
    if !json.ends_with('\n') {
        stdout.write_all(b"\n").expect("write newline");
    }
}
