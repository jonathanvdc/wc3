# Lights, cameras, attachments, and effects

Features attach behavior to hierarchy nodes. Use the shared inspection views to
find that behavior across model profiles, and select its complete native definition
when editing specialized properties. This guide builds on the [model graph](semantic.md).

## Inspect and edit behavior

Universal's `NodeFeature` selects behavior directly: lights, cameras, attachments,
particle and ribbon emitters, collision shapes, and geometry animation.
Features are independently identified and anchored to ordinary hierarchy nodes,
with no outer source-format variant to unwrap before discovering their purpose.

`FeatureSchema::feature` returns a borrowed `FeatureView` for inspection across
profiles. Custom features can return `FeatureView::Custom`. Unsupported native
feature families use `Unavailable`, so they cannot be constructed accidentally.

## Lights and cameras

Shared structures retain common authored fields while explicit alternatives
preserve different interpretations. `AmbientDiffuseLight` owns independently
animated diffuse and ambient emission and distance attenuation. `LightActivation`
distinguishes fractional visibility from boolean enablement; `LightShadows` and
optional falloff coefficients are independent of activation. World, region-point,
and single-emission lighting retain their complete definitions as separate illumination
models. Unresolved light-kind selectors retain their original vocabulary, so
changing activation cannot reinterpret an unknown code in another format.

`Camera::Node` owns a `NodeCamera` that follows its hierarchy node's position and orientation, with an
optional target. `Camera::LookAt` owns a `LookAtCamera` with eye displacement,
model-space target and target displacement, and roll. Both camera definitions
own a `PerspectiveProjection` containing field of view, its vertical or diagonal
`FieldOfViewAxis`, and clipping distances. Projection is independent of
application role, visibility, and `CameraLens` optical parameters. A camera role
does not select a projection or imply lens parameters.

Native profiles constrain these shared meanings. MDLX uses a static vertical
field of view, optional visibility and lens parameters, and no application role.
M2 uses an animated diagonal field of view and an application role, without
visibility or lens parameters. Widening a static angle creates a constant value
slot. Exact MDLX narrowing rejects animation of that slot, even if another
property shares it. Native profile validation rejects unsupported combinations
before an exporter can discard them.

Inspection uses the same `CameraView::LookAt` and `LightView::AmbientDiffuse`
alternatives for shared behavior across profiles. Borrowed references preserve
static versus animated angles and boolean versus fractional activation without
materializing or reinterpreting their curves.

## Attachments and mounts

Attachments compose identity, visibility, and placement independently.
`Socket` exposes a named identity, an application role, or an MDLX point
index; those namespaces remain distinct even when their numeric values coincide.
Its `visibility: Property<f32>` describes mounted content visibility; constant
`1.0` represents an uncontrolled socket. Native M2 boolean visibility is read as
scalar zero or one through `PropertyValues::scalar_from_boolean`, preserving the source
slot, shared tracks, and clocks. The socket also retains unresolved authored
controls. Exact M2 narrowing requires the original explicit boolean source;
fractional visibility cannot be represented. Native formats without a visibility
control require constant-one visibility. Exact export checks identity namespace,
visibility representability, and control presence
against the destination vocabulary. `ChildModel` combines a resource with an
optional socket and placement appearance. `Attachment::Socket` exposes a socket
without instantiating a child; `Attachment::Child` declares a child reference. MDLX
attachments and WMO doodads therefore share the child model structure, while
world tint and light selection remain an explicit appearance program. MDLX path strings retain their borrowed or owned
storage as byte paths; exact MDLX narrowing requires UTF-8.

`ModelMount` describes where a child follows the carrying model, independently of
resource selection and playback. `Attachment::mount` derives this description
for an authored child reference. Its `MountTarget` selects a node or a socket,
with socket targets retaining authored visibility. A local transform supplies the
offset. `CoordinatePolicy::Host` uses the host's authored axes and units;
`CoordinatePolicy::Child` retains the child's standalone axes and units.

## Particles and ribbons

Particle emitters retain their native semantic family. MDLX exposes
`mdlx::ParticleEmitter1` for resource particles and `mdlx::ParticleEmitter2` for
atlas sprites and tails; M2 exposes `m2::ParticleEmitter`. Each uses
`ParticleComposition` to group emission, motion, lifetime, appearance, and details,
with component types that preserve its family's authored controls. Model-clock
animation changes emitter parameters, while particle-age programs control the
appearance of each living particle. Native angle conventions, random variation,
unknown controls, and resource modes retain their original meanings.

Universal's `ParticleEmitter` selects `MdlxEmitter1`, `MdlxEmitter2`, or `M2`,
each owning the same complete definition used by its native profile. Its
`Resource` alternative references an externally defined effect. Widening adds a family tag without
allocating property slots or changing borrowed curve storage. Narrowing into the
matching family moves the definition back; another family returns the original
payload as an error. The family tag keeps MDLX staged appearance and M2 age curves
paired with their corresponding simulation rules.

Ribbons follow the same contract. `mdlx::RibbonEmitter` and `m2::RibbonEmitter`
use `TrailComposition` to share section geometry and appearance while retaining
their exact rate, atlas selector, rendering bindings, and behavior.
Universal's `RibbonEmitter` owns either definition in its `Mdlx` or `M2`
alternative, or references an external effect through `Resource`. MDLX rates
remain `u32` and M2 rates remain `f32`; atlas selectors retain their `u32` or
`u16` property identities. MDLX gravity and M2 emission angle stay in their
respective behavior definitions, alongside activation and M2 effect identity.
M2 pass priority, color-table selection, and texture transforms belong to
`M2RibbonRendering` with its material and texture bindings.

The inspection views expose common light, camera, particle, and ribbon fields
without cloning definitions or materializing curves. For specialized edits, select
the complete family after widening:

```rust
use wc3::model::profiles::mdlx;
use wc3::model::{Construct, ParticleEmitter, PropertyValues};

let mut values = PropertyValues::default();
let native = mdlx::ParticleEmitter2::construct(&mut values);
let mut emitter = ParticleEmitter::from(native);
let ParticleEmitter::MdlxEmitter2(definition) = &mut emitter else { unreachable!() };
definition.emission.rate = values.add(24.);
definition.motion.speed = values.add(8.);
let exported = mdlx::ParticleEmitter2::try_from(emitter).unwrap();
assert_eq!(values[exported.emission.rate], 24.);
assert_eq!(values[exported.motion.speed], 8.);
```

Widening moves each feature into its behavior representation. Exact narrowing
checks its current fields and interpretations, preserving resources, animation
bindings, and relationship identities. For example, editing shared diffuse
intensity survives MDLX narrowing; changing boolean enablement into fractional
visibility requires an explicit authoring decision. Unsupported features and
unrepresentable additions return errors instead of being discarded.
