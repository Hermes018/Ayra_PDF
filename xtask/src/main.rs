use std::env;
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "doctor" {
        let binaries = vec![
            "rustc",
            "cargo",
            "pnpm",
            "qpdf",
            "pdftotext",
            "tesseract",
            "soffice",
        ];

        let mut all_found = true;
        for bin in binaries {
            match Command::new("which").arg(bin).output() {
                Ok(output) if output.status.success() => {
                    println!("✅ Found {}", bin);
                }
                _ => {
                    println!("❌ Missing {}", bin);
                    all_found = false;
                }
            }
        }

        if all_found {
            println!("Doctor check passed. Gate green.");
            std::process::exit(0);
        } else {
            println!("Doctor check failed. Some dependencies are missing.");
            std::process::exit(1);
        }
    } else {
        println!("Usage: cargo xtask doctor");
        std::process::exit(1);
    }
}
