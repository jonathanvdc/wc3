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
relative to the asset root. Bitmap indices enumerate the model's image dependency
table, shared by material and effect inputs; there is no separate native particle
texture namespace. For example:

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
can replace the explicit eye/target with `--model-camera INDEX`; see
[model cameras](rendering/cameras.md).

Capture times are nonnegative, strictly increasing seconds from the initial
sequence start. Select a sequence with `--sequence INDEX`; indices start at zero.
The tool simulates from time zero in steps of at most `1 / FPS`, splitting steps
to reach requested times exactly. Loading, pipeline warmup, and GPU readback
use zero simulation delta,
so wall-clock delays do not advance animation. Seeking directly to a time skips
particle births and events and produces a different effect population.

Schedule a sequence change with `--play SECONDS=INDEX`, for example
`--sequence 0 --play 0.5=1 --blend-ms 250 --times 0.5,0.625,0.75`.
Repeat `--play` in increasing time order for several changes. Omit `--blend-ms`
to use authored transition timing or set it to zero for an immediate switch.

Images are named `frame-0000-0.000s.png`, and so on; reruns overwrite matching
files. The tool requires a GPU even though it does not open a window. Loading,
pipeline, or readback failures terminate with an error. Add `--particle-stats`
to report live sprite populations, dropped births, and skipped simulation time,
alongside model-particle populations and omissions, ribbon sections, connected
segments, and omitted sections.
External programmes report living controllers and particles, extracted billboard
sprites and ribbon segments, geometry omissions, simulation omissions and checked
execution errors; an execution error terminates the capture.
These counters help distinguish simulation and visibility problems; inspect the
images to assess appearance.

Model-particle trees also report accepted actors, pending readiness and exclusive
actor-limit or work-limit omissions. Use `--model-particle-limits 8,2,16` to
set the actor ceiling, admission work and examined emission work per application
frame on the captured model root. Zero disables the corresponding allowance.
These limits include nested particle-owned models, rigid parts and automatic
attachments; see [model-particle limits](rendering/prem.md#limits).

For M2 guild materials, `--guild-palette 0.2,0.4,0.8:1,0.8,0.1:0.1,0.2,0.3`
sets background, emblem, and border multipliers in linear RGB. All three triples
are required and must be finite; values above one are accepted. These application
colours apply to both live and baked material assets before rendering or timing.
Other materials retain their appearance. Applications can update the same palette
through `ModelMaterialExtension::set_guild_palette` on their Bevy material assets.

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
body identity. For example:

```sh
cargo run -p bevy-wc3 --example capture -- \
  /path/to/assets/filedata/921844.m2 /tmp/wc3-equipped \
  --asset-root /path/to/assets --character-recipe /path/to/recipe.json \
  --sequence 0 --times 0.25,1,2
```

Add `--physics` for authored secondary motion, or `--bake-pose` for
static composition snapshots; these modes cannot be combined.

A resolved JSON recipe names the body, visible native geosets, texture replacement
categories and model parts. The optional part `slot` is an arbitrary application
label, such as `chest` or `antenna`; it does not select an inventory slot or affect
rendering. A part with `attachment` uses that native M2 role;
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
not an item database. Its native geoset and replacement-category choices belong to
the WoW adapter. Applications composing arbitrary models use the format-independent
`assembly::Assembly` plan, whose texture atlases and part bindings do not require
WoW inventory or native geoset concepts.

Body selections must retain the palms and fingers needed
beneath equipment cuffs; selecting a glove mesh alone does not restore omitted
body geometry. The identifiers above belong to that specific body, rather than
being defaults for other models. Applications can use `CharacterTables` or their
own `CharacterDatabase` to produce the equivalent typed definition; see
[characters and equipment](rendering/characters.md).

## Compare rendering behavior

Use identical simulation FPS, times, and camera settings when comparing model
variants or renderer revisions. Inspect placement, motion, color and alpha,
atlas frames, geometry, and blending in the resulting images. Compilation or a
nonempty image alone does not establish rendering fidelity; controlled fixtures
also do not establish equivalence to Warcraft III.

`--msaa 1`, `2`, `4` or `8` selects the samples per pixel; the default is four.
Keep the sample count fixed alongside the camera and viewport.
`--hdr` enables an HDR camera before the selected tonemapping operation; choosing
tonemapping alone does not enable HDR. `--viewport X,Y,W,H` restricts the camera
to a positive physical rectangle inside the output image.
`--main-pass-size WIDTHxHEIGHT` overrides the main-pass render resolution while
retaining the output dimensions, for checking dynamic-resolution composition.
`--refraction-strength` and `--refraction-fade-distance` override camera distortion
settings; strength zero provides an identity comparison for authored refraction.

Add `--no-bindless` to compare the material fallback on the same GPU. It disables
texture binding arrays when creating the device, affecting every material in the
capture. The tool reports whether model materials use bindless bindings or the
fallback; keep this setting fixed when comparing performance.

Use `--no-batching` to disable automatic batching on authored model draws when
isolating a batching or material-binding difference. Keep this diagnostic setting
fixed across paired captures; it does not change the default application pipeline.

Use `--fog START,END --fog-color R,G,B` to check linear Bevy distance fog in
world units with a linear RGB color. Keep fog distances, tonemapping, and lighting
identical across comparisons. Unlit sprites and trails still receive camera
output processing; additive and modulation effects fade toward their blend’s
neutral contribution.

For focused comparisons, use a small complete MDL that isolates the behavior
under investigation, with consistent node IDs, pivots, geometry references,
sequence intervals, materials, and textures. Start from a fixture under
`crates/bevy-wc3/tests/fixtures/` or `crates/wc3/tests/fixtures/mdl/`, and keep the
camera and unrelated scene inputs fixed. Model bounds must encompass the content,
or the capture must use an explicit camera.

The [animation guide](rendering/animation-blending.md) explains transition sampling,
and [LOD](rendering/lod.md) explains fixed and automatic geometry policies.
For the complete command options, run:

```sh
cargo run -p bevy-wc3 --example capture -- --help
```

## Measure performance

The [performance guide](performance.md) covers synchronized completed-frame timing
and pipelined character crowds, including report interpretation and measurement
limits. Use optimized builds and hold scene and quality settings fixed.

For a small-scene scheduling experiment, add `--single-threaded` to `capture` or
`crowd`. It runs main and render schedules with serial executors while retaining
the selected frame completion policy and task-pool configuration. Crowd reports
record this choice as `single_threaded_schedules`. Compare otherwise identical
runs and check their images and populations before interpreting the timing;
this configuration does not imply a renderer speedup for every workload.

## Validate without rendering

The `inspect` and `compile` examples inspect decoded data and prepared assets
without opening a GPU window. For a preparation check, run:

```sh
cargo run -p bevy-wc3 --example compile -- path/to/model.mdx
```

Both examples directly import MDX or MDL into a Universal graph, and `compile` prepares geometry without
resolving texture images. These checks help diagnose decoding and preparation
problems; use the viewer or captures to verify texture loading and appearance.
