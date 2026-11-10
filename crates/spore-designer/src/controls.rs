//! The parameter panel. Edits `Editor` in place; the app repaints when the design changes.
//!
//! The mould's fields are listed once, in [`mold`], against the [`Form`] trait. Drawing the
//! panel and randomizing the design are two `Form`s, so they always share the same ranges.

use std::ops::RangeInclusive;

use game_toolkit::prelude::egui;
use spore_core::noise::Rng;
use spore_core::{ColorStop, MoldParams, Rgb, presets};

use crate::{Action, Editor, View};

/// Something that visits every editable field of a mould.
pub trait Form {
    fn section(&mut self, title: &str, body: &mut dyn FnMut(&mut dyn Form));
    fn float(&mut self, label: &str, v: &mut f32, range: RangeInclusive<f32>);
    fn int(&mut self, label: &str, v: &mut u32, range: RangeInclusive<u32>);
    fn toggle(&mut self, label: &str, v: &mut bool);
    fn color(&mut self, label: &str, v: &mut Rgb);
    fn gradient(&mut self, stops: &mut Vec<ColorStop>);
}

pub fn panel(ui: &mut egui::Ui, ed: &mut Editor, status: &str) -> Option<Action> {
    let mut action = None;
    ui.heading("Spore Designer");
    ui.horizontal_wrapped(|ui| {
        if ui.button("Load…").clicked() {
            action = Some(Action::Load);
        }
        if ui.button("Save…").clicked() {
            action = Some(Action::Save);
        }
        if ui.button("Export stage…").clicked() {
            action = Some(Action::ExportStage);
        }
        if ui.button("Export sheet…").clicked() {
            action = Some(Action::ExportSheet);
        }
    });
    if !status.is_empty() {
        ui.small(status);
    }
    ui.separator();

    ui.horizontal_wrapped(|ui| {
        ui.label("Preset:");
        for (name, preset) in presets::ALL {
            if ui.button(*name).clicked() {
                ed.design.mold = preset();
            }
        }
    });
    if ui
        .button("Randomize everything")
        .on_hover_text("New random value for every parameter and the seed")
        .clicked()
    {
        let mut r = Randomizer(Rng(ed.dice));
        mold(&mut r, &mut ed.design.mold);
        ed.design.seed = r.0.next_u32() >> 8;
        ed.dice = r.0.0;
    }
    let d = &mut ed.design;
    ui.horizontal(|ui| {
        ui.label("Seed");
        ui.add(egui::DragValue::new(&mut d.seed));
        if ui.button("Random").on_hover_text("Random seed").clicked() {
            d.seed = d.seed.wrapping_mul(747_796_405).wrapping_add(2_891_336_453) >> 8;
        }
    });
    ui.add(egui::Slider::new(&mut d.size, 32..=512).text("size px"));
    ui.add(egui::Slider::new(&mut d.stages, 1..=24).text("stages"));

    ui.horizontal(|ui| {
        let label = if ed.playing { "Pause" } else { "Play" };
        if ui.button(label).clicked() {
            ed.playing = !ed.playing;
        }
        // The playhead is fractional; the slider shows whole stages and only writes back
        // when dragged (a stepped slider would snap the playhead back every frame).
        let mut shown = ed.stage as usize;
        if ui
            .add(egui::Slider::new(&mut shown, 0..=d.stages - 1).text("stage"))
            .changed()
        {
            ed.stage = shown as f32;
        }
    });
    ui.add(egui::Slider::new(&mut ed.fps, 1.0..=30.0).text("fps"));
    let mut form = Panel { ui };
    form.color("background", &mut ed.background);
    form.ui.horizontal(|ui| {
        ui.label("View:");
        ui.selectable_value(&mut d.view, View::Colony, "Colony");
        ui.selectable_value(&mut d.view, View::Patch, "Patch");
        ui.selectable_value(&mut d.view, View::Field, "Endless field");
    });
    match d.view {
        View::Colony => {}
        View::Patch => {
            let p = &mut d.patch;
            form.int("colonies", &mut p.count, 1..=40);
            form.float("area (colony widths)", &mut p.area, 1.0..=6.0);
            form.float("stagger", &mut p.stagger, 0.0..=0.95);
        }
        View::Field => {
            let f = &mut d.field;
            form.float("spacing (colony widths)", &mut f.spacing, 0.4..=3.0);
            form.float("jitter", &mut f.jitter, 0.0..=1.0);
            form.float("density", &mut f.density, 0.0..=1.0);
            form.float("stagger", &mut f.stagger, 0.0..=0.95);
            form.float("pan speed (px/s)", &mut ed.pan_speed, 0.0..=400.0);
            form.float("pan direction (deg)", &mut ed.pan_angle, 0.0..=360.0);
            form.ui.small("Drag the preview to move around.");
        }
    }
    form.ui.separator();
    mold(&mut form, &mut d.mold);
    action
}

