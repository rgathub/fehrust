use super::*;
use std::path::{Path, PathBuf};

fn make_list(names: &[&str]) -> FileList {
    let files: Vec<FehFile> = names
        .iter()
        .map(|n| FehFile::new(PathBuf::from(n)))
        .collect();
    let mut list = FileList::from_single(files[0].clone());
    *list.files_mut() = files;
    list
}

fn make_empty_list() -> FileList {
    let mut list = FileList::from_single(FehFile::new(PathBuf::from("dummy.jpg")));
    list.remove_current();
    list
}

// ── Navigation ──────────────────────────────────────────────

#[test]
fn from_single() {
    let f = FehFile::new(PathBuf::from("photo.jpg"));
    let list = FileList::from_single(f);
    assert_eq!(list.len(), 1);
    assert!(list.current().is_some());
    assert_eq!(list.current().unwrap().name, "photo.jpg");
}

#[test]
fn current_empty() {
    let list = make_empty_list();
    assert!(list.current().is_none());
    assert!(list.is_empty());
}

#[test]
fn next_advances() {
    let mut list = make_list(&["a.jpg", "b.jpg", "c.jpg"]);
    assert_eq!(list.current_index(), 0);
    assert!(list.next());
    assert_eq!(list.current_index(), 1);
}

#[test]
fn next_wraps() {
    let mut list = make_list(&["a.jpg", "b.jpg", "c.jpg"]);
    list.next();
    list.next();
    assert_eq!(list.current_index(), 2);
    list.next();
    assert_eq!(list.current_index(), 0);
}

#[test]
fn prev_wraps() {
    let mut list = make_list(&["a.jpg", "b.jpg", "c.jpg"]);
    assert_eq!(list.current_index(), 0);
    assert!(list.prev());
    assert_eq!(list.current_index(), 2);
}

#[test]
fn prev_decrements() {
    let mut list = make_list(&["a.jpg", "b.jpg", "c.jpg"]);
    list.set_current(2);
    assert!(list.prev());
    assert_eq!(list.current_index(), 1);
}

#[test]
fn next_empty() {
    let mut list = make_empty_list();
    assert!(!list.next());
}

#[test]
fn prev_empty() {
    let mut list = make_empty_list();
    assert!(!list.prev());
}

#[test]
fn jump_first() {
    let mut list = make_list(&["a.jpg", "b.jpg", "c.jpg", "d.jpg", "e.jpg", "f.jpg"]);
    list.set_current(5);
    assert_eq!(list.current_index(), 5);
    list.jump_first();
    assert_eq!(list.current_index(), 0);
}

#[test]
fn jump_last() {
    let names: Vec<String> = (0..10).map(|i| format!("img{}.jpg", i)).collect();
    let refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
    let mut list = make_list(&refs);
    list.jump_last();
    assert_eq!(list.current_index(), 9);
}

#[test]
fn jump_forward() {
    let names: Vec<String> = (0..10).map(|i| format!("img{}.jpg", i)).collect();
    let refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
    let mut list = make_list(&refs);
    list.jump_forward(3);
    assert_eq!(list.current_index(), 3);
}

#[test]
fn jump_forward_wraps() {
    let names: Vec<String> = (0..10).map(|i| format!("img{}.jpg", i)).collect();
    let refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
    let mut list = make_list(&refs);
    list.set_current(8);
    list.jump_forward(5);
    assert_eq!(list.current_index(), 3);
}

#[test]
fn jump_back() {
    let names: Vec<String> = (0..10).map(|i| format!("img{}.jpg", i)).collect();
    let refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
    let mut list = make_list(&refs);
    list.set_current(5);
    list.jump_back(3);
    assert_eq!(list.current_index(), 2);
}

#[test]
fn jump_back_wraps() {
    let names: Vec<String> = (0..10).map(|i| format!("img{}.jpg", i)).collect();
    let refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
    let mut list = make_list(&refs);
    list.set_current(1);
    list.jump_back(3);
    assert_eq!(list.current_index(), 8);
}

