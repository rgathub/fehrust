use crate::filelist::FehFile;

/// Execute a custom action command, expanding format specifiers.
///
/// Specifiers: %f=filepath, %n=filename, %u=index (1-based), %l=total
pub fn execute_action(action_str: &str, file: &FehFile, index: usize, total: usize) {
    let expanded = expand_action(action_str, file, index, total);
    let parts: Vec<&str> = expanded.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }

    let program = parts[0];
    let args = &parts[1..];

    // Run asynchronously so we don't block the UI
    let program = program.to_string();
    let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    std::thread::spawn(
        move || match std::process::Command::new(&program).args(&args).spawn() {
            Ok(mut child) => {
                let _ = child.wait();
            }
            Err(e) => {
                eprintln!("fehrust: failed to execute action '{}': {}", program, e);
            }
        },
    );
}

fn expand_action(action_str: &str, file: &FehFile, index: usize, total: usize) -> String {
    let mut result = String::with_capacity(action_str.len() * 2);
    let mut chars = action_str.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            match chars.next() {
                Some('f') => result.push_str(&file.path.to_string_lossy()),
                Some('n') => result.push_str(&file.name),
                Some('u') => result.push_str(&(index + 1).to_string()),
                Some('l') => result.push_str(&total.to_string()),
                Some('%') => result.push('%'),
                Some(other) => {
                    result.push('%');
                    result.push(other);
                }
                None => result.push('%'),
            }
        } else {
            result.push(c);
        }
    }

    result
}