/// Every editable field of a mould, by section.
fn mold(f: &mut dyn Form, m: &mut MoldParams) {
    f.section("Shape", &mut |f| {
        let s = &mut m.shape;
        f.float("radius base", &mut s.radius_base, 0.0..=16.0);
        f.float("radius growth", &mut s.radius_growth, 0.0..=28.0);
        f.float("wobble base", &mut s.wobble_base, 0.3..=1.2);
        f.float("wobble amount", &mut s.wobble_amp, 0.0..=0.8);
        f.float("wobble freq", &mut s.wobble_freq, 0.2..=6.0);
        f.int("lobes", &mut s.lobes, 0..=16);
        f.float("lobe depth", &mut s.lobe_amp, 0.0..=0.4);
        f.toggle("random rotation", &mut s.spin);
    });
    f.section("Body", &mut |f| {
        let b = &mut m.body;
        f.gradient(&mut b.gradient);
        f.float("softness (velvet to cotton)", &mut b.softness, 0.0..=1.0);
        f.float("fluff scale", &mut b.fluff_scale, 0.5..=8.0);
        f.float("fluff darkest", &mut b.fluff_low, 0.0..=1.0);
        f.float("youth", &mut b.youth, 0.0..=3.0);
        f.color("young colour", &mut b.young);
        f.float("age", &mut b.age, 0.0..=1.0);
        f.color("aged colour", &mut b.aged);
    });
    f.section("Zones", &mut |f| {
        let z = &mut m.zones;
        f.float("amount", &mut z.amount, 0.0..=1.0);
        f.float("frequency", &mut z.freq, 0.0..=30.0);
        f.float("start", &mut z.start, 0.0..=1.0);
        f.float("end", &mut z.end, 0.0..=1.0);
        f.color("colour", &mut z.rgb);
    });
    f.section("Dust", &mut |f| {
        let x = &mut m.dust;
        f.float("amount", &mut x.amount, 0.0..=1.0);
        f.float("scale", &mut x.scale, 0.5..=12.0);
        f.float("low", &mut x.low, 0.0..=1.0);
        f.float("high", &mut x.high, 0.0..=1.0);
        f.float("extent", &mut x.extent, 0.0..=1.0);
        f.color("colour", &mut x.rgb);
    });
    f.section("Furrows", &mut |f| {
        let x = &mut m.furrows;
        f.float("depth", &mut x.depth, 0.0..=1.0);
        f.float("count", &mut x.count, 0.0..=40.0);
        f.float("sharpness", &mut x.sharpness, 0.5..=16.0);
    });
    f.section("Lighting", &mut |f| {
        let l = &mut m.lighting;
        f.float("dome", &mut l.dome, 0.0..=1.5);
        f.float("grain", &mut l.grain, 0.0..=1.0);
        f.float("grain scale", &mut l.grain_scale, 0.3..=6.0);
    });
    f.section("Specks", &mut |f| {
        let s = &mut m.specks;
        f.float("amount", &mut s.amount, 0.0..=0.5);
        f.float("bright", &mut s.bright, 1.0..=2.0);
        f.float("dark", &mut s.dark, 0.0..=1.0);
        f.float("extent", &mut s.extent, 0.0..=1.0);
    });
    f.section("Colour variance", &mut |f| {
        let v = &mut m.variance;
        f.float("hue drift (turns)", &mut v.hue, 0.0..=0.5);
        f.float("saturation", &mut v.saturation, 0.0..=2.5);
        f.float("scale", &mut v.scale, 1.0..=40.0);
    });
    f.section("Rosettes", &mut |f| {
        let r = &mut m.rosettes;
        f.toggle("enabled", &mut r.enabled);
        f.int("count", &mut r.count, 0..=48);
        f.float("size min", &mut r.size_min, 0.02..=1.0);
        f.float("size max", &mut r.size_max, 0.02..=1.0);
        f.float("spread", &mut r.spread, 0.0..=1.5);
        f.gradient(&mut r.gradient);
        f.float("rings", &mut r.rings, 0.0..=8.0);
        f.float("blotch", &mut r.blotch, 0.0..=4.0);
        f.float("hue jitter", &mut r.hue_jitter, 0.0..=0.5);
        f.float("rim dark", &mut r.rim_dark, 0.0..=1.0);
        f.float("wobble", &mut r.wobble, 0.0..=1.0);
        f.float("alpha", &mut r.alpha, 0.0..=1.0);
    });
    f.section("Fringe", &mut |f| {
        let x = &mut m.fringe;
        f.toggle("enabled", &mut x.enabled);
        f.float("width", &mut x.width, 0.01..=1.0);
        f.float("alpha", &mut x.alpha, 0.0..=1.0);
        f.float("scale", &mut x.scale, 0.3..=6.0);
        f.color("colour", &mut x.rgb);
    });
    f.section("Hyphae", &mut |f| {
        let h = &mut m.hyphae;
        f.toggle("enabled", &mut h.enabled);
        f.float("base count", &mut h.base, 0.0..=200.0);
        f.float("growth count", &mut h.growth, 0.0..=400.0);
        f.float("jitter", &mut h.jitter, 0.0..=1.5);
        f.float("length", &mut h.length, 0.1..=3.0);
        f.float("variance", &mut h.variance, 0.0..=1.0);
        f.float("alpha", &mut h.alpha, 0.0..=1.0);
        f.float("outer alpha", &mut h.outer_alpha, 0.0..=1.0);
        f.color("colour", &mut h.rgb);
    });
    f.section("Sporangia", &mut |f| {
        let s = &mut m.sporangia;
        f.toggle("enabled", &mut s.enabled);
        f.float("count", &mut s.count, 0.0..=200.0);
        f.float("ripe chance", &mut s.ripe_chance, 0.0..=1.0);
        f.float("size min", &mut s.size_min, 0.3..=4.0);
        f.float("size max", &mut s.size_max, 0.3..=4.0);
        f.color("ripe", &mut s.ripe);
        f.color("unripe", &mut s.unripe);
        f.color("stalk", &mut s.stalk);
    });
    f.section("Drops", &mut |f| {
        let x = &mut m.drops;
        f.toggle("enabled", &mut x.enabled);
        f.float("from growth", &mut x.min_growth, 0.0..=1.0);
        f.int("count", &mut x.count, 0..=20);
        f.float("size min", &mut x.size_min, 0.3..=4.0);
        f.float("size max", &mut x.size_max, 0.3..=4.0);
        f.color("colour", &mut x.rgb);
    });
    f.section("Glisten", &mut |f| {
        let g = &mut m.glisten;
        f.toggle("enabled", &mut g.enabled);
        f.float("sheen", &mut g.sheen, 0.0..=1.0);
        f.float("power", &mut g.power, 1.0..=120.0);
        f.float("beads base", &mut g.beads_base, 0.0..=20.0);
        f.float("beads growth", &mut g.beads_growth, 0.0..=40.0);
    });
    f.section("Slime", &mut |f| {
        let s = &mut m.slime;
        f.toggle("enabled", &mut s.enabled);
        f.float("centre size", &mut s.size, 0.0..=32.0);
        f.int("lobes", &mut s.lobes, 0..=16);
        f.float("lobe reach", &mut s.reach, 0.0..=28.0);
        f.float("reach jitter", &mut s.reach_jitter, 0.0..=20.0);
        f.float("lobe size", &mut s.lobe_size, 0.0..=16.0);
        f.float("lobe jitter", &mut s.lobe_jitter, 0.0..=12.0);
        f.float("ripple", &mut s.ripple, 0.0..=1.0);
        f.color("deep", &mut s.deep);
        f.color("light", &mut s.light);
        f.float("alpha", &mut s.alpha, 0.0..=1.0);
        f.float("depth alpha", &mut s.depth_alpha, 0.0..=1.0);
        f.float("gloss", &mut s.gloss, 1.0..=200.0);
        f.float("meniscus", &mut s.meniscus, 0.0..=1.0);
        f.float("bubbles", &mut s.bubbles, 0.0..=0.2);
        f.color("bubble colour", &mut s.bubble);
    });
}

