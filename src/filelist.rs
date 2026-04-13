use std::path::{Path, PathBuf};

use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct FehFile {
    pub path: PathBuf,
    pub name: String,
    pub size: Option<u64>,
    pub mtime: Option<std::time::SystemTime>,
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
    "jpg", "jpeg", "png", "bmp", "gif", "tiff", "tif", "webp", "ico", "svg",
    "heif", "heic", "avif", "jxl", "raw", "cr2", "nef", "arw", "dng",
];

fn is_image_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| IMAGE_EXTENSIONS.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}

impl FileList {
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            current: 0,
        }
    }

    /// Collect files from a list of paths (files and/or directories)
    pub fn collect(paths: &[String], recursive: bool) -> Self {
        let mut files = Vec::new();

        for path_str in paths {
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
            "name" => self.files.sort_by(|a, b| {
                natord::compare(&a.name.to_lowercase(), &b.name.to_lowercase())
            }),
            "filename" => self.files.sort_by(|a, b| {
                natord::compare(
                    &a.path.to_string_lossy().to_lowercase(),
                    &b.path.to_string_lossy().to_lowercase(),
                )
            }),
            "dirname" => self.files.sort_by(|a, b| {
                let da = a.path.parent().map(|p| p.to_string_lossy().to_lowercase());
                let db = b.path.parent().map(|p| p.to_string_lossy().to_lowercase());
                da.cmp(&db).then_with(|| {
                    natord::compare(&a.name.to_lowercase(), &b.name.to_lowercase())
                })
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
            "none" => {}
            _ => self.files.sort_by(|a, b| {
                natord::compare(&a.name.to_lowercase(), &b.name.to_lowercase())
            }),
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
        if let Some(name) = target_name {
            if let Some(idx) = self
                .files
                .iter()
                .position(|f| f.name.to_lowercase() == name)
            {
                self.current = idx;
            }
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
}
