use std::process::Command;

// Example custom build script.
fn main() {
    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo::rerun-if-changed=src/landing/input.css");

    Command::new("npx").args([
        "@tailwindcss/cli",
        "-i",
        "./src/landing/input.css",
        "-o",
        "./src/static/responder.css",
    ]).status().expect("Failed running Tailwind CLI");
}
