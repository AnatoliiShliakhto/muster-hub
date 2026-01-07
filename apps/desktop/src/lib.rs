use dioxus::desktop::{Config, WindowBuilder};
use dioxus::prelude::*;

pub struct DesktopApp {
    title: String,
    width: f64,
    height: f64,
}

impl Default for DesktopApp {
    fn default() -> Self {
        Self { title: "MusterHub Desktop".into(), width: 1200.0, height: 800.0 }
    }
}

impl DesktopApp {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn with_size(mut self, width: f64, height: f64) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// The entry point for launching the app
    pub fn launch(self, root: fn() -> Element) {
        let window = WindowBuilder::new()
            .with_title(&self.title)
            .with_inner_size(dioxus::desktop::LogicalSize {
                width: self.width,
                height: self.height,
            });

        let cfg = Config::default()
            .with_window(window)
            // Enable devtools in debug mode for a pro experience
            .with_custom_head(r#"<meta name="viewport" content="width=device-width, initial-scale=1.0">"#.into());

        LaunchBuilder::desktop()
            .with_cfg(cfg)
            .with_context_provider(|| Box::new(()))
            .launch(root);
    }
}
