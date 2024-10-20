// build.rs

use std::path::Path;

const WARNING: &str = "//// //////////////////////////////////////////////////////////
/// File added by build.rs, do not modify directly.
/// Modify the corresponding file in solana_program/ instead.
/// ///////////////////////////////////////////////////////////

";

fn main() {
    let copy_files = [
        ("solana_program/src/event.rs", "src/event/mod.rs"),
        (
            "solana_program/src/instruction.rs",
            "src/solana_client/instruction.rs",
        ),
    ];

    for i in copy_files {
        let copy_from = Path::new(i.0);
        let copy_to = Path::new(i.1);

        let content = std::fs::read_to_string(copy_from);
        if content.is_err() {
            println!(
                "cargo:warning=Failed to read {}: {:#?}",
                copy_from.to_str().unwrap(),
                content.unwrap_err()
            );
            return;
        }
        let content =
            (String::from(WARNING) + &content.unwrap()).replace("solana_program", "solana_sdk");
        let res = std::fs::write(copy_to, content);
        if res.is_err() {
            println!(
                "cargo:warning=Failed to write {}: {:#?}",
                copy_to.to_str().unwrap(),
                res.unwrap_err()
            );
            return;
        }
        println!("cargo::rerun-if-changed={}", copy_from.to_str().unwrap());
    }

    println!("cargo::rerun-if-changed=build.rs");
}
