# Using bevy-wc3

Register `Wc3BevyPlugin` alongside Bevy’s `DefaultPlugins`. Each model root
has its own animation, materials, and texture choices; geometry is shared.
This guide covers loading models, configuring their appearance and playback,
resolving child models, and integrating the renderer with application systems.

## Load and spawn models

For repeated instances, put the MDX and textures under Bevy's `assets/`
directory and let `Wc3BevyPlugin` load and prepare the model:

```rust
let model = asset_server.load("units/footman.mdx");
commands.spawn((Wc3ModelInstance::new(model.clone()), Transform::default()));
commands.spawn((Wc3ModelInstance::new(model), Transform::from_xyz(150.0, 0.0, 0.0)));
```

Bitmap paths are checked beside the model first, then at the asset root.
At each location, the literal filename is tried first, followed by the same path
with `.blp`, `.dds`, `.png`, and `.tga` extensions, in that order. This allows
references such as `Textures\Body.tif` to resolve to `Textures/Body.blp`.
Missing files advance to the next candidate; other read failures stop model
loading. Decoder failures do not try another candidate.
File contents must match the selected filename’s format. Explicit texture
overrides remain exact.
BLP files are decoded by the plugin. Replaceable IDs remain unresolved until a
consumer supplies an image handle. Meshes and bind poses are shared; materials
belong to each instance so texture changes remain local.

## Load from named sources and MPQs

The model loader preserves the model's Bevy asset source for bitmap textures,
attachments, and Classic PREM child models, including recursive dependencies.
For example, a model loaded as `warcraft://units/footman.mdx` searches beside
that model and at the root of the `warcraft` source. It does not fall back to
Bevy's default source. Each candidate is looked up through the source before
trying the next location or extension, so candidate order takes precedence over
any mount order inside that source.

Applications register sources before adding `DefaultPlugins`. The separate
[`bevy-mpq` crate](../../bevy-mpq/README.md) provides `MpqAssetReader` and a
caller-ordered `OverlayAssetReader`; `bevy-wc3` itself has no MPQ dependency.
An application can combine loose overrides, a map archive, and base archives
under one source name, or give different maps separate source names to isolate
identical asset paths. Archive order, locales, and parsing limits belong to the
application. Replacement texture handles can use any source explicitly.

Run the archive-backed example with an internal model path followed by archives
in highest-to-lowest priority order:

```sh
cargo run -p bevy-wc3 --example mpq -- \
  units/human/footman/footman.mdx map.w3x base.mpq
```

This example loads complete archive entries; MPQ patch deltas, archive watching,
and directory enumeration are unsupported. See the adapter README for setup
with loose-file overrides and the reader's memory and concurrency behavior.

## Choose textures

A viewer or game can choose textures using the decoded model (`Wc3ModelAsset::source`)
and its own state, including model type, team, and blight. Bind by replaceable ID
or by exact bitmap/PRE2 slot. Slot bindings take precedence:

```rust
let mut textures = Wc3TextureBindings::default();
textures.set_replaceable(31, healthy_tree);
textures.set_slot(Wc3TextureSlot::Bitmap(2), blighted_tree);
commands.spawn((Wc3ModelInstance::new(model), textures));
```

Change `Wc3TextureBindings` on that root entity later to hot swap images. A PRE2
emitter with `ReplaceableId == 0` uses its `TextureID` bitmap slot, including
bitmap overrides. `prepare_model` and `spawn_prepared_model_with_bindings` offer
the same behavior for custom model sources.

## Choose geometry quality

Models initially select authored level zero, resolving to the next coarser level
if zero is absent. Enable automatic LOD globally with a quality preset, then
adjust individual roots when needed:

```rust
app.insert_resource(Wc3LodSettings::medium());
commands.entity(root).insert(Wc3LodOverride(Wc3LodSettings::high()));
commands.entity(other_root).insert(Wc3Lod::Fixed(2));
```

The presets use projected screen size, so camera zoom and viewport resolution
influence selection. Larger quality bias keeps more detailed geometry visible;
`minimum_level` caps detail for weaker machines. Settings also expose descending
pixel thresholds and hysteresis to stabilize transitions. Put
`Wc3Lod::Automatic` on a root to opt in without changing the global default.
Query `Wc3LodState` after loading to inspect available and selected levels.
See [geometry LOD](rendering/lod.md) for override precedence, camera selection,
and limits. Models need authored levels to benefit from switching.

## Prepare custom model sources

For custom model sources, prepare shared geometry once and spawn it repeatedly.
Only spawning needs the material asset store:

```rust
let prepared = prepare_model(
    &mut meshes,
    &mut inverse_bindposes,
    &source,
    |path| Some(asset_server.load(path.to_owned())),
)?;
let root = spawn_prepared_model(
    &mut commands,
    &mut meshes,
    &mut materials,
    &prepared,
);
```

Use `prepare_model_with_resources` to additionally resolve child-model paths, and
`spawn_prepared_model_with_bindings` to supply per-instance texture choices.

## Configure model lights

Model point and directional lights spawn as ordinary Bevy scene lights, illuminating
both WC3 and Bevy materials. Their transforms, colors, intensities, ranges, and
visibility animate; authored ShadowCasting flags enable Bevy shadows. Supply
`Wc3LightSettings` on the instance root to tune power/range conversion or disable
imported lights and shadows:

