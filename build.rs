fn main() {
    // Check the target OS environment variable provided by Cargo
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();

    if target_os != "linux" {
        println!("\n==============================================================");
        println!("  [!] This program can only be compiled on/for Linux targets");
        println!("==============================================================\n");
        std::process::exit(1);

    }
}

