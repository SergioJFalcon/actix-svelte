use std::{env, fs};
use std::process::Command;

fn main() -> std::io::Result<()> {
    let metadata: String = env::var("CARGO_MANIFEST_DIR").unwrap();
    let cargo_toml: String = std::fs::read_to_string(format!("{}/Cargo.toml", metadata)).unwrap();
    
    if let Some(env_section) = cargo_toml.split("[package.metadata.env]").nth(1) {
      for line in env_section.lines().filter(|line| line.contains('=')) {
          let parts: Vec<_> = line.split('=').collect();
          let key: &str = parts[0].trim();
          let value: &str = parts[1].trim().trim_matches('"');
          println!("cargo:rustc-env={}={}", key, value);
      }
    }
    // build client as static files
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=client/src/**");
    println!("cargo:rerun-if-changed=client/static/**");
    println!("cargo:rerun-if-changed=client/svelte.config.js");
    println!("cargo:rerun-if-changed=.env");
    println!("cargo:rerun-if-changed=src/bin/openapi_generator.rs");

    if !check_program_installed("pnpm") {
        panic!("pnpm is not installed! install it first.");
    }

    #[cfg(not(debug_assertions))]
    {  
        let _ = build_client();
    }

     // 2. Generate the openapi.json spec.
    //    We run our `openapi-generator` binary and capture its stdout.
    println!("cargo:warning=Generating OpenAPI spec...");
    let output = Command::new("cargo")
        .args(&["run", "--bin", "openapi-generator"])
        .output()
        .expect("Failed to run openapi-generator binary");

    if !output.status.success() {
        // If the command failed, print the error and stop the build.
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("openapi-generator failed: {}", stderr);
    }
    
    // Write the captured stdout to the `openapi.json` file.
    let spec = String::from_utf8(output.stdout).expect("Invalid UTF-8 from openapi-generator");
    fs::write("openapi.json", spec).expect("Failed to write openapi.json");
    println!("cargo:warning=Successfully generated openapi.json.");

    // 3. Generate the TypeScript client using orval.
    //    We assume `orval` is installed in the `./web` directory.
    //    We run `npx orval` from the root of the project.
    println!("cargo:warning=Generating TypeScript client with orval...");
    
    // Check if orval is installed
    let orval_check = Command::new("npm")
        .args(&["list", "orval"])
        .current_dir("./client") // Run npm list inside the web directory
        .output()
        .expect("Failed to check for orval installation. Is npm installed?");
        
    if !orval_check.status.success() {
        println!("cargo:warning=orval not found or `npm list` failed. Skipping TS client generation.");
        return Ok(());
    }

    let orval_status = Command::new("npx")
        .arg("orval")
        .status() // We just need to know if it succeeded
        .expect("Failed to execute npx orval. Is Node.js/npx installed?");

    if orval_status.success() {
        println!("cargo:warning=Successfully generated TypeScript client at ./web/src/lib/client.ts");
    } else {
        panic!("npx orval command failed. Check orval configuration and ensure openapi.json is valid.");
    }

    Ok(())
}

#[cfg(not(debug_assertions))]
fn build_client() -> std::io::Result<()> {
    let node_modules = std::path::Path::new("client/node_modules");
    if !node_modules.exists() {
        let _exit_status = Command::new("pnpm.cmd")
            .current_dir("client")
            .arg("install")
            .status()?;
    }
    
    // run pnpm build
    let _build_status = Command::new("pnpm.cmd")
        .current_dir("client")
        .arg("build")
        .status().expect("Failed to run pnpm build");
    
    Ok(())
}

#[cfg(windows)]
fn check_program_installed(program: &str) -> bool {
    let output = Command::new("where")
        .arg(program)
        .output()
        .expect("failed to execute process");
    println!("OUTPUT: {:?} - {:?}", output, output.status.success());

    output.status.success()
}

#[cfg(unix)]
fn check_program_installed(program: &str) -> bool {
    let output = Command::new("which")
        .arg(program)
        .output()
        .expect("failed to execute process");
    
    output.status.success()
}