/// Draws the fields as egui widgets.
struct Panel<'a> {
    ui: &'a mut egui::Ui,
}

impl Form for Panel<'_> {
    fn section(&mut self, title: &str, body: &mut dyn FnMut(&mut dyn Form)) {
        egui::CollapsingHeader::new(title).show(self.ui, |ui| body(&mut Panel { ui }));
    }

    fn float(&mut self, label: &str, v: &mut f32, range: RangeInclusive<f32>) {
        self.ui.add(egui::Slider::new(v, range).text(label));
    }

    fn int(&mut self, label: &str, v: &mut u32, range: RangeInclusive<u32>) {
        self.ui.add(egui::Slider::new(v, range).text(label));
    }

    fn toggle(&mut self, label: &str, v: &mut bool) {
        self.ui.checkbox(v, label);
    }

    fn color(&mut self, label: &str, rgb: &mut Rgb) {
        self.ui.horizontal(|ui| {
            color_button(ui, rgb);
            ui.label(label);
        });
    }

    fn gradient(&mut self, stops: &mut Vec<ColorStop>) {
        let ui = &mut *self.ui;
        ui.label("Gradient (centre to edge)");
        let mut remove = None;
        for (i, stop) in stops.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                color_button(ui, &mut stop.rgb);
                ui.add(egui::Slider::new(&mut stop.t, 0.0..=1.0));
                if ui.small_button("x").clicked() {
                    remove = Some(i);
                }
            });
        }
        if let Some(i) = remove {
            stops.remove(i);
        }
        if ui.small_button("+ stop").clicked() {
            let rgb = stops.last().map_or([255.0; 3], |s| s.rgb);
            stops.push(ColorStop { t: 1.0, rgb });
        }
    }
}

