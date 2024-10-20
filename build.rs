// build.rs

use std::path::Path;

const WARNING: &str = "//// //////////////////////////////////////////////////////////
/// File added by build.rs, do not modify directly.
/// Modify solana_program/src/event.rs instead.
/// ///////////////////////////////////////////////////////////

";

fn main() {
    let copy_from = Path::new("solana_program/src/event.rs");
    let copy_to = Path::new("src/event/mod.rs");

    let content = std::fs::read_to_string(copy_from);
    if content.is_err() {
        println!(
            "cargo:warning=Failed to read {}: {:#?}",
            copy_from.to_str().unwrap(),
            content.unwrap_err()
        );
        return;
    }
    let content = String::from(WARNING) + &content.unwrap();
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
    println!("cargo::rerun-if-changed=build.rs");
}
