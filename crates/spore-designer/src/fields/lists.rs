//! The lists: every value of a mould, a patch and a field, with its range.

use spore_core::{Field, MoldParams, Patch};

use super::Form;

/// Patch layout values.
pub fn patch(form: &mut dyn Form, patch: &mut Patch) {
    form.int("colonies", &mut patch.count, 1..=40);
    form.float("area (colony widths)", &mut patch.area, 1.0..=6.0);
    form.float("stagger", &mut patch.stagger, 0.0..=0.95);
}

/// Endless field layout values.
pub fn field(form: &mut dyn Form, field: &mut Field) {
    form.float("spacing (colony widths)", &mut field.spacing, 0.4..=3.0);
    form.float("jitter", &mut field.jitter, 0.0..=1.0);
    form.float("density", &mut field.density, 0.0..=1.0);
    form.float("stagger", &mut field.stagger, 0.0..=0.95);
}

/// Every mould value, by section, in panel order.
pub fn mold(form: &mut dyn Form, mold: &mut MoldParams) {
    form.section("Shape", &mut |form| {
        let shape = &mut mold.shape;
        form.float("radius base", &mut shape.radius_base, 0.0..=16.0);
        form.float("radius growth", &mut shape.radius_growth, 0.0..=28.0);
        form.float("wobble base", &mut shape.wobble_base, 0.3..=1.2);
        form.float("wobble amount", &mut shape.wobble_amp, 0.0..=0.8);
        form.float("wobble freq", &mut shape.wobble_freq, 0.2..=6.0);
        form.int("lobes", &mut shape.lobes, 0..=16);
        form.float("lobe depth", &mut shape.lobe_amp, 0.0..=0.4);
        form.toggle("random rotation", &mut shape.spin);
        form.float("meld (patch and field)", &mut shape.meld, 0.0..=24.0);
    });

    form.section("Body", &mut |form| {
        let body = &mut mold.body;
        form.gradient(&mut body.gradient);
        form.float("softness (velvet to cotton)", &mut body.softness, 0.0..=1.0);
        form.float("fluff scale", &mut body.fluff_scale, 0.5..=8.0);
        form.float("fluff darkest", &mut body.fluff_low, 0.0..=1.0);
        form.float("youth", &mut body.youth, 0.0..=3.0);
        form.color("young colour", &mut body.young);
        form.float("age", &mut body.age, 0.0..=1.0);
        form.color("aged colour", &mut body.aged);
    });

    form.section("Zones", &mut |form| {
        let zones = &mut mold.zones;
        form.float("amount", &mut zones.amount, 0.0..=1.0);
        form.float("frequency", &mut zones.freq, 0.0..=30.0);
        form.float("start", &mut zones.start, 0.0..=1.0);
        form.float("end", &mut zones.end, 0.0..=1.0);
        form.color("colour", &mut zones.rgb);
    });

    form.section("Dust", &mut |form| {
        let dust = &mut mold.dust;
        form.float("amount", &mut dust.amount, 0.0..=1.0);
        form.float("scale", &mut dust.scale, 0.5..=12.0);
        form.float("low", &mut dust.low, 0.0..=1.0);
        form.float("high", &mut dust.high, 0.0..=1.0);
        form.float("extent", &mut dust.extent, 0.0..=1.0);
        form.color("colour", &mut dust.rgb);
    });

    form.section("Furrows", &mut |form| {
        let furrows = &mut mold.furrows;
        form.float("depth", &mut furrows.depth, 0.0..=1.0);
        form.float("count", &mut furrows.count, 0.0..=40.0);
        form.float("sharpness", &mut furrows.sharpness, 0.5..=16.0);
    });

    form.section("Lighting", &mut |form| {
        let lighting = &mut mold.lighting;
        form.float("dome", &mut lighting.dome, 0.0..=1.5);
        form.float("grain", &mut lighting.grain, 0.0..=1.0);
        form.float("grain scale", &mut lighting.grain_scale, 0.3..=6.0);
    });

    form.section("Specks", &mut |form| {
        let specks = &mut mold.specks;
        form.float("amount", &mut specks.amount, 0.0..=0.5);
        form.float("bright", &mut specks.bright, 1.0..=2.0);
        form.float("dark", &mut specks.dark, 0.0..=1.0);
        form.float("extent", &mut specks.extent, 0.0..=1.0);
    });

    form.section("Colour variance", &mut |form| {
        let variance = &mut mold.variance;
        form.float("hue drift (turns)", &mut variance.hue, 0.0..=0.5);
        form.float("saturation", &mut variance.saturation, 0.0..=2.5);
        form.float("scale", &mut variance.scale, 1.0..=40.0);
    });

    form.section("Rosettes", &mut |form| {
        let rosettes = &mut mold.rosettes;
        form.toggle("enabled", &mut rosettes.enabled);
        form.int("count", &mut rosettes.count, 0..=48);
        form.float("size min", &mut rosettes.size_min, 0.02..=1.0);
        form.float("size max", &mut rosettes.size_max, 0.02..=1.0);
        form.float("spread", &mut rosettes.spread, 0.0..=1.5);
        form.gradient(&mut rosettes.gradient);
        form.float("rings", &mut rosettes.rings, 0.0..=8.0);
        form.float("blotch", &mut rosettes.blotch, 0.0..=4.0);
        form.float("hue jitter", &mut rosettes.hue_jitter, 0.0..=0.5);
        form.float("rim dark", &mut rosettes.rim_dark, 0.0..=1.0);
        form.float("wobble", &mut rosettes.wobble, 0.0..=1.0);
        form.float("alpha", &mut rosettes.alpha, 0.0..=1.0);
    });

    form.section("Fringe", &mut |form| {
        let fringe = &mut mold.fringe;
        form.toggle("enabled", &mut fringe.enabled);
        form.float("width", &mut fringe.width, 0.01..=1.0);
        form.float("alpha", &mut fringe.alpha, 0.0..=1.0);
        form.float("scale", &mut fringe.scale, 0.3..=6.0);
        form.color("colour", &mut fringe.rgb);
    });

    form.section("Hyphae", &mut |form| {
        let hyphae = &mut mold.hyphae;
        form.toggle("enabled", &mut hyphae.enabled);
        form.float("base count", &mut hyphae.base, 0.0..=200.0);
        form.float("growth count", &mut hyphae.growth, 0.0..=400.0);
        form.float("jitter", &mut hyphae.jitter, 0.0..=1.5);
        form.float("length", &mut hyphae.length, 0.1..=3.0);
        form.float("variance", &mut hyphae.variance, 0.0..=1.0);
        form.float("alpha", &mut hyphae.alpha, 0.0..=1.0);
        form.float("outer alpha", &mut hyphae.outer_alpha, 0.0..=1.0);
        form.color("colour", &mut hyphae.rgb);
    });

    form.section("Sporangia", &mut |form| {
        let sporangia = &mut mold.sporangia;
        form.toggle("enabled", &mut sporangia.enabled);
        form.float("count", &mut sporangia.count, 0.0..=200.0);
        form.float("ripe chance", &mut sporangia.ripe_chance, 0.0..=1.0);
        form.float("size min", &mut sporangia.size_min, 0.3..=4.0);
        form.float("size max", &mut sporangia.size_max, 0.3..=4.0);
        form.color("ripe", &mut sporangia.ripe);
        form.color("unripe", &mut sporangia.unripe);
        form.color("stalk", &mut sporangia.stalk);
    });

    form.section("Drops", &mut |form| {
        let drops = &mut mold.drops;
        form.toggle("enabled", &mut drops.enabled);
        form.float("from growth", &mut drops.min_growth, 0.0..=1.0);
        form.int("count", &mut drops.count, 0..=20);
        form.float("size min", &mut drops.size_min, 0.3..=4.0);
        form.float("size max", &mut drops.size_max, 0.3..=4.0);
        form.color("colour", &mut drops.rgb);
    });

    form.section("Glisten", &mut |form| {
        let glisten = &mut mold.glisten;
        form.toggle("enabled", &mut glisten.enabled);
        form.float("sheen", &mut glisten.sheen, 0.0..=1.0);
        form.float("power", &mut glisten.power, 1.0..=120.0);
        form.float("beads base", &mut glisten.beads_base, 0.0..=20.0);
        form.float("beads growth", &mut glisten.beads_growth, 0.0..=40.0);
    });

    form.section("Slime", &mut |form| {
        let slime = &mut mold.slime;
        form.toggle("enabled", &mut slime.enabled);
        form.float("centre size", &mut slime.size, 0.0..=32.0);
        form.int("lobes", &mut slime.lobes, 0..=16);
        form.float("lobe reach", &mut slime.reach, 0.0..=28.0);
        form.float("reach jitter", &mut slime.reach_jitter, 0.0..=20.0);
        form.float("lobe size", &mut slime.lobe_size, 0.0..=16.0);
        form.float("lobe jitter", &mut slime.lobe_jitter, 0.0..=12.0);
        form.float("ripple", &mut slime.ripple, 0.0..=1.0);
        form.color("deep", &mut slime.deep);
        form.color("light", &mut slime.light);
        form.float("alpha", &mut slime.alpha, 0.0..=1.0);
        form.float("depth alpha", &mut slime.depth_alpha, 0.0..=1.0);
        form.float("gloss", &mut slime.gloss, 1.0..=200.0);
        form.float("meniscus", &mut slime.meniscus, 0.0..=1.0);
        form.float("bubbles", &mut slime.bubbles, 0.0..=0.2);
        form.color("bubble colour", &mut slime.bubble);
    });
}
