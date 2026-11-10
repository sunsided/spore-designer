//! Spore Designer: tweak a mould's parameters and watch it grow.
//!
//! The parameter panel is egui (via game-toolkit's `ui` overlay). The preview is all growth
//! stages, painted on the CPU by `spore-core`, uploaded as one sprite-sheet texture and drawn
//! behind the panel: the current stage large, the whole sheet as a filmstrip below.

mod controls;
mod field_view;

use std::path::Path;

use game_toolkit::prelude::*;
use serde::{Deserialize, Serialize};
use spore_core::{
    Field, MoldParams, Patch, Rgb, Rgba, presets, render, render_patch, sheet, stage_growth,
};

use field_view::FieldView;

/// What the preview shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum View {
    /// One colony.
    #[default]
    Colony,
    /// A few colonies growing together on one square.
    Patch,
    /// An endless field of colonies to pan across.
    Field,
}

/// What a saved `.ron` file holds: the mould plus how it was rendered.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Design {
    pub seed: u32,
    pub size: usize,
    pub stages: usize,
    pub mold: MoldParams,
    #[serde(default)]
    pub view: View,
    #[serde(default)]
    pub patch: Patch,
    #[serde(default)]
    pub field: Field,
}

impl Design {
    /// One stage at growth `g`: a patch in the patch view, else a lone colony (the field
    /// view paints its own chunks).
    pub fn paint(&self, g: f32) -> Rgba {
        match self.view {
            View::Patch => render_patch(&self.mold, &self.patch, self.seed, g, self.size),
            View::Colony | View::Field => render(&self.mold, self.seed, g, self.size),
        }
    }
}

/// Everything the panel edits.
pub struct Editor {
    pub design: Design,
    /// Current stage, fractional while playing.
    pub stage: f32,
    pub playing: bool,
    pub fps: f32,
    pub background: Rgb,
    /// State of the "Randomize everything" RNG.
    pub dice: u64,
    /// Field view camera: canvas pixel at the centre of the preview.
    pub cam: [f32; 2],
    /// Field view auto-pan, pixels per second and direction in degrees.
    pub pan_speed: f32,
    pub pan_angle: f32,
}

/// File operations requested by the panel, run after the egui pass.
pub enum Action {
    Save,
    Load,
    ExportStage,
    ExportSheet,
}

struct App {
    ui: Ui,
    ed: Editor,
    /// The design `frames` were painted from.
    painted: Option<Design>,
    frames: Vec<Rgba>,
    texture: Option<(TextureId, [u32; 2])>,
    field: FieldView,
    /// A drag in the field view started on the preview (not on the panel).
    dragging: bool,
    /// Right edge of the egui panel in physical pixels, from the previous frame.
    panel_px: f32,
    status: String,
}

const FILMSTRIP: f32 = 0.18;

impl Game for App {
    fn init(ctx: &mut Context) -> Result<Self> {
        Ok(Self {
            ui: Ui::new(&ctx.gfx)?,
            ed: Editor {
                design: Design {
                    seed: 1,
                    size: 128,
                    stages: 10,
                    mold: presets::penicillium(),
                    view: View::Colony,
                    patch: Patch::default(),
                    field: Field::default(),
                },
                stage: 9.0,
                playing: false,
                fps: 8.0,
                background: [62.0, 66.0, 72.0],
                dice: 0x2545_F491_4F6C_DD1D,
                cam: [0.0, 0.0],
                pan_speed: 40.0,
                pan_angle: 20.0,
            },
            painted: None,
            frames: Vec::new(),
            texture: None,
            field: FieldView::default(),
            dragging: false,
            panel_px: 0.0,
            status: String::new(),
        })
    }

    fn raw_window_event(&mut self, _ctx: &mut Context, event: &winit::event::WindowEvent) -> bool {
        self.ui.on_window_event(event)
    }

    fn update(&mut self, ctx: &mut Context, dt: f32) {
        let ed = &mut self.ed;
        let stages = ed.design.stages as f32;
        if ed.playing {
            ed.stage = (ed.stage + dt * ed.fps) % stages;
        }
        ed.stage = ed.stage.min(stages - 1.0);
        if ed.design.view == View::Field {
            let a = ed.pan_angle.to_radians();
            ed.cam[0] += a.cos() * ed.pan_speed * dt;
            ed.cam[1] -= a.sin() * ed.pan_speed * dt;
            let input = &ctx.input;
            if input.mouse_pressed(MouseButton::Left) {
                self.dragging = input.mouse_pos_window().0 > self.panel_px;
            }
            if !input.mouse_held(MouseButton::Left) {
                self.dragging = false;
            }
            if self.dragging {
                let (dx, dy) = input.mouse_delta();
                ed.cam = [ed.cam[0] - dx, ed.cam[1] - dy];
            }
        }
    }

