# Model cameras

[Documentation index](../README.md) · [Verification notes](../verification.md)

## Integration

Spawned animation roots expose `Wc3ModelCameras`. Its `definitions()` accessor
returns authored records in source order, including names and unmapped tracks.
`sample(index, animation)` evaluates a model-local `Wc3CameraSample` using the
instance's sequence and global-sequence clocks. Loading a model does not spawn
or activate views.

Bind an application-owned perspective camera explicitly:

```rust
use bevy::prelude::*;
use bevy_wc3::{Wc3CameraBinding, Wc3NodeCamera};

let view = commands.spawn((
    Camera3d::default(),
    Wc3CameraBinding::new(model_root, 0),
)).id();
commands.entity(model_root).insert(Wc3NodeCamera(view));
```

`Wc3CameraBinding` drives the view; `Wc3NodeCamera` separately selects it for
billboards and camera-anchored nodes. The binding owns `Transform`, perspective
FOV, near/far distances, and the standard near clip plane. The application owns
activation, render target, viewport, aspect ratio, exposure, and other render
settings. Removing the binding stops playback. The plugin does not despawn views
when the source root is removed; application hierarchy/ownership relationships
still control camera lifetime. An unparented view retains its previous pose.

`new(root, index)` uses authored vertical FOV radians directly. `portrait(root,
index)` uses a 0.75 multiplier. The public `fov_multiplier` can be changed at
runtime.

## Evaluation and scheduling

Eye and target tracks are additive offsets from their authored base positions;
missing samples default to zero. Both points and the Z-up direction pass through
the current model-root transform. Scalar rotation applies a right-handed roll
about the world eye-to-target direction. When up is parallel to the view,
transformed Y, then X, supplies a deterministic fallback. Root scale affects
positions and the initial up direction, but not clip distances. Camera world
scale is one.

`Wc3Systems::AnimateCameras` runs in `PostUpdate`, before Bevy camera projection
updates, `EvaluateNodePoses`, and transform propagation. Current local hierarchy
transforms are used, avoiding stale `GlobalTransform` values. Application systems
moving source roots or camera parents in `PostUpdate` should run before this set.

The source-root and camera-parent hierarchies must contain ordinary transforms;
animated WC3 nodes and other bound cameras are rejected to avoid stale poses and
circular camera dependencies. Rigid and uniform-scale camera parents are
supported. Parent transforms requiring shear in the camera's local transform are
rejected; an unparented camera avoids this limitation.

Invalid indices, missing roots, nonperspective projections, invalid lens values,
and nonfinite/degenerate views retain the previous transform and projection and
warn once per camera/failure reason, recovering when the inputs become valid.
FOV must be finite and in (0, pi); near must be positive and far greater than near.

## Projection and application controls

Visibility tracks do not change camera activation or select another view.
Modern focus distance, focal length, and f-stop tracks remain available in source
records but are not mapped to depth of field. Camera switching/blending is left
to applications. Camera-dependent nodes still have one driving view per model
instance, rather than a separate skeleton pose for each rendering view.

Projection uses Bevy's standard infinite reverse-depth perspective matrix. The
authored far distance feeds Bevy's frustum/culling behavior; it is not a finite
fragment clipping plane, particularly for geometry exempt from frustum culling.

## Offscreen captures

The shared capture example accepts `--model-camera INDEX` and
`--camera-fov-multiplier NUMBER`. Selecting a model camera excludes `--eye` and
`--target`. Invalid indices or lens values fail before GPU initialization;
invalid sampled views fail before writing a capture.
