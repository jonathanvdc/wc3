# Project coding style

- In implementation code, import the types, traits, and functions you need and use their short names. Avoid fully qualified paths such as `crate::Type` or `std::module::function` at call sites unless qualification resolves a real ambiguity or is required by the language.

- Model format code belongs under `wc3::model`; binary MDX codecs belong under `wc3::model::mdx` and text MDL codecs under `wc3::model::mdl`.
- Both codec modules expose `Read` and `Write` traits and derives. Import the modules and use `mdx::Read`, `mdx::Write`, `mdl::Read`, and `mdl::Write` in bounds, implementations, and derives to distinguish formats. For method syntax, import the needed trait as `_`; alias standard I/O traits when needed to avoid ambiguity.
- Codec errors follow the same format-qualified naming: `mdx::ReadError`, `mdx::WriteError`, `mdl::ReadError`, and `mdl::WriteError`.

# Tests

- Keep unit tests that exercise private implementation details in the module they test. When an inline `#[cfg(test)] mod tests` becomes large, move its contents into a dedicated test file and declare it with `#[cfg(test)] #[path = "…"] mod tests;` so the tests retain private access. Put tests of public crate behavior in `crates/*/tests/` as integration tests; keep their fixtures under `crates/*/tests/fixtures/`.

# Rendering references

- When working on `bevy-wc3`, read `crates/bevy-wc3/docs/README.md` and the relevant documents under `crates/bevy-wc3/docs/visual-fidelity/`, including `renderer.md`. Keep these documents and their index updated as part of the work when rendering behavior, implementation gaps, limitations, or verification results change. Distinguish missing features from partial implementations and implemented behavior whose game fidelity remains unverified.

- When implementing or investigating Warcraft III model rendering, consult Retera Model Studio, mdx-m3-viewer, and Warsmash as reference implementations. Compare their behavior for animation, skinning, materials, pass ordering, and effects against models in `data/`; do not assume any one implementation is definitive.

# Rendering captures and synthetic models

- Reuse `crates/bevy-wc3/examples/capture.rs` for offscreen rendering checks. It loads an MDX or MDL through `Wc3BevyPlugin`, simulates from time zero in steps of at most `1 / FPS` seconds to reach the requested times exactly, and writes PNGs after assets, pipelines, and GPU readback are ready. Do not replace it with a throwaway capture program.
- For focused rendering cases, create a small, complete MDL in a temporary directory that isolates the behavior under investigation: geometry, materials, animation, skinning, or effects. Start from an appropriate fixture in `crates/wc3/tests/fixtures/mdl/` or `crates/bevy-wc3/tests/fixtures/`, or author the model directly, and include any required textures. Keep node IDs, parent relationships, pivot points, bone/matrix references, sequence intervals, material IDs, and texture IDs consistent. Set model bounds to include the rendered content, or pass an explicit camera. Vary the relevant inputs while keeping the rest of the scene controlled, then compare captures at the same times and camera settings.
- `bevy-wc3` supports `.mdx` and `.mdl` assets. `Wc3Model::decode` detects MDX's `MDLX` header and otherwise reads UTF-8 MDL; `decode_mdx` and `decode_mdl` select a format explicitly. Both formats use strict conversion to the runtime model version. Synthetic MDL files can be rendered directly without first encoding MDX.

Capture the checked-in particle fixture:

```sh
cargo run -p bevy-wc3 --example capture -- \
  crates/bevy-wc3/tests/fixtures/particle_capture.mdl \
  /tmp/wc3-particle-captures \
  --times 0,0.5,1,2 --fps 60 --size 640x480 \
  --eye 0,-18,8 --target 0,0,2
```

Capture a game model with textures resolved from a shared asset root:

```sh
cargo run -p bevy-wc3 --example capture -- \
  /path/to/assets/units/model.mdx /tmp/wc3-model-captures \
  --asset-root /path/to/assets --sequence 0 --times 0.25,1,2
```

- Run `capture --help` through Cargo for all options. Capture times are strictly increasing seconds from the start of the selected sequence; sequence indices start at zero. The default is one capture at 1 second, 60 simulation steps per second, and 640x480 pixels. Without camera options, the camera frames the model bounds with Z up.
- Texture paths resolve beside the model first, then from `--asset-root` (which defaults to the model's directory). `--bitmap INDEX=PATH`, `--particle2 INDEX=PATH`, and `--replaceable ID=PATH` override textures; override paths are relative to that asset root.
- PNGs are named `frame-0000-0.000s.png`, etc., in the output directory; reruns overwrite matching files. Inspect the generated images to verify placement, motion, color/alpha, atlas frames, geometry, and blending. Successful compilation or a nonempty image alone does not establish rendering fidelity.
- The program renders offscreen but requires a GPU. If the sandbox cannot find an adapter, use permitted GPU access for the same command. Loading, pipeline, and screenshot failures terminate with an error; do not report them as successful visual checks.
- Use the same simulation FPS when comparing captures. `Wc3Animation::seek` skips particle spawning and event dispatch over the seek interval and produces a different live particle population. Loading, render warmup, and screenshot readback use zero simulation delta, so wall-clock delays do not advance the animation.
