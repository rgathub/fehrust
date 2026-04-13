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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_file() -> FehFile {
        FehFile::new(PathBuf::from("C:\\images\\photo.jpg"))
    }

    #[test]
    fn expand_action_f() {
        let f = test_file();
        let result = expand_action("echo %f", &f, 0, 1);
        assert!(result.contains("C:\\images\\photo.jpg"));
    }

    #[test]
    fn expand_action_n() {
        let f = test_file();
        let result = expand_action("%n", &f, 0, 1);
        assert_eq!(result, "photo.jpg");
    }

    #[test]
    fn expand_action_u() {
        let f = test_file();
        let result = expand_action("%u", &f, 3, 10);
        assert_eq!(result, "4");
    }

    #[test]
    fn expand_action_l() {
        let f = test_file();
        let result = expand_action("%l", &f, 0, 10);
        assert_eq!(result, "10");
    }

    #[test]
    fn expand_action_percent() {
        let f = test_file();
        let result = expand_action("%%", &f, 0, 1);
        assert_eq!(result, "%");
    }

    #[test]
    fn expand_action_mixed() {
        let f = test_file();
        let result = expand_action("cp %f /dest/%n", &f, 0, 1);
        assert_eq!(result, "cp C:\\images\\photo.jpg /dest/photo.jpg");
    }
}
