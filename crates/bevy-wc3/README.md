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
as batched head and tail quads. `spawn_prepared_model` now also takes mutable
mesh assets to create those per-instance particle meshes. Bitmap and PRE2 replaceable IDs use the same per-instance bindings.
The renderer does not yet implement Reforged normal/ORM slots, geoset and UV
animation, node billboards, Classic PREM particles,
ribbons, WC3 pass ordering, or skin groups with more than four influences.
