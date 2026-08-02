//! Image loader integration tests (Windows only)
//! These tests verify that the image loading pipeline works correctly

#[cfg(target_os = "windows")]
mod tests {
    use assert_cmd::Command;
    use std::path::PathBuf;

    #[test]
    fn test_can_load_png() {
        let path = PathBuf::from("tests/fixtures/test_1x1.png");
        let expected_path = path.display().to_string();

        let mut cmd = Command::cargo_bin("fehrust").unwrap();
        cmd.arg("--loadable")
            .arg(&path)
            .assert()
            .success()
            .stdout(predicates::str::contains(expected_path));
    }

    #[test]
    fn test_filesize_reasonable() {
        use std::fs;

        let path = PathBuf::from("tests/fixtures/test_1x1.png");
        let metadata = fs::metadata(&path).expect("Failed to get file metadata");
        let size = metadata.len();

        // 70 bytes for a 1x1 PNG is reasonable (67 bytes header + minimal data)
        assert!(size > 50 && size < 200);
    }

    #[test]
    fn test_directory_exists() {
        let fixtures_path = PathBuf::from("tests/fixtures");
        assert!(fixtures_path.exists());
        assert!(fixtures_path.is_dir());
    }
}

// The WIC loading pipeline is exercised through the CLI's --loadable mode.
