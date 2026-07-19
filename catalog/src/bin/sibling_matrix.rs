use std::process;

use blueos_catalog::tools::sibling_matrix;

fn main() {
    process::exit(sibling_matrix::run());
}
