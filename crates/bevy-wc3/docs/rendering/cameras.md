# Authored cameras

Authored views can be sampled through `ModelRig` or bound to an
application-owned perspective `Camera3d`. Loading a model never creates or activates
a view. Camera feature IDs come from the prepared model's `cameras()` iterator and
belong to that decoded model identity.

## Bind a view

`CameraBinding::new(model_root, camera_feature)` selects a root and typed
camera feature. Its public `fov_multiplier` scales the authored vertical FOV; use
0.75 explicitly for a portrait-style interpretation. Independently set
`Playback::driving_camera` to choose the view used by billboards and
camera-anchored nodes.

The binding owns camera transform and supported perspective lens fields. Activation,
render target, viewport, exposure, and camera lifetime remain application-owned.
Removing the binding stops updates. Replacing or reloading a model creates new
feature IDs: retarget the binding explicitly. An invalid ID retains the previous
view and produces a deduplicated warning.

## Sampling and scheduling

`ModelRig::sample_camera` returns `SampledCamera` at current
playback clocks. `world_transform` maps its authored eye/target/up through the root,
and `perspective_projection` interprets the lens. The live binding samples current
animation and overrides directly rather than reading last frame's rig. Ordinary
application transform chains are composed before propagation.

Look-at cameras retain authored base positions, animated offsets, roll, and
lens values. Field-of-view axis, role, visibility, and optical parameters remain
independent in the semantic definition. Node cameras follow their carrying node
and may supply an explicit target. Both own the same `PerspectiveProjection`;
sampling converts its angle to vertical field of view using the current aspect ratio. Degenerate up
directions have deterministic orthogonal fallbacks. Nonfinite views, invalid lens
ranges, nonperspective projections, missing roots, and unsupported parents retain
the previous view instead of writing invalid transforms.

`Wc3Systems::AnimateCameras` runs in `PostUpdate`, before Bevy camera projection
updates and node evaluation. Move source roots and camera parents before this set.
Animated rig nodes and other bound cameras in those chains are rejected to avoid
circular or stale dependencies. Parents requiring a sheared camera-local transform
are unsupported; ordinary rigid/uniformly scaled parents are supported.

## Lens and multi-view limits

The application controls camera switching and activation. Visibility and modern
focus/focal-length/f-stop tracks do not enable views or configure depth of field.
Projection follows Bevy's infinite reverse-depth perspective convention; authored
far distance informs culling rather than imposing a finite fragment plane.

One driving camera produces one skeleton pose per model instance. Multiple views,
including shadow views, share that pose. Use independent model instances when
portrait and scene billboards must face different cameras.
