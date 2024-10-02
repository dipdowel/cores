use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    // Check the target OS environment variable provided by Cargo
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();

    if target_os != "linux" {
        println!("\n==============================================================");
        println!("  [!] This program can only be compiled on/for Linux targets");
        println!("==============================================================\n");
        std::process::exit(1);
    }


    // Get the current date using `date` from the shell
    let output = Command::new("date")
        .arg("+%d %B %Y")
        .output()
        .expect("failed to execute process");

    let build_date = String::from_utf8(output.stdout).unwrap();

    // Set environment variable for build date
    println!("cargo:rustc-env=BUILD_DATE={}", build_date.trim());

}