/// Edit an RGB 0..255 colour.
fn color_button(ui: &mut egui::Ui, rgb: &mut Rgb) {
    let mut c = rgb.map(|v| v / 255.0);
    if ui.color_edit_button_rgb(&mut c).changed() {
        *rgb = c.map(|v| v * 255.0);
    }
}

/// Sets every field to a uniform random value in its range.
struct Randomizer(Rng);

impl Form for Randomizer {
    fn section(&mut self, _title: &str, body: &mut dyn FnMut(&mut dyn Form)) {
        body(self);
    }

    fn float(&mut self, _label: &str, v: &mut f32, range: RangeInclusive<f32>) {
        *v = self.0.range(*range.start(), *range.end());
    }

    fn int(&mut self, _label: &str, v: &mut u32, range: RangeInclusive<u32>) {
        let span = range.end() - range.start() + 1;
        *v = range.start() + self.0.next_u32() % span;
    }

    fn toggle(&mut self, _label: &str, v: &mut bool) {
        *v = self.0.f() < 0.5;
    }

    fn color(&mut self, _label: &str, v: &mut Rgb) {
        *v = [0; 3].map(|_| self.0.range(0.0, 255.0));
    }

    /// Same number of stops, spread from 0 to 1 at random positions, random colours.
    fn gradient(&mut self, stops: &mut Vec<ColorStop>) {
        let n = stops.len();
        for (i, stop) in stops.iter_mut().enumerate() {
            stop.t = match i {
                0 => 0.0,
                _ if i == n - 1 => 1.0,
                _ => self.0.f(),
            };
            self.color("", &mut stop.rgb);
        }
        stops.sort_by(|a, b| a.t.total_cmp(&b.t));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Asserts every field lies within its range.
    struct InRange;

    impl Form for InRange {
        fn section(&mut self, _: &str, body: &mut dyn FnMut(&mut dyn Form)) {
            body(self);
        }
        fn float(&mut self, _: &str, v: &mut f32, r: RangeInclusive<f32>) {
            assert!(r.contains(v), "{v} outside {r:?}");
        }
        fn int(&mut self, _: &str, v: &mut u32, r: RangeInclusive<u32>) {
            assert!(r.contains(v), "{v} outside {r:?}");
        }
        fn toggle(&mut self, _: &str, _: &mut bool) {}
        fn color(&mut self, _: &str, v: &mut Rgb) {
            assert!(v.iter().all(|c| (0.0..=255.0).contains(c)));
        }
        fn gradient(&mut self, stops: &mut Vec<ColorStop>) {
            assert!(stops.windows(2).all(|w| w[0].t <= w[1].t));
        }
    }

    #[test]
    fn randomize_stays_in_range_and_changes_things() {
        let mut m = presets::penicillium();
        let mut r = Randomizer(Rng(0x1234_5678_9ABC_DEF1));
        for _ in 0..50 {
            mold(&mut r, &mut m);
            mold(&mut InRange, &mut m);
            // Every random design must still paint.
            let _ = spore_core::render(&m, 1, 1.0, 32);
        }
        assert!(m != presets::penicillium());
    }

    /// Strongest alpha on the outermost ring of pixels.
    fn edge_alpha(img: &spore_core::Rgba) -> f32 {
        let w = img.w;
        (0..w)
            .flat_map(|i| [(i, 0), (i, w - 1), (0, i), (w - 1, i)])
            .map(|(x, y)| img.px[y * w + x][3])
            .fold(0.0, f32::max)
    }

    #[test]
    fn random_designs_are_never_cut_off() {
        let mut r = Randomizer(Rng(0x9E37_79B9_7F4A_7C15));
        let mut m = presets::penicillium();
        for i in 0..300 {
            mold(&mut r, &mut m);
            for g in [0.5, 1.0] {
                let img = spore_core::render(&m, i, g, 32);
                assert!(
                    edge_alpha(&img) < 0.004,
                    "design {i} cut off at g={g}: {m:?}"
                );
            }
            let patch = spore_core::Patch {
                count: 1 + r.0.next_u32() % 6,
                area: r.0.range(1.0, 3.0),
                stagger: r.0.f(),
            };
            let img = spore_core::render_patch(&m, &patch, i, 1.0, 40);
            assert!(
                edge_alpha(&img) < 0.004,
                "patch {i} cut off: {patch:?} {m:?}"
            );
        }
    }
}