#[test]
fn set_current_valid() {
    let names: Vec<String> = (0..10).map(|i| format!("img{}.jpg", i)).collect();
    let refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
    let mut list = make_list(&refs);
    list.set_current(5);
    assert_eq!(list.current_index(), 5);
}

#[test]
fn set_current_oob() {
    let names: Vec<String> = (0..10).map(|i| format!("img{}.jpg", i)).collect();
    let refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
    let mut list = make_list(&refs);
    list.set_current(20);
    assert_eq!(list.current_index(), 0);
}

// ── Removal ─────────────────────────────────────────────────

#[test]
fn remove_current_middle() {
    let mut list = make_list(&["a.jpg", "b.jpg", "c.jpg"]);
    list.set_current(1);
    assert!(list.remove_current());
    assert_eq!(list.len(), 2);
    assert!(list.current().is_some());
}

#[test]
fn remove_last_item() {
    let mut list = make_list(&["only.jpg"]);
    assert!(!list.remove_current());
    assert!(list.is_empty());
}

#[test]
fn remove_at_end() {
    let mut list = make_list(&["a.jpg", "b.jpg", "c.jpg"]);
    list.set_current(2);
    assert!(list.remove_current());
    assert_eq!(list.len(), 2);
    assert_eq!(list.current_index(), 1);
}

// ── Sorting ─────────────────────────────────────────────────

#[test]
fn sort_name_natural() {
    let mut list = make_list(&["img2.jpg", "img10.jpg", "img1.jpg"]);
    list.sort_by("name", false);
    let names: Vec<&str> = list.files().iter().map(|f| f.name.as_str()).collect();
    assert_eq!(names, vec!["img1.jpg", "img2.jpg", "img10.jpg"]);
}

#[test]
fn sort_reverse() {
    let mut list = make_list(&["a.jpg", "b.jpg", "c.jpg"]);
    list.sort_by("name", true);
    let names: Vec<&str> = list.files().iter().map(|f| f.name.as_str()).collect();
    assert_eq!(names, vec!["c.jpg", "b.jpg", "a.jpg"]);
}

#[test]
fn sort_none_preserves_order() {
    let mut list = make_list(&["z.jpg", "a.jpg", "m.jpg"]);
    list.sort_by("none", false);
    let names: Vec<&str> = list.files().iter().map(|f| f.name.as_str()).collect();
    assert_eq!(names, vec!["z.jpg", "a.jpg", "m.jpg"]);
}

#[test]
fn sort_format() {
    let mut list = make_list(&["b.png", "a.jpg", "c.png", "d.jpg"]);
    list.sort_by("format", false);
    let exts: Vec<String> = list
        .files()
        .iter()
        .map(|f| f.path.extension().unwrap().to_string_lossy().to_lowercase())
        .collect();
    assert_eq!(exts, vec!["jpg", "jpg", "png", "png"]);
}

// ── is_image_file ───────────────────────────────────────────

#[test]
fn is_image_jpg() {
    assert!(is_image_file(Path::new("photo.jpg")));
}

#[test]
fn is_image_png_upper() {
    assert!(is_image_file(Path::new("photo.PNG")));
}

#[test]
fn is_image_txt() {
    assert!(!is_image_file(Path::new("readme.txt")));
}

#[test]
fn is_image_no_ext() {
    assert!(!is_image_file(Path::new("binary")));
}

#[test]
fn remote_url_detection_is_scheme_specific() {
    assert!(is_remote_url("http://example.com/image.png"));
    assert!(is_remote_url("https://example.com/image.png"));
    assert!(!is_remote_url("ftp://example.com/image.png"));
    assert!(!is_remote_url("C:\\images\\image.png"));
}

#[test]
fn collect_local_ignores_remote_urls() {
    let dir = tempfile::tempdir().unwrap();
    let image_path = dir.path().join("local.png");
    std::fs::write(&image_path, b"fixture").unwrap();
    let list = FileList::collect_local(
        &[
            image_path.to_string_lossy().into_owned(),
            "https://example.com/remote.png".to_string(),
        ],
        false,
    );
    assert_eq!(list.len(), 1);
    assert_eq!(list.current().unwrap().path, image_path);
}

