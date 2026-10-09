# Viewer and captures

The examples let you inspect a model interactively, validate its preparation,
or render PNGs at repeatable animation times. The viewer and capture tools load
both MDX and MDL through `Wc3BevyPlugin`, using the same renderer as an application. Run the commands below
from the repository root. For application setup, see [Using bevy-wc3](usage.md).

## Explore a model

Open a local model in the viewer to inspect its geometry and sequences:

```sh
cargo run -p bevy-wc3 --example viewer -- path/to/model.mdx
```

Left drag rotates the camera, right drag pans, the scroll wheel zooms, and Space
cycles sequences. The model's directory is the default asset root. Texture paths
resolve beside the model first, then from that root, using the loader's
[extension fallbacks](usage.md#load-and-spawn-models).

Use `--replaceable ID=PATH`, `--bitmap INDEX=PATH`, or `--particle2 INDEX=PATH`
to override textures relative to the asset root. For example:

```sh
cargo run -p bevy-wc3 --example viewer -- path/to/tree.mdx \
  --replaceable 31=Textures/BlightedTree.blp
```

The viewer defaults to fixed geometry level zero. Use `--lod auto` to switch
among authored levels while zooming. `--lod-bias`, `--lod-thresholds`,
`--lod-hysteresis`, and `--lod-minimum` tune the same quality policy used by the
capture tool; see [geometry LOD](rendering/lod.md).

## Capture animation frames

The offscreen capture example produces PNGs at increasing simulation times.
This attachment fixture provides a small starting point with an explicit camera:

```sh
cargo run -p bevy-wc3 --example capture -- \
  crates/bevy-wc3/tests/fixtures/attachment_capture.mdl /tmp/wc3-attachments \
  --times 0,0.25,0.75,1.25 --fps 60 --size 640x480 \
  --eye 0,-18,12 --target 0,0,0
```

The default is one capture at 1 second, 60 simulation steps per second, and
640x480 pixels. Without camera options, the camera frames model bounds with Z up.
Use `--asset-root PATH` for textures shared across model directories; the same
texture override options as the viewer are available. An authored model camera
can replace the explicit eye/target; see [model cameras](rendering/cameras.md#offscreen-captures).

Capture times are nonnegative, strictly increasing seconds from the initial
sequence start. Select a sequence with `--sequence INDEX`; indices start at zero.
The tool simulates from time
zero in steps of at most `1 / FPS`, splitting steps to reach requested times
exactly. Loading, pipeline warmup, and GPU readback use zero simulation delta,
so wall-clock delays do not advance animation. Seeking directly to a time skips
particle births and events and produces a different effect population.

Images are named `frame-0000-0.000s.png`, and so on; reruns overwrite matching
files. The tool requires a GPU even though it does not open a window. Loading,
pipeline, or readback failures terminate with an error.

Use `--bake-pose` to render static meshes and material snapshots from the offline
baker at each capture time. The same camera, textures, authored LOD selection,
lighting, and prepass options apply, making paired captures useful for comparing
live GPU skinning with CPU-baked geometry. This mode excludes the live hierarchy
and its effects from rendering and rejects scheduled `--play` transitions. See
[static pose baking](usage.md#bake-a-static-pose) for the snapshot's scope.

## Compare rendering behavior

Use identical simulation FPS, times, and camera settings when comparing model
variants or renderer revisions. Inspect placement, motion, color and alpha,
atlas frames, geometry, and blending in the resulting images. Compilation or a
nonempty image alone does not establish rendering fidelity; controlled fixtures
also do not establish equivalence to Warcraft III.

For focused comparisons, use a small complete MDL that isolates the behavior
under investigation, with consistent node IDs, pivots, geometry references,
sequence intervals, materials, and textures. Start from a fixture under
`crates/bevy-wc3/tests/fixtures/` or `crates/wc3/tests/fixtures/mdl/`, and keep the
camera and unrelated scene inputs fixed. Model bounds must encompass the content,
or the capture must use an explicit camera.

[Animation blending](rendering/animation-blending.md#scheduled-playback-in-captures)
shows scheduled sequence switches, and [LOD captures](rendering/lod.md#capture-levels)
shows fixed and automatic geometry selection. Run `capture --help` through Cargo
for the complete set of options.

## Validate without rendering

The `inspect` and `compile` examples inspect decoded data and prepared assets
without opening a GPU window. For a preparation check, run:

```sh
cargo run -p bevy-wc3 --example compile -- path/to/model.mdx
```

Both examples decode the model directly, and `compile` prepares geometry without
resolving texture images. These checks help diagnose decoding and preparation
problems; use the viewer or captures to verify texture loading and appearance.
