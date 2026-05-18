use std::env;
use std::io::Read;
use std::process;

fn main() {
    let version = match resolve_version() {
        Some(v) => v,
        None => {
            eprintln!(
                "no claude-code version selected. Run 'ccvm use <version>' first."
            );
            process::exit(1);
        }
    };

    let ccvm_dir = dirs::home_dir()
        .expect("could not determine home directory")
        .join(".ccvm");

    let binary = ccvm_dir.join("versions").join(&version).join("claude.exe");

    if !binary.exists() {
        eprintln!(
            "claude-code version {} is not installed. Run 'ccvm install {}' first.",
            version, version
        );
        process::exit(1);
    }

    let args: Vec<String> = env::args().skip(1).collect();
    let status = process::Command::new(&binary)
        .args(&args)
        .status()
        .unwrap_or_else(|e| {
            eprintln!("failed to execute {}: {}", binary.display(), e);
            process::exit(1);
        });

    process::exit(status.code().unwrap_or(1));
}

fn resolve_version() -> Option<String> {
    // 1. Look for .ccvmrc in current dir and up to root
    if let Some(v) = find_ccvmrc() {
        return Some(v);
    }

    // 2. Fall back to ~/.ccvm/current
    let current_file = dirs::home_dir()
        .expect("could not determine home directory")
        .join(".ccvm")
        .join("current");

    if current_file.exists() {
        let mut content = String::new();
        std::fs::File::open(&current_file)
            .ok()?
            .read_to_string(&mut content)
            .ok()?;
        let v = content.trim().to_string();
        if !v.is_empty() {
            return Some(v);
        }
    }

    None
}

fn find_ccvmrc() -> Option<String> {
    let mut dir = env::current_dir().ok()?;

    loop {
        let rc_path = dir.join(".ccvmrc");
        if rc_path.exists() {
            let mut content = String::new();
            std::fs::File::open(&rc_path)
                .ok()?
                .read_to_string(&mut content)
                .ok()?;
            let v = content.trim().to_string();
            if !v.is_empty() {
                return Some(v);
            }
        }

        if !dir.pop() {
            break; // reached root
        }
    }

    None
}