#[test]
fn recursive_discovery_finds_nested_images_without_following_links() {
    let dir = tempfile::tempdir().unwrap();
    let nested = dir.path().join("nested");
    std::fs::create_dir(&nested).unwrap();
    let image_path = nested.join("nested.png");
    std::fs::write(&image_path, b"fixture").unwrap();

    let paths = vec![dir.path().to_string_lossy().into_owned()];
    let files = FileList::discover_recursive(&paths);
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].path, image_path);
}

#[test]
fn filelist_round_trip_ignores_missing_and_non_image_paths() {
    let dir = tempfile::tempdir().unwrap();
    let image = dir.path().join("image.png");
    let text = dir.path().join("notes.txt");
    let list_file = dir.path().join("files.txt");
    std::fs::write(&image, b"fixture").unwrap();
    std::fs::write(&text, b"not an image").unwrap();
    std::fs::write(
        &list_file,
        format!(
            "{}\n\n{}\n{}\n",
            image.display(),
            text.display(),
            dir.path().join("missing.jpg").display()
        ),
    )
    .unwrap();

    let list = FileList::from_filelist(&list_file);
    assert_eq!(list.len(), 1);
    assert_eq!(list.current().unwrap().path, image);
}

#[test]
fn save_filelist_reports_parent_directory_errors() {
    let list = make_list(&["image.png"]);
    let result = list.save_filelist(Path::new("tests/does-not-exist/files.txt"));
    assert!(result.is_err());
}

#[test]
fn filter_dimensions_applies_minimum_and_maximum_bounds() {
    let loader = crate::image_loader::ImageLoader::new().unwrap();
    let mut list =
        FileList::from_single(FehFile::new(PathBuf::from("tests/fixtures/test_1x1.png")));
    list.append(FehFile::new(PathBuf::from("tests/fixtures/missing.png")));

    list.filter_dimensions(&loader, Some((1, 1)), Some((1, 1)));
    assert_eq!(list.len(), 2);

    list.filter_dimensions(&loader, Some((2, 2)), None);
    assert_eq!(list.len(), 1);
    assert_eq!(
        list.current().unwrap().path,
        PathBuf::from("tests/fixtures/missing.png")
    );
}

// ── File I/O ────────────────────────────────────────────────

#[test]
fn save_and_load_filelist() {
    use std::io::Write;

    let dir = tempfile::tempdir().unwrap();
    let img_names = ["one.jpg", "two.png", "three.gif"];
    let mut paths = Vec::new();
    for name in &img_names {
        let p = dir.path().join(name);
        let mut f = std::fs::File::create(&p).unwrap();
        f.write_all(b"fake").unwrap();
        paths.push(p);
    }

    let files: Vec<FehFile> = paths.iter().map(|p| FehFile::new(p.clone())).collect();
    let mut list = FileList::from_single(files[0].clone());
    *list.files_mut() = files;

    let list_path = dir.path().join("filelist.txt");
    list.save_filelist(&list_path).unwrap();

    let loaded = FileList::from_filelist(&list_path);
    assert_eq!(loaded.len(), 3);
    let loaded_names: Vec<&str> = loaded.files().iter().map(|f| f.name.as_str()).collect();
    for name in &img_names {
        assert!(loaded_names.contains(name));
    }
}

#[test]
fn jump_to_match() {
    let mut list = make_list(&["alpha.jpg", "beta.jpg", "gamma.jpg"]);
    list.jump_to("beta.jpg");
    assert_eq!(list.current_index(), 1);
}

// ── Misc ────────────────────────────────────────────────────

#[test]
fn randomize_preserves_length() {
    let mut list = make_list(&["a.jpg", "b.jpg", "c.jpg", "d.jpg", "e.jpg"]);
    list.randomize();
    assert_eq!(list.len(), 5);
    let mut names: Vec<String> = list.files().iter().map(|f| f.name.clone()).collect();
    names.sort();
    assert_eq!(names, vec!["a.jpg", "b.jpg", "c.jpg", "d.jpg", "e.jpg"]);
}
