use libc;
use std::fs;
use std::process::{Command, Stdio};
use std::io::Write;

const RAPL_PATH: &str = "/sys/class/powercap/intel-rapl:0";
const PL0_PATH: &str = "/sys/class/powercap/intel-rapl:0/constraint_0_power_limit_uw";
const PL1_PATH: &str = "/sys/class/powercap/intel-rapl:0/constraint_1_power_limit_uw";

fn check_support() {
    if !std::path::Path::new(RAPL_PATH).exists() {
        eprintln!("❌ Intel RAPL not supported on this system.");
        std::process::exit(1);
    }
}

fn read_power_limit(path: &str) -> i64 {
    fs::read_to_string(path)
        .expect("Failed to read power limit")
        .trim()
        .parse()
        .expect("Invalid power limit value")
}

fn write_power_limit(path: &str, watts: f64) {
    let microwatts = (watts * 1_000_000.0) as u64;
    // Check if we're root — if so, write directly
    let is_root = unsafe { libc::getuid() } == 0;
    if is_root {
        fs::write(path, format!("{}\n", microwatts)).expect("Failed to write power limit");
    } else {
        // Fallback to sudo for manual CLI usage
        let mut cmd = Command::new("sudo")
            .arg("tee")
            .arg(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("Failed to spawn sudo tee");

        if let Some(stdin) = cmd.stdin.take() {
            let mut stdin = stdin;
            writeln!(stdin, "{}", microwatts).expect("Failed to write to stdin");
        }
        cmd.wait().expect("sudo tee failed");
    }
}

fn show_status() {
    check_support();

    let pl0 = read_power_limit(PL0_PATH) as f64 / 1_000_000.0;
    let pl1 = read_power_limit(PL1_PATH) as f64 / 1_000_000.0;

    println!();
    println!("⚙️  Intel RAPL Power Limits:");
    println!("  🔹 PL0 (Short-term Power Limit): {:.2} W", pl0);
    println!("  🔹 PL1 (Long-term Power Limit):  {:.2} W", pl1);

    match Command::new("upower")
        .args(&["-i", "/org/freedesktop/UPower/devices/battery_BAT0"])
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains("energy-rate") {
                    println!("\n🔋 Battery Info:");
                    println!("  {}", line.trim());
                    break;
                }
            }
        }
        Err(_) => {
            println!("\n🔋 Battery info unavailable (maybe desktop or no UPower).");
        }
    }
    println!();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: underwatt <status|set>");
        std::process::exit(1);
    }

    match args[1].as_str() {
        "status" => show_status(),

        "set" => {
            check_support();

            let mut pl0: Option<f64> = None;
            let mut pl1: Option<f64> = None;

            let mut i = 2;
            while i < args.len() {
                match args[i].as_str() {
                    "--pl0" if i + 1 < args.len() => {
                        pl0 = Some(args[i + 1].parse().expect("Invalid PL0 value"));
                        i += 2;
                    }
                    "--pl1" if i + 1 < args.len() => {
                        pl1 = Some(args[i + 1].parse().expect("Invalid PL1 value"));
                        i += 2;
                    }
                    _ => i += 1,
                }
            }

            if pl0.is_none() && pl1.is_none() {
                eprintln!("❗ Please specify at least one of --pl0 or --pl1");
                std::process::exit(1);
            }

            if let Some(w) = pl0 {
                println!("Setting PL0 (Short-term) → {:.2}W", w);
                write_power_limit(PL0_PATH, w);
            }
            if let Some(w) = pl1 {
                println!("Setting PL1 (Long-term) → {:.2}W", w);
                write_power_limit(PL1_PATH, w);
            }
        }

        _ => {
            eprintln!("Unknown command: {}", args[1]);
            std::process::exit(1);
        }
    }
}
