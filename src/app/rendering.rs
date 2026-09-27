use super::*;

impl AppState {
    pub fn update_title(&self) {
        if self.hwnd == HWND::default() {
            return;
        }

        let (img_w, img_h) = self
            .current_image
            .as_ref()
            .map(|i| (Some(i.width), Some(i.height)))
            .unwrap_or((None, None));

        let title = expand_format(
            &self.options.title,
            self.filelist.current(),
            self.filelist.current_index(),
            self.filelist.len(),
            self.zoom,
            img_w,
            img_h,
            self.paused,
        );

        window::update_title(self.hwnd, &title);
    }
}
