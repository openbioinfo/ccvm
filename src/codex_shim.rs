use std::env;
use std::io::Read;
use std::process;

mod platform;

fn main() {
    let version = match resolve_version() {
        Some(v) => v,
        None => {
            eprintln!("no codex version selected. Run 'ccvm codex use <version>' first.");
            process::exit(1);
        }
    };

    let ccvm_dir = dirs::home_dir()
        .expect("could not determine home directory")
        .join(".ccvm");

    let target_triple = platform::codex_target_triple().unwrap_or_else(|e| {
        eprintln!("{}", e);
        process::exit(1);
    });
    let binary = ccvm_dir
        .join("codex")
        .join("versions")
        .join(&version)
        .join("vendor")
        .join(target_triple)
        .join("bin")
        .join(platform::executable_name("codex"));

    if !binary.exists() {
        eprintln!(
            "codex version {} is not installed. Run 'ccvm codex install {}' first.",
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
    if let Some(v) = find_codexvmrc() {
        return Some(v);
    }

    let current_file = dirs::home_dir()
        .expect("could not determine home directory")
        .join(".ccvm")
        .join("codex")
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

fn find_codexvmrc() -> Option<String> {
    let mut dir = env::current_dir().ok()?;

    loop {
        let rc_path = dir.join(".codexvmrc");
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
            break;
        }
    }

    None
}
