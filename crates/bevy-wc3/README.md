# bevy-wc3

Render `wc3::model::Model<Universal>` in Bevy 0.19 through Bevy's PBR, lighting,
skinning, and render phases. Native adapters load Warcraft III MDX/MDL, World of
Warcraft M2/WMO, StarCraft II M3, and Diablo model families into the same semantic
compiler. Geometry support is broader than native shader and effect coverage; the
[renderer guide](docs/rendering/renderer.md) describes those limits.

## Get started

Register the plugin and load a model through Bevy's asset server. Native resources
that use external database identities need an application resource resolver.

```rust,no_run
use bevy::prelude::*;
use bevy_wc3::{UniversalModelInstance, Wc3BevyPlugin};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, Wc3BevyPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    commands.spawn((UniversalModelInstance(assets.load("units/footman.mdx")),
        Transform::default()));
    commands.spawn((Camera3d::default(),
        Transform::from_xyz(0.0, 2.0, 4.0).looking_at(Vec3::Y, Vec3::Y)));
    commands.spawn((DirectionalLight::default(),
        Transform::from_xyz(2.0, 4.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y)));
}
```

Model coordinates are converted to Bevy's Y-up frame and declared physical units.
Adjust the camera to the model bounds. `UniversalInstance` appears on the root when
loading finishes. Select a typed clip from `instance.model().clips()` through
`UniversalPlayback::play`; default standalone playback samples the rest state.

Meshes, bind palettes, and constant materials are shared. Rigs, clocks, animated
materials, and effect populations remain independent. Replaceable images and
file-data IDs are resolved through `UniversalResourceResolver`; native shader
fallbacks and missing optional resources remain inspectable diagnostics.

## Documentation and tools

The application guide explains custom resources, playback, instance replacement,
attachments, and offline baking. Focused rendering guides describe implemented
behavior without implying complete game fidelity.

- [Application guide](docs/usage.md)
- [Renderer overview](docs/rendering/renderer.md)
- [Semantic compiler and native resources](docs/rendering/universal.md)
- [Architecture and scheduling](docs/architecture.md)
- [Viewer and offscreen captures](docs/tools.md)
- [Documentation index](docs/README.md)

From this workspace, run `cargo run -p bevy-wc3 --example viewer --
path/to/model.mdx`. Use the shared `capture` example for controlled offscreen
rendering. API reference is available with `cargo doc -p bevy-wc3 --no-deps --open`.
