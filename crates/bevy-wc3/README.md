# bevy-wc3

An initial Bevy 0.19 renderer for MDX models. `Wc3Model::decode` strictly
converts supported MDX versions to V1800. `spawn_model` creates a node hierarchy,
GPU skinned geosets, and one material pass per layer. Per-instance animation
samples MDX sequence and global sequence tracks for node transforms, layer
alpha, and texture selection.

For repeated instances, put the MDX and textures under Bevy's `assets/`
directory and let `Wc3BevyPlugin` load and prepare the model:

```rust
let model = asset_server.load("units/footman.mdx");
commands.spawn((Wc3ModelInstance::new(model.clone()), Transform::default()));
commands.spawn((Wc3ModelInstance::new(model), Transform::from_xyz(150.0, 0.0, 0.0)));
```

Literal bitmap paths are checked beside the MDX first, then at the asset root.
BLP files are decoded by the plugin. Replaceable IDs remain unresolved until a
consumer supplies an image handle. Meshes and bind poses are shared; materials
belong to each instance so texture changes remain local.

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

Attachment paths and model-based Classic PREM paths are also resolved beside the
parent model, then at the asset root. Backslashes are normalized. At each location,
a real `.mdl` is preferred, with `.mdx` as its fallback. Empty and missing paths
keep their record slots; PREM image resources are excluded. Read handles through
`Wc3ModelAsset::model_resources()`, `PreparedModel::model_resources()`, or the
`Wc3ModelResources` component on a spawned instance. `attachment(index)` and
`particle(index)` use source record indices, not attachment IDs or node object IDs.
Custom sources can use `prepare_model_with_resources` with texture and model
resolver callbacks; the existing `prepare_model` API remains available.

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

Each instance root owns its rig, geometry, and PRE2 render entities through the
Bevy hierarchy. Hiding the root hides its geometry and effects; despawning it
cleans them up, including its attachment models. `Wc3ModelOwner` also supports
detached child instances: omit `ChildOf` to keep world-space transforms and
visibility independent while retaining cleanup when the owner despawns. Model-based
PREM emitters spawn world-space child models with independent sequence-zero
animation, sampled birth transforms and physical properties, gravity, and lifetime
cleanup. Parent playback pause/speed controls particle time; sequence changes and
backward seeks clear particles. Image-based PREM is unsupported. See
[PREM fidelity and capture instructions](docs/visual-fidelity/prem.md).

Capture the animated attachment fixture with the existing offscreen renderer:

```sh
cargo run -p bevy-wc3 --example capture -- \
  crates/bevy-wc3/tests/fixtures/attachment_capture.mdl /tmp/wc3-attachments \
  --times 0,0.25,0.75,1.25 --fps 60 --size 640x480 \
  --eye 0,-18,12 --target 0,0,0
```

Run the viewer with any local model path:

```sh
cargo run -p bevy-wc3 --example viewer -- path/to/model.mdx
cargo run -p bevy-wc3 --example viewer -- path/to/tree.mdx --replaceable 31=Textures/BlightedTree.blp
```

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

The current renderer uses Bevy PBR shading with WC3 layer blend and depth
states. PRE2 textured particles are simulated per model instance and rendered
as GPU-instanced head and tail quads. The CPU simulates compact particle records;
the vertex shader constructs camera-facing geometry independently for each view.
Heads and tail widths retain XYZ scale sampled at birth and apply it componentwise
in world space after orienting the quad. Tail length follows velocity, which already
contains the emitter scale. ModelSpace moves live centers and tails with the current
node transform while retaining the scale sampled at birth for quad dimensions.
XYQuad heads stay in world XY with a facing angle sampled from the initial XY
velocity; vertical and stationary particles still produce complete quads.
Shaded particles use Bevy scene lighting with a matte, zero-reflectance material;
Unshaded particles use texture and segment color directly. Billboard heads and tails
use the camera-facing normal; XYQuad heads use world +Z. Lighting leaves alpha intact.
`spawn_prepared_model` takes mutable mesh assets to create a static emitter quad.
Bitmap and PRE2 replaceable IDs use the same per-instance bindings.

See the [renderer documentation](docs/README.md) for visual fidelity gaps and
verification work by rendering topic.
