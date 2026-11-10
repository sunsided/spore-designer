# Spore Designer

Design procedural moulds by twiddling parameters, from plain fluff to lurid lichen blooms.
The painters started as the Penicillium and Rhizopus colonies of
[Sporagon](https://github.com/sunsided/sporagon), generalised into one layered generator.

- `crates/spore-core`: pure painter (`MoldParams`, seed, growth → RGBA sprite). No platform
  dependencies; builds for `wasm32-unknown-unknown`.
- `crates/spore-designer`: desktop editor on [game-toolkit](https://game-toolkit.dev/) + egui.
  Presets, live sliders, "randomize everything", growth playback, RON save/load, PNG and
  sprite-sheet export.

Three views:

- **Colony**: one colony in its slime puddle. It may grow past its cell; the image is padded,
  never cut off.
- **Patch**: several colonies on one square, growing at staggered times. Their puddles share
  one metaball field, and with *meld* their bodies fuse into one mass.
- **Endless field**: colonies on an infinite jittered grid, painted in seamless chunks on
  worker threads while the view pans (drag it, or let it drift).

```sh
task run      # or: cargo run --release -p spore-designer
task check    # fmt, clippy, wasm check of the core, tests
```

On Linux the toolkit's audio and gamepad crates need `libasound2-dev libdbus-1-dev libudev-dev`.
