use super::*;

impl AppState {
    pub fn zoom_to_fit(&mut self) {
        if let Some(ref img) = self.current_image
            && self.window_width > 0
            && self.window_height > 0
        {
            self.zoom = transforms::fit_zoom(
                img.width as f64,
                img.height as f64,
                self.window_width as f64,
                self.window_height as f64,
            );
            self.pan_x = 0.0;
            self.pan_y = 0.0;
        }
    }

    pub fn navigate_next(&mut self) {
        self.filelist.next();
        self.load_current_image();
    }

    pub fn navigate_prev(&mut self) {
        self.filelist.prev();
        self.load_current_image();
    }

    pub fn paint(&mut self) -> windows::core::Result<()> {
        let started = std::time::Instant::now();
        // Thumbnail / index mode rendering
        if let Some(ref mut thumb_view) = self.thumbnail_view {
            if let Some(rt) = self.renderer.render_target() {
                let result = thumb_view.render(rt, &self.filelist, &self.image_loader);
                self.metrics
                    .log("thumbnail_render", started, "mode=thumbnail");
                return result;
            }
            return Ok(());
        }

        let filename = self
            .filelist
            .current()
            .map(|f| f.path.to_string_lossy().into_owned())
            .unwrap_or_default();

        let info_text = if self.options.draw_info {
            crate::overlay::build_info_string(self)
        } else {
            String::new()
        };

        let result = self.renderer.render(
            self.zoom,
            self.pan_x,
            self.pan_y,
            self.rotation,
            self.flip_h,
            self.flip_v,
            self.options.draw_filename,
            &filename,
            self.options.draw_info,
            &info_text,
            self.dpi_scale,
        );
        self.metrics.log("frame_render", started, "mode=image");
        if result.is_err() {
            self.renderer.recreate_render_target(
                self.hwnd,
                self.window_width,
                self.window_height,
            )?;
            if let Some(image) = self.current_image.as_ref() {
                self.renderer
                    .load_bitmap(image, self.image_loader.wic_factory())?;
            }
            window::invalidate(self.hwnd);
            return Ok(());
        }
        result
    }

    pub fn handle_resize(&mut self, width: u32, height: u32) -> windows::core::Result<()> {
        self.window_width = width;
        self.window_height = height;
        if self.renderer.resize(width, height).is_err() {
            self.renderer
                .recreate_render_target(self.hwnd, width, height)?;
            if let Some(image) = self.current_image.as_ref() {
                self.renderer
                    .load_bitmap(image, self.image_loader.wic_factory())?;
            }
        }

        // Re-fit image on resize if we're at fit-to-window zoom
        if self.options.scale_down {
            self.zoom_to_fit();
        }

        Ok(())
    }

    pub fn handle_timer(&mut self) {
        if !self.paused && self.filelist.len() > 1 {
            let at_last = self.filelist.current_index() == self.filelist.len() - 1;
            if at_last {
                match self.options.on_last_slide_action() {
                    crate::config::OnLastSlide::Quit => {
                        if self.hwnd != HWND::default() {
                            unsafe {
                                let _ = DestroyWindow(self.hwnd);
                            }
                        }
                        return;
                    }
                    crate::config::OnLastSlide::Hold => {
                        self.paused = true;
                        return;
                    }
                    crate::config::OnLastSlide::Resume => {}
                }
            }
            self.navigate_next();
        }
    }

    pub(super) fn reset_slideshow_timer(&self, hwnd: HWND) {
        if let Some(ms) = slideshow_timer_ms(self.options.slideshow_delay) {
            unsafe {
                SetTimer(Some(hwnd), SLIDESHOW_TIMER_ID, ms, None);
            }
        }
    }
}
