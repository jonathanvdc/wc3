# bevy-wc3

Load Warcraft III MDX/MDL and World of Warcraft M2/WMO models into Bevy 0.19.
The renderer supports animated geometry, layered materials, attachments, lights,
cameras, particles, and ribbons. Models share Bevy meshes and constant materials
while keeping their animation and effects independent.

Native shader and effect coverage varies by model family. Check the
[renderer overview](docs/rendering/renderer.md) for support and limits.
Game assets are not included.

## Try a model

From the workspace root, open a local model in the viewer:

```sh
cargo run -p bevy-wc3 --example viewer -- path/to/model.mdx
```

Drag to orbit, right-drag to pan, scroll to zoom, and press Space to change
sequences. The model's directory is the default texture root. See
[viewer and captures](docs/tools.md) for shared asset roots, texture overrides,
M2/WMO companion resources, and repeatable PNG captures.

## Add it to an application

Add the renderer and its matching Bevy version:

```sh
cargo add bevy@0.19.1 bevy-wc3
```

Register `Wc3BevyPlugin` with Bevy's default plugins and load a model from your
Bevy asset directory:

```rust,no_run
use bevy::prelude::*;
use bevy_wc3::{ModelInstance, Wc3BevyPlugin};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, Wc3BevyPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    commands.spawn((
        ModelInstance(assets.load("units/footman.mdx")),
        Transform::default(),
    ));
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.0, 4.0).looking_at(Vec3::Y, Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(2.0, 4.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
```

Adjust the camera and root scale for your asset. Loading adds `ModelRig` when the
instance is ready. A standalone instance starts in its rest state; choose a clip
from `ModelRig::model().clips()` and call `Playback::play` to animate it.
The [application guide](docs/usage.md) shows how to do this and how to inspect
loading failures.

## Go further

Choose a guide for the task at hand:

- [Application integration](docs/usage.md): resources, playback, replacements, sockets, and static pose baking.
- [Model assembly](docs/rendering/assembly.md): shared skeletons, mounted parts, and composed textures.
- [WoW characters](docs/rendering/characters.md): native appearance and equipment choices.
- [Rendering reference](docs/README.md#rendering-reference): materials, motion, scene features, and effects.
- [Architecture](docs/architecture.md): ownership, compiler stages, and scheduling for contributors.

The renderer also accepts application-authored `Model<Universal>` graphs. See
[custom model sources](docs/usage.md#prepare-custom-model-sources), or build the
API reference with `cargo doc -p bevy-wc3 --no-deps --open`.
