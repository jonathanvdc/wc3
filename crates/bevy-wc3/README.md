# bevy-wc3

Load and render Warcraft III MDX and MDL models in Bevy 0.19. The crate provides
GPU-skinned meshes, sequence animation and pose blending, Classic and Reforged
materials, attachments, model lights and cameras, event notifications, particles,
and ribbon trails.

## Get started

Add Bevy and the renderer to your application:

```sh
cargo add bevy bevy-wc3
```

Register `Wc3BevyPlugin` alongside Bevy’s default plugins, put the model and its
textures under `assets/`, and spawn a model root:

```rust
use bevy::prelude::*;
use bevy_wc3::{Wc3BevyPlugin, Wc3ModelInstance};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, Wc3BevyPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Wc3ModelInstance::new(asset_server.load("units/footman.mdx")),
        Transform::default(),
    ));
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, -400.0, 200.0)
            .looking_at(Vec3::new(0.0, 0.0, 80.0), Vec3::Z),
    ));
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(100.0, -100.0, 200.0).looking_at(Vec3::ZERO, Vec3::Z),
    ));
}
```

The root is populated when the asset is ready. Literal texture paths resolve
beside the model first, then from the asset root; BLP decoding is included.
Supply replaceable textures, such as team colors, through `Wc3TextureBindings`.
Instances share meshes and bind poses and own their animation and materials.

## Try a model

From a workspace checkout, open a local MDX or MDL file in the viewer:

```sh
cargo run -p bevy-wc3 --example viewer -- path/to/model.mdx
```

Left drag rotates, right drag pans, the scroll wheel zooms, and Space cycles
sequences. The viewer also accepts MDL and texture overrides. Add `--lod auto`
to switch authored geometry while zooming, and tune `--lod-bias` for the desired
quality. Global presets and per-instance overrides are described in the
[geometry LOD guide](docs/rendering/lod.md). For offscreen PNGs,
see the [viewer and capture guide](docs/tools.md), which covers camera setup,
texture overrides, and controlled animation times.

## Documentation

Continue with the application guide to configure instances, or the renderer
and architecture guides to understand and extend the implementation:

- [Application guide](docs/usage.md): loading, textures, animation, attachments,
  lights, geometry quality, custom sources, and instance lifetime.
- [Renderer guide](docs/rendering/renderer.md): geometry, materials,
  animation, and effects, with links to each implementation topic.
- [Architecture](docs/architecture.md): module responsibilities, asset ownership,
  and system scheduling.
- [Documentation index](docs/README.md): all guides and rendering topics.
- API reference: run `cargo doc -p bevy-wc3 --no-deps --open`.

Rendering integrates with Bevy lighting and render phases. Individual renderer
notes describe supported behavior, implementation limits, and the scope of
visual checks; exact Warcraft III appearance has not been established.
