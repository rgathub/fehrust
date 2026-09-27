use super::*;

impl AppState {
    pub fn remove_current_from_list(&mut self, hwnd: HWND) {
        if !self.filelist.remove_current() {
            // List is empty, quit
            unsafe {
                let _ = DestroyWindow(hwnd);
            }
            return;
        }
        self.load_current_image();
        self.reset_slideshow_timer(hwnd);
        window::invalidate(hwnd);
    }

    pub fn delete_current_from_disk(&mut self, hwnd: HWND) {
        let Some(path) = self.filelist.current().map(|file| file.path.clone()) else {
            return;
        };

        self.current_image = None;
        self.renderer.clear_bitmap();

        match std::fs::remove_file(&path) {
            Ok(()) => {
                eprintln!("Deleted: {}", path.display());
                if !self.filelist.remove_current() {
                    unsafe {
                        let _ = DestroyWindow(hwnd);
                    }
                    return;
                }
                self.load_current_image();
                self.reset_slideshow_timer(hwnd);
                window::invalidate(hwnd);
            }
            Err(e) => {
                eprintln!("Failed to delete {}: {e}", path.display());
                self.load_current_image();
                window::invalidate(hwnd);
            }
        }
    }

    pub fn move_current_to_directory(&mut self, hwnd: HWND) {
        let Some(destination) = self.options.move_to.as_deref() else {
            eprintln!("Move failed: specify a destination with --move DIRECTORY");
            return;
        };
        let destination = Path::new(destination);
        if let Err(e) = std::fs::create_dir_all(destination) {
            eprintln!(
                "Move failed: could not create destination directory {}: {e}",
                destination.display()
            );
            return;
        }

        let Some(path) = self.filelist.current().map(|file| file.path.clone()) else {
            return;
        };
        let Some(file_name) = path.file_name() else {
            eprintln!("Move failed: current image has no file name");
            return;
        };
        let target = destination.join(file_name);
        if target == path {
            eprintln!("Move skipped: destination is the current image directory");
            return;
        }

        self.current_image = None;
        self.renderer.clear_bitmap();

        match move_file(&path, &target) {
            Ok(()) => {
                eprintln!("Moved {} to {}", path.display(), target.display());
                if !self.filelist.remove_current() {
                    unsafe {
                        let _ = DestroyWindow(hwnd);
                    }
                    return;
                }
                self.load_current_image();
                self.reset_slideshow_timer(hwnd);
                window::invalidate(hwnd);
            }
            Err(e) => {
                eprintln!(
                    "Failed to move {} to {}: {e}",
                    path.display(),
                    target.display()
                );
                self.load_current_image();
                window::invalidate(hwnd);
            }
        }
    }
}
