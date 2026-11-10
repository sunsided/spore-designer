//! Spore Designer: tweak a mould's parameters and watch it grow.
//!
//! # Where to find what
//!
//! - **The app loop:** this file. Each frame: advance playback and the camera
//!   ([`App::update`]), draw the preview, then the panel, then run any file operation.
//! - **State:** `editor.rs` (what the user is editing and viewing), `design.rs` (what gets
//!   saved, and painting all stages of it).
//! - **The panel:** `panel.rs` (layout) and `panel/egui_form.rs` (widgets). Every value and
//!   its range is listed once in `fields.rs`, shared with randomizing and loading.
//! - **The preview:** `preview.rs`; colony and patch in `preview/sheet.rs`, the endless field
//!   in `preview/field.rs`.
//! - **Files:** `files.rs` (save, load, PNG export).
//!
//! The painting itself lives in the `spore-core` crate.

mod design;
mod editor;
mod fields;
mod files;
mod panel;
mod preview;
mod work;

use game_toolkit::prelude::*;

use design::View;
use editor::{Action, Editor};
use preview::{FieldPreview, PreviewArea, SheetPreview};
use work::Background;

struct App {
    ui: Ui,
    editor: Editor,
    sheet: SheetPreview,
    field: FieldPreview,
    /// Painting jobs (threads on the desktop, a per-frame queue in the browser).
    background: Background,
    /// Right edge of the panel in screen pixels, from the previous frame (the panel is laid
    /// out after the preview is drawn).
    panel_width: f32,
    /// Message from the last file operation.
    status: String,
}

impl Game for App {
    fn init(ctx: &mut Context) -> Result<Self> {
        Ok(Self {
            ui: Ui::new(&ctx.gfx)?,
            editor: Editor::new(),
            sheet: SheetPreview::new(),
            field: FieldPreview::new(),
            background: Background::default(),
            panel_width: 0.0,
            status: String::new(),
        })
    }

    fn raw_window_event(&mut self, _ctx: &mut Context, event: &winit::event::WindowEvent) -> bool {
        self.ui.on_window_event(event)
    }

    fn update(&mut self, ctx: &mut Context, seconds: f32) {
        self.editor.advance_playback(seconds);

        // The cursor light follows the mouse while it is over the preview.
        let (mouse_x, mouse_y) = ctx.input.mouse_pos_window();
        if mouse_x > self.panel_width {
            self.editor.cursor = Some([mouse_x, mouse_y]);
        }

        if self.editor.design.view == View::Field {
            self.field
                .pan(&mut self.editor, &ctx.input, seconds, self.panel_width);
        }
    }

    fn render(&mut self, ctx: &mut Context, frame: &mut Frame) {
        self.background.run_queued();
        self.draw_preview(&mut ctx.gfx, frame);

        if let Some(action) = self.draw_panel(&mut ctx.gfx, frame) {
            self.status = files::run(action, &mut self.editor, self.sheet.frames());
        }
        if let Some(status) = files::poll(&mut self.editor) {
            self.status = status;
        }
    }
}

impl App {
    /// The preview fills the window right of the panel.
    fn draw_preview(&mut self, gfx: &mut Graphics, frame: &mut Frame) {
        let (window_width, window_height) = gfx.size();
        let area = PreviewArea {
            left: self.panel_width,
            width: (window_width as f32 - self.panel_width).max(1.0),
            height: window_height as f32,
        };

        match self.editor.design.view {
            View::Field => self
                .field
                .draw(gfx, frame, &self.editor, area, &mut self.background),
            View::Colony | View::Patch => {
                self.sheet
                    .update(gfx, &self.editor.design, &mut self.background);
                self.sheet.draw(gfx, frame, &self.editor, area);
            }
        }
    }

    /// The egui panel on the left, drawn over the preview. Returns the file operation the
    /// user asked for, if any, and remembers how wide the panel is.
    fn draw_panel(&mut self, gfx: &mut Graphics, frame: &mut Frame) -> Option<Action> {
        let (editor, status) = (&mut self.editor, &self.status);
        let mut action = None;
        let mut panel_width = self.panel_width;

        self.ui.run(gfx, frame, |context| {
            context.root_ui(|root| {
                let shown = egui::Panel::left("params")
                    .default_size(340.0)
                    .show(root, |ui| panel::show(ui, editor, status));
                action = shown.inner;
                panel_width = shown.response.rect.right() * context.pixels_per_point();
            });
        });

        self.panel_width = panel_width;
        action
    }
}

fn main() -> Result<()> {
    // In the browser the toolkit logs to the console itself.
    #[cfg(not(target_arch = "wasm32"))]
    env_logger::init();

    run::<App>(AppConfig {
        title: "Spore Designer".into(),
        width: 1400,
        height: 900,
        // In the browser, draw into the page's `<canvas id="designer">` (see index.html).
        canvas_id: cfg!(target_arch = "wasm32").then(|| "designer".into()),
        ..Default::default()
    })
}