```rust
commands.spawn((
    Wc3ModelInstance::new(model),
    Wc3LightSettings {
        point_intensity_scale: 2_000.0,
        shadows_enabled: false,
        ..default()
    },
));
```

Query `Wc3Light` to inspect the source record or disable a particular light with
its `enabled` field. Ambient contributions and custom Reforged falloff/shadow
ranges have no native mapping; scene ambient lighting remains application-owned.
See [model lights](rendering/lights.md) for conversion defaults,
customization, and rendering semantics.

## Resolve child models

Attachment paths and model-based Classic PREM paths are also resolved beside the
parent model, then at the asset root. Backslashes are normalized. At each location,
a real `.mdl` is preferred, with `.mdx` as its fallback. Empty and missing paths
keep their record slots; PREM image resources are excluded. Read handles through
`Wc3ModelAsset::model_resources()`, `PreparedModel::model_resources()`, or the
`Wc3ModelResources` component on a spawned instance. `attachment(index)` and
`particle(index)` use source record indices, not attachment IDs or node object IDs.
Custom sources can use `prepare_model_with_resources` with texture and model
resolver callbacks; the existing `prepare_model` API remains available.

## Mount models on attachment points

Resolved attachment paths automatically spawn child model instances. Query
`Wc3Attachments` on the parent root to find points by record index (`get`),
attachment ID (`by_id`), or full name (`by_name`, ignoring ASCII case). Empty or
missing paths still expose points for consumer-supplied models:

```rust
let point = attachments.by_name("Weapon Ref").unwrap();
let child = point.spawn_model(&mut commands, child_model);
commands.entity(child).insert(child_texture_bindings);
```

The returned entity is the child model's transform and animation root. The
point's `node` is the original animated MDX node; its separate `mount` follows
that node and gates attached content with the attachment visibility track.
Mounts inherit translation, rotation, and scale. Visibility uses the parent
sequence/global-sequence clock and is shown above `0.1`, with missing keys
falling back to visible. Attached models loop sequence zero, pause while their
mount is hidden, and restart when shown again, when the parent changes sequence,
or when the parent animation seeks backward. Each child has its own rig,
materials, texture bindings, and animation clock. Nested attachments are updated
from outer to inner models; recursive model paths are blocked during spawning.

## Manage instance lifetime

Each instance root owns its rig, geometry, and PRE2/ribbon render entities through the
Bevy hierarchy. Hiding the root hides its geometry and effects; despawning it
cleans them up, including its attachment models. `Wc3ModelOwner` also supports
detached child instances: omit `ChildOf` to keep world-space transforms and
visibility independent while retaining cleanup when the owner despawns. Model-based
PREM emitters spawn world-space child models with independent sequence-zero
animation, sampled birth transforms and physical properties, gravity, and lifetime
cleanup. Parent playback pause/speed controls particle time; immediate sequence
changes and backward seeks clear particles, while blended changes retain them. Image-based PREM is unsupported. See
[Classic model particles](rendering/prem.md).

## Inspect and capture models

Capture the animated attachment fixture with the existing offscreen renderer:

```sh
cargo run -p bevy-wc3 --example capture -- \
  crates/bevy-wc3/tests/fixtures/attachment_capture.mdl /tmp/wc3-attachments \
  --times 0,0.25,0.75,1.25 --fps 60 --size 640x480 \
  --eye 0,-18,12 --target 0,0,0
```

The capture tool also accepts `--lod LEVEL|auto` and quality overrides; see
[LOD captures](rendering/lod.md#capture-levels).

Run the viewer with any local model path:

```sh
cargo run -p bevy-wc3 --example viewer -- path/to/model.mdx
cargo run -p bevy-wc3 --example viewer -- path/to/tree.mdx --replaceable 31=Textures/BlightedTree.blp
```

Use `--lod auto` to switch authored geometry while zooming. The viewer accepts
the same `--lod-bias`, `--lod-thresholds`, `--lod-hysteresis`, and `--lod-minimum`
quality controls as the capture tool; its default remains fixed level zero.

The viewer uses the MDX's directory as its Bevy asset root. Literal bitmap
paths are resolved beside the MDX first, then from that root. Left drag rotates the
camera, right drag pans, the scroll wheel zooms, and Space cycles sequences.
Use `--replaceable ID=PATH`, `--bitmap INDEX=PATH`, or `--particle2 INDEX=PATH`
to bind viewer textures relative to that asset root.
The `inspect` and `compile` examples validate
models without opening a GPU window:

```sh
cargo run -p bevy-wc3 --example compile -- path/to/model.mdx
```

## Control animation

Query `Wc3Animation` on a spawned root. `sequences()` lists the available
sequences; `play(index)` starts or restarts one using the model’s BlendTime.
Call it when the desired animation changes. Use `play_immediately(index)` for
an immediate switch or `play_with_blend(index, Duration)` for a custom fade.
Set `playing` to pause/resume and `speed` to change playback speed.
`seek(milliseconds)` changes the sampling clock without replaying skipped
particle births or events. See [animation blending](rendering/animation-blending.md).

## Integrate with application systems

Use `Wc3Systems` to order application systems around loading, animation, and
effect simulation. See the [scheduling contract](architecture.md#scheduling-contract).
[Model cameras](rendering/cameras.md) explains opt-in camera playback;
[model events](rendering/events.md) explains reading animation notifications.