    fn render(&mut self, ctx: &mut Context, frame: &mut Frame) {
        if self.painted.as_ref() != Some(&self.ed.design) {
            self.repaint(&mut ctx.gfx);
        }
        self.draw_preview(&mut ctx.gfx, frame);

        let (ed, status) = (&mut self.ed, &self.status);
        let mut action = None;
        let mut panel_px = self.panel_px;
        self.ui.run(&mut ctx.gfx, frame, |ectx| {
            ectx.root_ui(|root| {
                let panel = egui::Panel::left("params")
                    .default_size(340.0)
                    .show(root, |ui| controls::panel(ui, ed, status));
                action = panel.inner;
                panel_px = panel.response.rect.right() * ectx.pixels_per_point();
            });
        });
        self.panel_px = panel_px;
        if let Some(a) = action {
            self.status = match self.run(a) {
                Ok(msg) => msg,
                Err(e) => format!("Error: {e}"),
            };
        }
    }
}

impl App {
    /// Paint every stage (one thread each) and upload them as one sheet texture.
    fn repaint(&mut self, gfx: &mut Graphics) {
        let d = &self.ed.design;
        self.frames = std::thread::scope(|s| {
            let jobs: Vec<_> = (0..d.stages)
                .map(|i| s.spawn(move || d.paint(stage_growth(i, d.stages))))
                .collect();
            jobs.into_iter()
                .map(|j| j.join().expect("painter panicked"))
                .collect()
        });
        let pixels = sheet(&self.frames);
        // Frames are padded past the cell when the design grows out of it.
        let fw = self.frames[0].w;
        let dims = [(fw * d.stages) as u32, fw as u32];
        match self.texture {
            Some((id, size)) if size == dims => gfx.update_texture_rgba(id, &pixels),
            // shortcut: the old texture is not freed (the toolkit has no release call); only
            // size/stage-count changes leak, fine for an editor session.
            _ => {
                let id = gfx.create_texture_rgba(dims[0], dims[1], &pixels, Some("sheet"));
                self.texture = Some((id, dims));
            }
        }
        self.painted = Some(d.clone());
    }

    fn draw_preview(&mut self, gfx: &mut Graphics, frame: &mut Frame) {
        let (w, h) = gfx.size();
        let (w, h) = (w as f32, h as f32);
        let bg = self.ed.background.map(|c| c / 255.0);
        if self.ed.design.view == View::Field {
            // Canvas pixels 1:1; the camera sits at the centre of the area right of the panel.
            let area_w = (w - self.panel_px).max(1.0);
            let origin = [
                self.ed.cam[0] - area_w * 0.5 - self.panel_px,
                self.ed.cam[1] - h * 0.5,
            ];
            let view = [origin[0] + self.panel_px, origin[1], area_w, h];
            let stage = (self.ed.stage as usize).min(self.ed.design.stages - 1);
            let chunks = self.field.prepare(gfx, &self.ed.design, stage, view);
            let side = (self.ed.design.size * spore_core::CHUNK_CELLS) as f32;
            let mut p = frame.painter(gfx);
            p.clear([bg[0], bg[1], bg[2], 1.0]);
            for (tex, pos) in chunks {
                // Whole pixels, so chunk edges meet exactly.
                let at = [(pos[0] - origin[0]).round(), (pos[1] - origin[1]).round()];
                p.sprite(tex, at, [side, side]);
            }
            return;
        }
        let mut p = frame.painter(gfx);
        p.clear([bg[0], bg[1], bg[2], 1.0]);
        let Some((tex, [_, fw])) = self.texture else {
            return;
        };
        let d = self.painted.as_ref().unwrap_or(&self.ed.design);
        let size = fw as f32;
        let (x0, area_w) = (self.panel_px, (w - self.panel_px).max(1.0));
        let strip_h = h * FILMSTRIP;
        let main_h = h - strip_h;

        // Current stage, as large as fits.
        let side = (area_w.min(main_h) * 0.92).max(1.0);
        let stage = (self.ed.stage as usize).min(d.stages - 1);
        let (px, py) = (x0 + (area_w - side) * 0.5, (main_h - side) * 0.5);
        p.sprite_region(
            tex,
            [stage as f32 * size, 0.0],
            [size, size],
            [px, py],
            [side, side],
        );
        // Outline the cell (or patch area) when the growth spills out of it.
        if fw as usize > d.size {
            let cell = side * d.size as f32 / size;
            let (cx, cy) = (px + (side - cell) * 0.5, py + (side - cell) * 0.5);
            let line = [1.0, 1.0, 1.0, 0.25];
            p.rect([cx, cy], [cell, 1.0], line);
            p.rect([cx, cy + cell - 1.0], [cell, 1.0], line);
            p.rect([cx, cy], [1.0, cell], line);
            p.rect([cx + cell - 1.0, cy], [1.0, cell], line);
        }

        // All stages in a row.
        let cell = (area_w / d.stages as f32).min(strip_h * 0.9);
        let strip_w = cell * d.stages as f32;
        let sy = main_h + (strip_h - cell) * 0.5;
        let sx = x0 + (area_w - strip_w) * 0.5;
        p.rect(
            [sx + stage as f32 * cell, sy],
            [cell, cell],
            [1.0, 1.0, 1.0, 0.08],
        );
        p.sprite(tex, [sx, sy], [strip_w, cell]);
    }

