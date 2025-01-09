use std::env;
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

    if !check_program_installed("pnpm") {
        panic!("pnpm is not installed! install it first.");
    }

    #[cfg(not(debug_assertions))]
    {  
        let _ = build_client();
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