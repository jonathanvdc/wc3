# Viewer and captures

The examples let you inspect a model interactively, validate its preparation,
or render PNGs at repeatable animation times. The viewer and capture tools load
native model assets through `Wc3BevyPlugin`, using the same Universal pipeline
as applications. Run the commands below from the repository root. For application
setup, see [Using bevy-wc3](usage.md).

## Explore a model

Open a local model in the viewer to inspect its geometry and sequences:

```sh
cargo run -p bevy-wc3 --example viewer -- path/to/model.mdx
```

Left drag rotates the camera, right drag pans, the scroll wheel zooms, and Space
cycles sequences. The model's directory is the default asset root. Texture paths
resolve beside the model first, then from that root, using the loader's
[resource provider](usage.md).

Use `--replaceable ID=PATH` or `--bitmap INDEX=PATH` to override textures
relative to the asset root. Bitmap indices enumerate semantic textures, including
effect textures; there is no separate native particle texture namespace. For example:

```sh
cargo run -p bevy-wc3 --example viewer -- path/to/tree.mdx \
  --replaceable 31=Textures/BlightedTree.blp
```

The viewer defaults to fixed geometry level zero. Use `--lod auto` to switch
among authored levels while zooming. `--lod-bias`, `--lod-thresholds`,
`--lod-hysteresis`, and `--lod-minimum` tune the same quality policy used by the
capture tool; see [geometry LOD](rendering/lod.md).

For M2 companions and archive texture identities, use repeated `--file-id ID=PATH`
arguments. WMO loading also requires an explicit `--wmo-profile` matching the
source build, such as `warwithin`; the renderer does not guess the build layout.
The capture tool accepts the same provider choices. Native import is strict by
default. `--allow-loss` explicitly accepts conversion loss for unsupported native
records, with each dropped interpretation reported as a rendering diagnostic.
It does not repair malformed models or supply missing required companions.

## Capture animation frames

The offscreen capture example produces PNGs at increasing simulation times.
This attachment fixture provides a small starting point with an explicit camera:

```sh
cargo run -p bevy-wc3 --example capture -- \
  crates/bevy-wc3/tests/fixtures/attachment_capture.mdl /tmp/wc3-attachments \
  --times 0,0.25,0.75,1.25 --fps 60 --size 640x480 \
  --eye 0,12,18 --target 0,0,0
```

The default is one capture at 1 second, 60 simulation steps per second, and
640x480 pixels. Without camera options, the camera frames rest-pose model bounds with Bevy Y up.
Use `--asset-root PATH` for textures shared across model directories; the same
texture override options as the viewer are available. An authored model camera
can replace the explicit eye/target; see [model cameras](rendering/cameras.md).

Capture times are nonnegative, strictly increasing seconds from the initial
sequence start. Select a sequence with `--sequence INDEX`; indices start at zero.
The tool simulates from time zero in steps of at most `1 / FPS`, splitting steps
to reach requested times exactly. Loading, pipeline warmup, and GPU readback
use zero simulation delta,
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
[static pose baking](usage.md) for the snapshot's scope.

## Capture an equipped character

`--character-recipe PATH` assembles a resolved WoW appearance before entering the
same capture loop. This tool branch accepts modern chunked M2 version 274 and
FileDataID resources under `ASSET_ROOT/filedata/`; ordinary native model loading
retains its existing dialect support. The positional model must match the recipe's
body identity. Add `--physics` for authored secondary motion, or `--bake-pose` for
static composition snapshots; these modes cannot be combined.

A resolved JSON recipe names the body, visible native geosets, texture replacement
categories and model parts. A part with `attachment` uses that native M2 role;
without one it uses checked shared-skeleton matching. Texture values are either
image FileDataIDs or atlas objects with explicit dimensions and ordered layers:

```json
{
  "body": 921844,
  "geosets": [
    0, 401, 501, 802, 902, 1002, 1102, 1301,
    1801, 2002, 2201, 2301, 3202, 3301
  ],
  "textures": {
    "1": {
      "width": 1024,
      "height": 512,
      "layers": [{"image": 997259, "destination": [0, 0, 1024, 512]}]
    },
    "9": {"width": 1, "height": 1, "clear": [0, 0, 0, 0], "layers": []},
    "19": 3622959
  },
  "parts": [
    {"model": 1551025, "slot": "chest", "geosets": [401, 2201], "textures": {"2": 1550866}},
    {"model": 1533273, "slot": "head", "attachment": 11, "textures": {"2": 1550871}}
  ]
}
```

The example uses external asset identities; game assets are not distributed with
the crate. Atlas layers may specify a `source` crop and byte `opacity` as well as
the destination `[x, y, width, height]`. The recipe is a resolved rendering input,
not an item database.

Body selections must retain the palms and fingers needed
beneath equipment cuffs; selecting a glove mesh alone does not restore omitted
body geometry. The identifiers above belong to that specific body, rather than
being defaults for other models. Applications can use `CharacterTables` or their
own
`CharacterDatabase` to produce the equivalent typed definition; see
[characters and equipment](rendering/characters.md).

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

[Animation blending](rendering/animation-blending.md)
shows scheduled sequence switches, and [LOD captures](rendering/lod.md)
shows fixed and automatic geometry selection. Run `capture --help` through Cargo
for the complete set of options.

## Measure completed frames

Pass `--benchmark-frames N` to measure completed frames at each requested capture
pose. The tool warms the pose with eight synchronized frames, then measures `N`
frames while keeping simulation time fixed. It writes a timing CSV beside the
PNG and prints median and p95 wall times.

Each sample includes CPU update, render submission, and waiting for the submitted
GPU work to finish. Loading, screenshot readback, and file writes are excluded.
Use the [benchmark guide](../../../benchmarks/README.md#host-gpu-measurements)
for synthetic asset export and instructions for comparing renderer revisions.

## Validate without rendering

The `inspect` and `compile` examples inspect decoded data and prepared assets
without opening a GPU window. For a preparation check, run:

```sh
cargo run -p bevy-wc3 --example compile -- path/to/model.mdx
```

Both examples directly import MDX or MDL into a Universal graph, and `compile` prepares geometry without
resolving texture images. These checks help diagnose decoding and preparation
problems; use the viewer or captures to verify texture loading and appearance.