    fn run(&mut self, action: Action) -> std::result::Result<String, String> {
        let d = &self.ed.design;
        let ron_filter = |f: rfd::FileDialog| f.add_filter("Mould design", &["ron"]);
        let png_filter = |f: rfd::FileDialog| f.add_filter("PNG image", &["png"]);
        let cancelled = || Ok("Cancelled".to_string());
        match action {
            Action::Save => {
                let Some(path) = ron_filter(rfd::FileDialog::new())
                    .set_file_name("mould.ron")
                    .save_file()
                else {
                    return cancelled();
                };
                let text = ron::ser::to_string_pretty(d, ron::ser::PrettyConfig::default())
                    .map_err(|e| e.to_string())?;
                std::fs::write(&path, text).map_err(|e| e.to_string())?;
                Ok(format!("Saved {}", path.display()))
            }
            Action::Load => {
                let Some(path) = ron_filter(rfd::FileDialog::new()).pick_file() else {
                    return cancelled();
                };
                let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
                let mut design: Design = ron::from_str(&text).map_err(|e| e.to_string())?;
                design.size = design.size.clamp(16, 1024);
                design.stages = design.stages.clamp(1, 32);
                self.ed.design = design;
                Ok(format!("Loaded {}", path.display()))
            }
            Action::ExportStage | Action::ExportSheet if d.view == View::Field => {
                Ok("Export works in the colony and patch views".into())
            }
            Action::ExportStage => {
                let stage = (self.ed.stage as usize).min(self.frames.len().saturating_sub(1));
                let Some(img) = self.frames.get(stage) else {
                    return Ok("Nothing to export".into());
                };
                let Some(path) = png_filter(rfd::FileDialog::new())
                    .set_file_name(format!("mould-stage{}.png", stage + 1))
                    .save_file()
                else {
                    return cancelled();
                };
                save_png(&path, &img.to_rgba8(), img.w, img.w)
            }
            Action::ExportSheet => {
                let Some(path) = png_filter(rfd::FileDialog::new())
                    .set_file_name("mould-sheet.png")
                    .save_file()
                else {
                    return cancelled();
                };
                let w = self.frames.first().map_or(0, |f| f.w);
                save_png(&path, &sheet(&self.frames), w * self.frames.len(), w)
            }
        }
    }
}

fn save_png(path: &Path, rgba: &[u8], w: usize, h: usize) -> std::result::Result<String, String> {
    image::save_buffer(path, rgba, w as u32, h as u32, image::ColorType::Rgba8)
        .map_err(|e| e.to_string())?;
    Ok(format!("Exported {}", path.display()))
}

fn main() -> Result<()> {
    env_logger::init();
    run::<App>(AppConfig {
        title: "Spore Designer".into(),
        width: 1400,
        height: 900,
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn design_file_and_png_roundtrip() {
        let design = Design {
            seed: 42,
            size: 32,
            stages: 3,
            mold: presets::annihilation(),
            view: View::Patch,
            patch: Patch::default(),
            field: Field::default(),
        };
        let text = ron::ser::to_string_pretty(&design, ron::ser::PrettyConfig::default()).unwrap();
        assert!(ron::from_str::<Design>(&text).unwrap() == design);

        let frames: Vec<Rgba> = (0..design.stages)
            .map(|i| design.paint(stage_growth(i, design.stages)))
            .collect();
        let path = std::env::temp_dir().join(format!("spore-sheet-{}.png", std::process::id()));
        let w = frames[0].w;
        assert!(w >= 32, "frames are at least one cell");
        save_png(&path, &sheet(&frames), w * 3, w).unwrap();
        let back = image::open(&path).unwrap().into_rgba8();
        std::fs::remove_file(&path).ok();
        assert_eq!(back.dimensions(), (w as u32 * 3, w as u32));
        assert_eq!(back.as_raw(), &sheet(&frames));
    }
}
