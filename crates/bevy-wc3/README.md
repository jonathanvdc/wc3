# bevy-wc3

An initial Bevy 0.19 renderer for MDX models. `Wc3Model::decode` strictly
converts supported MDX versions to V1800. `spawn_model` creates a node hierarchy,
GPU skinned geosets, and one material pass per layer. Per-instance animation
samples MDX sequence and global sequence tracks for node transforms, layer
alpha, and texture selection.

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
skin groups with more than four influences. `spawn_model` creates fresh mesh and
material assets for each instance; shared compiled assets remain to be added.
