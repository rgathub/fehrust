use std::path::{Path, PathBuf};

use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct FehFile {
    pub path: PathBuf,
    pub name: String,
    pub size: Option<u64>,
    pub mtime: Option<std::time::SystemTime>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

impl FehFile {
    pub fn new(path: PathBuf) -> Self {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        Self {
            path,
            name,
            size: None,
            mtime: None,
            width: None,
            height: None,
        }
    }

    /// Populate size/mtime from filesystem metadata
    pub fn load_stat(&mut self) {
        if let Ok(meta) = std::fs::metadata(&self.path) {
            self.size = Some(meta.len());
            self.mtime = meta.modified().ok();
        }
    }
}

#[derive(Debug)]
pub struct FileList {
    files: Vec<FehFile>,
    current: usize,
}

const IMAGE_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "png", "bmp", "gif", "tiff", "tif", "webp", "ico", "svg", "heif", "heic",
    "avif", "jxl", "raw", "cr2", "nef", "arw", "dng",
];

fn is_image_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| IMAGE_EXTENSIONS.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}

impl FileList {
    /// Create a FileList from a single file
    pub fn from_single(file: FehFile) -> Self {
        Self {
            files: vec![file],
            current: 0,
        }
    }

    /// Collect files from a list of paths (files and/or directories)
    pub fn collect(paths: &[String], recursive: bool) -> Self {
        let mut files = Vec::new();

        for path_str in paths {
            // Handle HTTP/HTTPS URLs
            if path_str.starts_with("http://") || path_str.starts_with("https://") {
                match crate::http::fetch_image(path_str) {
                    Ok(local_path) => {
                        if is_image_file(&local_path) {
                            files.push(FehFile::new(local_path));
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to fetch {}: {}", path_str, e);
                    }
                }
                continue;
            }

            let path = PathBuf::from(path_str);
            if path.is_file() {
                if is_image_file(&path) {
                    files.push(FehFile::new(path));
                }
            } else if path.is_dir() {
                if recursive {
                    for entry in WalkDir::new(&path)
                        .follow_links(true)
                        .into_iter()
                        .filter_map(|e| e.ok())
                    {
                        if entry.file_type().is_file() && is_image_file(entry.path()) {
                            files.push(FehFile::new(entry.into_path()));
                        }
                    }
                } else {
                    if let Ok(entries) = std::fs::read_dir(&path) {
                        for entry in entries.filter_map(|e| e.ok()) {
                            let p = entry.path();
                            if p.is_file() && is_image_file(&p) {
                                files.push(FehFile::new(p));
                            }
                        }
                    }
                }
            }
        }

        Self { files, current: 0 }
    }

    pub fn sort_by(&mut self, mode: &str, reverse: bool) {
        match mode {
            "name" => self
                .files
                .sort_by(|a, b| natord::compare(&a.name.to_lowercase(), &b.name.to_lowercase())),
            "filename" => self.files.sort_by(|a, b| {
                natord::compare(
                    &a.path.to_string_lossy().to_lowercase(),
                    &b.path.to_string_lossy().to_lowercase(),
                )
            }),
            "dirname" => self.files.sort_by(|a, b| {
                let da = a.path.parent().map(|p| p.to_string_lossy().to_lowercase());
                let db = b.path.parent().map(|p| p.to_string_lossy().to_lowercase());
                da.cmp(&db)
                    .then_with(|| natord::compare(&a.name.to_lowercase(), &b.name.to_lowercase()))
            }),
            "mtime" => {
                for f in &mut self.files {
                    f.load_stat();
                }
                self.files.sort_by(|a, b| a.mtime.cmp(&b.mtime));
            }
            "size" => {
                for f in &mut self.files {
                    f.load_stat();
                }
                self.files.sort_by(|a, b| a.size.cmp(&b.size));
            }
            "width" | "height" | "pixels" => {
                let loader = crate::image_loader::ImageLoader::new();
                if let Ok(loader) = loader {
                    for f in &mut self.files {
                        if f.width.is_none()
                            && let Ok((w, h)) = loader.get_dimensions(&f.path)
                        {
                            f.width = Some(w);
                            f.height = Some(h);
                        }
                    }
                }
                match mode {
                    "width" => self.files.sort_by_key(|f| f.width.unwrap_or(0)),
                    "height" => self.files.sort_by_key(|f| f.height.unwrap_or(0)),
                    "pixels" => self.files.sort_by_key(|f| {
                        (f.width.unwrap_or(0) as u64) * (f.height.unwrap_or(0) as u64)
                    }),
                    _ => unreachable!(),
                }
            }
            "format" => self.files.sort_by(|a, b| {
                let ext_a = a
                    .path
                    .extension()
                    .map(|e| e.to_string_lossy().to_lowercase())
                    .unwrap_or_default();
                let ext_b = b
                    .path
                    .extension()
                    .map(|e| e.to_string_lossy().to_lowercase())
                    .unwrap_or_default();
                ext_a
                    .cmp(&ext_b)
                    .then_with(|| natord::compare(&a.name.to_lowercase(), &b.name.to_lowercase()))
            }),
            "none" => {}
            _ => self
                .files
                .sort_by(|a, b| natord::compare(&a.name.to_lowercase(), &b.name.to_lowercase())),
        }

        if reverse {
            self.files.reverse();
        }
    }

    pub fn randomize(&mut self) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        use std::time::SystemTime;

        let mut seed = {
            let mut h = DefaultHasher::new();
            SystemTime::now().hash(&mut h);
            std::process::id().hash(&mut h);
            h.finish()
        };

        // Fisher-Yates shuffle with simple xorshift
        let n = self.files.len();
        for i in (1..n).rev() {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            let j = (seed as usize) % (i + 1);
            self.files.swap(i, j);
        }
    }

