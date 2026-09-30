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

Texture paths are checked beside the MDX first, then at the asset root. BLP
files are decoded by the plugin. Meshes, bind poses, and static materials are
shared; animated materials remain private to each instance. See the
`instances` example for a complete app. The lower-level `prepare_model` and
`spawn_prepared_model` APIs remain available for custom model sources.

Run the viewer with any local model path:

```sh
cargo run -p bevy-wc3 --example viewer -- path/to/model.mdx
```

The viewer searches beside the model for referenced BLP and DDS files and
cycles sequences with Space. The `inspect` and `compile` examples validate
models without opening a GPU window:

```sh
cargo run -p bevy-wc3 --example compile -- path/to/model.mdx
```

The current renderer uses Bevy PBR shading with WC3 layer blend and depth
states. It does not yet implement Reforged normal/ORM slots, geoset and UV
animation, team color, billboards, particles, ribbons, WC3 pass ordering, or
skin groups with more than four influences.
