use super::*;

impl AppState {
    pub fn load_current_image(&mut self) {
        self.pan_x = 0.0;
        self.pan_y = 0.0;
        self.rotation = 0.0;
        self.flip_h = false;
        self.flip_v = false;
        self.renderer.clear_bitmap();
        self.current_exif = None;
        self.current_caption = None;

        if let Some(file) = self.filelist.current() {
            let exif_info = exif::read_exif(&file.path);

            // Load caption if caption_path is set
            if let Some(ref caption_path) = self.options.caption_path
                && let Some(stem) = file.path.file_stem()
            {
                let caption_file =
                    Path::new(caption_path).join(format!("{}.txt", stem.to_string_lossy()));
                if let Ok(text) = std::fs::read_to_string(&caption_file) {
                    let trimmed = text.trim().to_string();
                    if !trimmed.is_empty() {
                        self.current_caption = Some(trimmed);
                    }
                }
            }

            self.current_exif = exif_info;
            self.next_load_id = self.next_load_id.wrapping_add(1);
            let load_id = self.next_load_id;
            self.load_cancel.store(load_id, Ordering::Release);
            let path = file.path.clone();
            let tx = self.image_load_tx.clone();
            let hwnd_raw = self.hwnd.0 as isize;
            let load_cancel = Arc::clone(&self.load_cancel);
            thread::spawn(move || {
                if load_cancel.load(Ordering::Acquire) != load_id {
                    return;
                }
                let result = ImageLoader::new()
                    .and_then(|loader| loader.decode_pixels(&path))
                    .map_err(|error| format!("Failed to load {}: {error}", path.display()));
                if load_cancel.load(Ordering::Acquire) != load_id {
                    return;
                }
                let _ = tx.send((load_id, path, result));
                if hwnd_raw != 0 {
                    unsafe {
                        let hwnd = HWND(hwnd_raw as *mut _);
                        let _ = PostMessageW(Some(hwnd), WM_IMAGE_LOADED, WPARAM(0), LPARAM(0));
                    }
                }
            });
        }

        self.refresh_watcher();
        self.update_title();
    }

    fn refresh_watcher(&mut self) {
        if !self.options.auto_reload || self.hwnd == HWND::default() {
            return;
        }
        if let Some(watcher) = self.watcher.take() {
            watcher.stop();
        }
        if let Some(parent) = self.filelist.current().and_then(|file| file.path.parent()) {
            self.watcher = Some(crate::filewatcher::start_watcher(
                parent.to_path_buf(),
                self.hwnd,
            ));
        }
    }

    pub fn process_image_loads(&mut self, hwnd: HWND) {
        while let Ok((load_id, path, result)) = self.image_load_rx.try_recv() {
            let current_path = self.filelist.current().map(|file| file.path.clone());
            if load_id != self.next_load_id || current_path.as_ref() != Some(&path) {
                continue;
            }

            match result {
                Ok(decoded) => {
                    let image = match self.image_loader.loaded_from_pixels(decoded) {
                        Ok(image) => image,
                        Err(error) => {
                            eprintln!("Failed to create WIC image: {error}");
                            continue;
                        }
                    };
                    if let Err(e) = self
                        .renderer
                        .load_bitmap(&image, self.image_loader.wic_factory())
                    {
                        eprintln!("Failed to create bitmap: {e}");
                    }
                    self.current_image = Some(image);
                    self.zoom_to_fit();

                    if let Some(ref exif) = self.current_exif
                        && exif.orientation != 1
                    {
                        let (rot, fh, fv) = exif::exif_orientation_to_rotation(exif.orientation);
                        self.rotation = rot;
                        self.flip_h = fh;
                        self.flip_v = fv;
                    }
                }
                Err(error) => {
                    eprintln!("{error}");
                    self.current_image = None;
                }
            }
            self.update_title();
            window::invalidate(hwnd);
        }
    }

    pub fn start_background_tasks(&mut self, hwnd: HWND) {
        let urls: Vec<String> = self
            .options
            .files
            .iter()
            .filter(|path| crate::filelist::is_remote_url(path))
            .cloned()
            .collect();
        if !urls.is_empty() {
            let queue = Arc::new(Mutex::new(VecDeque::from(urls)));
            let tx = self.remote_image_tx.clone();
            let hwnd_raw = hwnd.0 as isize;
            for _ in 0..4 {
                let queue = Arc::clone(&queue);
                let tx = tx.clone();
                let cancel = Arc::clone(&self.background_cancel);
                thread::spawn(move || {
                    loop {
                        if cancel.load(Ordering::Acquire) {
                            break;
                        }
                        let url = queue.lock().ok().and_then(|mut urls| urls.pop_front());
                        let Some(url) = url else {
                            break;
                        };
                        let result = crate::http::fetch_image(&url)
                            .map(crate::filelist::FehFile::new)
                            .map_err(|error| format!("Failed to fetch {url}: {error}"));
                        if cancel.load(Ordering::Acquire) {
                            break;
                        }
                        if tx.send(result).is_err() {
                            break;
                        }
                        unsafe {
                            let hwnd = HWND(hwnd_raw as *mut _);
                            let _ = PostMessageW(Some(hwnd), WM_REMOTE_IMAGE, WPARAM(0), LPARAM(0));
                        }
                    }
                });
            }
        }

        if self.options.recursive && self.options.filelist.is_none() {
            let paths = self.options.files.clone();
            let tx = self.discovery_tx.clone();
            let hwnd_raw = hwnd.0 as isize;
            let cancel = Arc::clone(&self.background_cancel);
            thread::spawn(move || {
                for file in crate::filelist::FileList::discover_recursive(&paths) {
                    if cancel.load(Ordering::Acquire) {
                        return;
                    }
                    if tx.send(file).is_err() {
                        return;
                    }
                    unsafe {
                        let hwnd = HWND(hwnd_raw as *mut _);
                        let _ = PostMessageW(Some(hwnd), WM_FILE_DISCOVERED, WPARAM(0), LPARAM(0));
                    }
                }
            });
        }
    }

    pub fn process_remote_images(&mut self, hwnd: HWND) {
        let mut added = false;
        while let Ok(result) = self.remote_image_rx.try_recv() {
            match result {
                Ok(file) => {
                    self.filelist.append(file);
                    added = true;
                }
                Err(error) => eprintln!("{error}"),
            }
        }
        if added {
            let current_path = self.filelist.current().map(|file| file.path.clone());
            if self.options.randomize {
                self.filelist.randomize();
            } else {
                self.filelist
                    .sort_by(&self.options.sort, self.options.reverse);
            }
            if let Some(path) = current_path {
                self.filelist.jump_to(&path.to_string_lossy());
            }
            if self.current_image.is_none() {
                self.load_current_image();
            }
            self.update_title();
            window::invalidate(hwnd);
        }
    }

    pub fn process_discovered_files(&mut self, hwnd: HWND) {
        let mut added = false;
        while let Ok(file) = self.discovery_rx.try_recv() {
            self.filelist.append(file);
            added = true;
        }
        if !added {
            return;
        }
        let current_path = self.filelist.current().map(|file| file.path.clone());
        if self.options.randomize {
            self.filelist.randomize();
        } else {
            self.filelist
                .sort_by(&self.options.sort, self.options.reverse);
        }
        if let Some(path) = current_path {
            self.filelist.jump_to(&path.to_string_lossy());
        }
        if self.current_image.is_none() {
            self.load_current_image();
        }
        self.update_title();
        window::invalidate(hwnd);
    }
}