    /// Move to file matching start_at, if found
    pub fn jump_to(&mut self, start_at: &str) {
        let target = Path::new(start_at);
        // Try exact match first
        if let Some(idx) = self.files.iter().position(|f| f.path == target) {
            self.current = idx;
            return;
        }
        // Try basename match
        let target_name = target
            .file_name()
            .map(|n| n.to_string_lossy().to_lowercase());
        if let Some(name) = target_name
            && let Some(idx) = self
                .files
                .iter()
                .position(|f| f.name.to_lowercase() == name)
        {
            self.current = idx;
        }
    }

    pub fn current(&self) -> Option<&FehFile> {
        self.files.get(self.current)
    }

    pub fn current_index(&self) -> usize {
        self.current
    }

    pub fn len(&self) -> usize {
        self.files.len()
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    pub fn file_at(&self, index: usize) -> Option<&FehFile> {
        self.files.get(index)
    }

    pub fn set_current(&mut self, index: usize) {
        if index < self.files.len() {
            self.current = index;
        }
    }

    pub fn next(&mut self) -> bool {
        if self.files.is_empty() {
            return false;
        }
        if self.current + 1 < self.files.len() {
            self.current += 1;
            true
        } else {
            self.current = 0;
            true
        }
    }

    pub fn prev(&mut self) -> bool {
        if self.files.is_empty() {
            return false;
        }
        if self.current > 0 {
            self.current -= 1;
        } else {
            self.current = self.files.len() - 1;
        }
        true
    }

    pub fn jump_first(&mut self) {
        self.current = 0;
    }

    pub fn jump_last(&mut self) {
        if !self.files.is_empty() {
            self.current = self.files.len() - 1;
        }
    }

    pub fn jump_forward(&mut self, n: usize) {
        if self.files.is_empty() {
            return;
        }
        self.current = (self.current + n) % self.files.len();
    }

    pub fn jump_back(&mut self, n: usize) {
        if self.files.is_empty() {
            return;
        }
        let len = self.files.len();
        self.current = (self.current + len - (n % len)) % len;
    }

    pub fn remove_current(&mut self) -> bool {
        if self.files.is_empty() {
            return false;
        }
        self.files.remove(self.current);
        if self.current >= self.files.len() && !self.files.is_empty() {
            self.current = self.files.len() - 1;
        }
        !self.files.is_empty()
    }

    /// Get a shared reference to the file list
    pub fn files(&self) -> &[FehFile] {
        &self.files
    }

    /// Get a mutable reference to the file list
    pub fn files_mut(&mut self) -> &mut Vec<FehFile> {
        &mut self.files
    }

    /// Load file paths from a text file (one per line), keeping only image files
    pub fn from_filelist(path: &Path) -> Self {
        let content = std::fs::read_to_string(path).unwrap_or_default();
        let files: Vec<FehFile> = content
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| PathBuf::from(l.trim()))
            .filter(|p| p.is_file() && is_image_file(p))
            .map(FehFile::new)
            .collect();
        Self { files, current: 0 }
    }

    /// Save file paths to a text file (one per line)
    pub fn save_filelist(&self, path: &Path) -> std::io::Result<()> {
        use std::io::Write;
        let mut f = std::fs::File::create(path)?;
        for file in &self.files {
            writeln!(f, "{}", file.path.display())?;
        }
        Ok(())
    }

    /// Filter files by image dimensions
    pub fn filter_dimensions(
        &mut self,
        loader: &crate::image_loader::ImageLoader,
        min: Option<(u32, u32)>,
        max: Option<(u32, u32)>,
    ) {
        self.files.retain(|f| {
            match loader.get_dimensions(&f.path) {
                Ok((w, h)) => {
                    if let Some((min_w, min_h)) = min
                        && (w < min_w || h < min_h)
                    {
                        return false;
                    }
                    if let Some((max_w, max_h)) = max
                        && (w > max_w || h > max_h)
                    {
                        return false;
                    }
                    true
                }
                Err(_) => true, // keep files we can't read
            }
        });
        if self.current >= self.files.len() && !self.files.is_empty() {
            self.current = self.files.len() - 1;
        } else if self.files.is_empty() {
            self.current = 0;
        }
    }
}

#[cfg(test)]
mod tests {
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
}
