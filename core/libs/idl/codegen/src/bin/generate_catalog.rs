//! Regenerates vendored catalog schema lookup (`schema_catalog.rs`, `catalog.ts`).

use std::{env, path::PathBuf, process::ExitCode};

use blueos_idl_codegen::{flag_value, generate_catalog_outputs, idl_root, write_typescript_index};

fn main() -> ExitCode {
    if let Err(error) = run() {
        eprintln!("{error}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn run() -> Result<(), blueos_idl_codegen::CodegenError> {
    run_with_arguments(env::args().collect())
}

fn run_with_arguments(arguments: Vec<String>) -> Result<(), blueos_idl_codegen::CodegenError> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let idl_root = idl_root(
        &manifest_dir,
        flag_value(&arguments, "--idl-root").map(PathBuf::from),
    )?;
    let generated_output = flag_value(&arguments, "--generated-output")
        .map(PathBuf::from)
        .unwrap_or_else(|| idl_root.join("src/generated"));
    let typescript_output = flag_value(&arguments, "--typescript-output")
        .map(PathBuf::from)
        .unwrap_or_else(|| idl_root.join("typescript"));
    generate_catalog_outputs(&idl_root, &generated_output, &typescript_output)?;
    write_typescript_index(&typescript_output)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{env, fs};

    use super::*;

    #[test]
    fn run_with_temp_outputs() {
        let output = env::temp_dir().join(format!(
            "blueos-idl-codegen-catalog-bin-{}",
            std::process::id()
        ));
        _ = fs::remove_dir_all(&output);
        fs::create_dir_all(&output).expect("temp dir");
        let typescript = output.join("typescript");
        let arguments = vec![
            "blueos-idl-codegen-catalog".to_owned(),
            "--generated-output".to_owned(),
            output.display().to_string(),
            "--typescript-output".to_owned(),
            typescript.display().to_string(),
        ];
        run_with_arguments(arguments).expect("catalog bin");
        assert!(output.join("schema_catalog.rs").is_file());
        assert!(typescript.join("catalog.ts").is_file());
        _ = fs::remove_dir_all(output);
    }
}
