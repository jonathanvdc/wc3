# Sprite particles

Sprite emitters draw small textured heads and trails with authored motion and
lifetime appearance. MDLX `ParticleEmitter2` uses GPU birth records; M2
`ParticleEmitter` uses compact sampled live records and GPU-generated quads.
Both paths have bounded populations and independent instance clocks, with
different geometry and material coverage. Inspect `SpriteEmitter` for live
populations and omitted-work counters.

## Births and motion

Continuous emission uses bounded 60 Hz midpoint integration. MDLX staged
bursts follow crossed keys and cycles instead of observing only the update
endpoint. Birth parameters and hierarchy transforms sample occurrence times,
including transitions and nonlinear parent motion. Application root movement
interpolates observed frame endpoints. This integration approximates
continuously changing rates.

World particles retain their birth frame; local particles follow the current
emitter. Gravity and fixed-plane geometry follow authored axes after coordinate
conversion. Sprite lighting uses Bevy's PBR pipeline, so supported native motion
and materials do not imply exact game lighting.

### M2 sampled bursts

M2's `Squirt` flag (native bit 15) selects sampled-rate bursts instead of
continuous streaming. Each advancing actor update observes its effective rate
once. A transition from at most 0.02 to above 0.02 requests one burst; positive
rate changes do not repeat it. A newly initialized or reset emitter also has one
startup request for its first advancing update. Activation and hierarchy
visibility gate the effective rate before observation. A pulse entirely between
two actor samples can be missed; this is distinct from MDLX keyed emission.

A burst adds signed random variation to the rate, clamps the result to zero or
above, and truncates it to a whole particle count. It has no duration multiplier or fractional carry. Empty requests
and births omitted by capacity or work limits are consumed without catch-up.
Each accepted birth uses the current endpoint pose and starts at age zero,
through the same motion, appearance and model-child admission as continuous M2
births. Native first-substep ages and discrete stepping differ; camera-distance
and global emission-count scaling are not evaluated. Exact game timing and
random-sequence fidelity remain unverified.

Zero-delta warmup and pause neither observe new edges nor consume startup or
random packets. Seek rebases the destination without replaying skipped keys;
positive authorized time after that destination can still produce a sampled
edge. Reset clears living particles and rearms startup, while a natural animation
loop only bursts if a low rate was observed before a later high rate. Adopted
emitter templates retain their externally driven continuous policy and do not
run autonomous squirts.

## Geometry and appearance

Staged particles support view-facing heads, velocity tails, fixed-plane heads,
normalized lifetime size/color/alpha stages, and atlas frames. Shaded sprites
use Bevy lighting; unlit sprites retain texture/color directly. Authored blend
and depth states remain separate from lighting. Per-view particle ordering
changes draw indices rather than birth records. MDLX staged emitters also retain
per-view ordering while their accepted particles have stationary trajectory
centers and the view is unchanged. Lifetime appearance still updates normally.
Moving particles and attached-emitter transforms invalidate that ordering when
their sorting centers change; births, retirement and reset retain independent
source invalidation.

MDLX `ParticleScaling` values are head half extents and tail half widths; tail
length still follows velocity and the authored tail time. Staged RGB interpolates
in its authored display color space before conversion to linear Bevy lighting
and blending. Alpha interpolates independently. This preserves authored color
without changing the source values or imposing byte quantization. Linear scene
blending can still differ from legacy game output.

M2 age-driven particles sample independent color, opacity, scale, and cell
curves. Import expands their native always-repeat addressing into each particle
image input, independently of mesh inputs sharing the same image dependency.
Motion and age curves sample once per particle into a compact live record shared
by its head, tail, and every camera. The GPU expands corners against each view;
native sort bit 1 selects per-view far-to-near order; otherwise live particles
retain their natural order. Camera changes update enabled ordering without
rebuilding vertex meshes or resampling appearance. Each particle appears once,
with head then tail adjacent and stable ties. Exact client ordering quirks remain
unverified. Paused, unchanged effects retain their GPU records. Unchanged views
also retain their sorted draw-order buffers across paused frames and temporary
culling. Native head and tail style bits become a semantic head/tail selection
during import. That selection can enable either geometry, both, or neither; it
does not change the emission direction. Selecting neither keeps births, motion,
retirement, and statistics active without sampling or uploading live geometry.
Velocity-oriented heads use the projected particle velocity, while fixed-plane
heads use the appropriate attached or detached XY frame. M2 lifetime scale values
are head half extents and tail cross-section half widths: a size of `(1, 2)`
produces a camera-facing head spanning `2 × 4` before scene and size controls.
Tail length uses velocity multiplied by the authored tail time, optionally
clamped to particle age; changing width does not change that temporal extent.
Anisotropic tail projection, nearly view-aligned tails and exact native tail
geometry remain unverified. Model-particle child scale is an affine transform
and does not use the sprite full-span factor.

## M2 motion and coordinate frames

Plane emitters sample a rectangular region and angular velocity around their
positive Z axis. Sphere emitters sample a radial interval with latitude measured
from the XY plane; their zero-angle direction is positive X. The sphere-up flag
selects positive-Z initial velocity when the source-height aim is inactive.
Spline emitters sample the authored path between normalized area-control
endpoints, using segment index rather than physical path length. Their initial
velocity follows the path tangent unless source-height aiming overrides it;
exact native spline framing remains unverified. M2 emitter frames include the
native quarter turn around Z relative to the carrying bone, before conversion
into the Bevy scene basis.

Body emitters choose among the first LOD's declared section bone palettes,
excluding joints marked `NoBodySpawn` and deduplicating shared members. An
explicitly empty palette emits nothing, even when skinning weights name other
joints. Models without a declaration approximate eligibility from positive
influences in placed, drawn geometry and report that choice. Each birth samples
the selected joint and its parent at the occurrence time, interpolates their
posed model-space span, and scatters in a plane containing that span. The parent
can remain a span endpoint even when excluded from candidate selection. Joint
positions include their authored bind pivots; an unanimated limb therefore
keeps its model-space placement. The result converts into the emitter frame
once before normal attached or detached simulation. Initial velocity and
source-height aiming use emitter-local axes. Native Bone/Joint distinctions,
random distributions, and client frame fidelity remain unverified.

The attached flag, native bit `0x10`, keeps particle motion in the current
emitter frame. Detached particles transform their birth position and velocity
once. Gravity and static wind for detached particles pass through the model
basis, excluding the emitter bone's rotation and scale. Attached particles
retain those controls in their local simulation frame and follow the current
emitter when rendered. Scalar gravity and already decoded directional gravity
vectors use the same simulation path; packed native keys are decoded by the
import adapter before animation sampling. Wind timing and dynamic wind callbacks
remain unsupported.

Motion evaluates constant acceleration and linear drag analytically. Rendered
size inherits bone scale only when its authored flag requests it; emitter
position and initial velocity still follow the emitter transform. Detached
particles follow emitter translation only when `FollowPosition` (native bit 14)
is enabled. The two speed/response pairs define a line; the solved response
clamps to 0..1 while speed can extrapolate beyond either knot. Equal or nearly
equal speed knots disable follow. Speed uses the model's declared native-unit
conversion; unspecified units retain a numerical scale of one. New births
receive only translation after their occurrence. Attached particles retain their
emitter attachment without additional follow carry. Emitter speed uses the
observed outer update interval. The analytical carry does not reproduce native
live-step age gates or additional attached-particle follow; those behaviors
remain partial.

Sphere emitters can retire imploding particles when their attempted movement
points outward relative to the live emitter center. The filter checks analytic
motion in bounded 60 Hz intervals, starting at each particle's birth occurrence.
Attached particles use the local origin; detached particles use the emitter's
interpolated world center. Follow translation contributes to the tested
movement. This approximates the native discrete integrator rather than
reproducing its acceleration and damping order exactly.

## M2 texture composition and twinkle

Basic multi-texture emitters combine three independently selected texture
inputs. The primary image retains its atlas cells. The two additional images
have their own UV scales, randomized starting coordinates and authored scrolling
velocity ranges. Their UV motion depends on particle age and wraps
independently. Two-color and three-color combinations preserve their
corresponding alpha masks and use unlit materials. Two-color RGB uses a
twofold gain; three-color RGB uses a fourfold gain. Alpha combines all three
texture masks with a twofold gain, or a fourfold gain when native bit 29 is set.
For multi-texture cutout particles, the texture alpha test precedes vertex
opacity, so fading does not prematurely discard otherwise accepted texels.
Single-texture particles honor the authored lighting flag. The native texture ordering word supplies a
material priority bias; this does not establish exact mesh/effect ordering
across Bevy render phases.

Particle materials distinguish all eight native blend codes: opaque, alpha test,
alpha blend, unweighted additive, alpha-weighted additive, modulation, double
modulation and premultiplied blending. Opaque and alpha-test particles write
depth; the blended modes retain depth testing without writing depth. These
states remain independent of whether the particle receives Bevy lighting.

Twinkle samples a deterministic per-emitter table with per-particle phases to
gate visibility and vary scale over time. This renderer's random streams and
table are independent implementation choices. Native random-number sequences,
twinkle patterns and stochastic distribution fidelity remain unverified.

Container-level TXAC selectors are retained on each emitter without invented
byte semantics. Nonzero selectors produce a diagnostic independently of known
composition flags; they do not disable the supported multi-texture path. The
refraction flag selects a separate distortion mask only when multi-texture is
inactive. The primary image is sampled as numeric data, preserving explicit
semantic transfer overrides. Atlas UVs and two independently scrolling UV pairs
sample this same image; particle red controls height and alpha controls mask
contribution. Mask blend factors retain their own native interpretation rather
than using the ordinary sprite color pass.

The mask shares the view's depth and sample count, resolves before composition,
and distorts the completed transparent scene before tonemapping. Targets exist
only for views with visible refraction particles. Each view retains its own mask,
settings and scene bindings. Unsupported mask sample counts produce a warning
and omit distortion. Native game fidelity of displacement and overlapping masks
remains unverified.

Add `ParticleRefraction` to a camera to control displacement strength and view-space
fade distance. Defaults are strength one and 100 scene units, independent of
model and bone scale. Applications converting native yards to metres can set
91.44 explicitly; native import leaves the numerical model units unchanged.
Strength zero skips mask allocation and composition.
Applications supply any underwater-camera policy through these controls.

Texture rotation programmes and collisions still require additional
interpretation or application integration. M2 geometry-model particles use the
separate [model-particle path](prem.md).

## M2 adopted emitter templates

An M2 emitter-template resource (`RPID`, or a legacy child-emitter filename)
supplies pooled sprite emitters. Resolve it with
`ChildModelKey::particle_emitters`; it does not create a model actor per parent
particle. The parent's own sprite or model output remains active. Each source
emitter considers the first four authored M2 candidate slots, retaining rejected
slots rather than promoting later candidates. Nested template resources,
geometry-model outputs, Bone generators and attached templates are diagnosed
and omitted. Other geometry and scene features in the template model are not
instantiated.

The template's emission controls freeze at its first clip and time zero. Its
own activation does not gate sourcing, and it never emits autonomously. Lifetime
appearance curves still sample each child's age. Images retain the template
model's scope. Each source emitter owns one carry, random stream and child pool
per accepted template, shared by all its living parent particles. Each surviving
parent drives that shared pool at its current point, using the source emitter's
axes; unrelated parents are never joined by a birth interpolation segment.
The template's inherit-velocity flag controls inherited parent velocity. Parent
particle color, opacity and size do not override the child's appearance.

Children age once per simulation interval and can survive the parent that
emitted them. Sources born and retired within a long update can still leave
living children from their earlier surviving intervals. Reset clears every pool.
A newly created model-particle owner captures actual template readiness at its
accepted birth frame. Available templates share that initial interval; pending
templates cannot later recover it. Late resource readiness and replacement start
at the current observed time without replaying old drives; paused readiness does
not create historical births.
The common analytical motion and age policy remains an approximation of native
discrete stepping. Attached-parent framing, advanced template texture routing,
random distributions and exact game fidelity remain unverified.

Query `AdoptedParticleTemplate` alongside `SpriteEmitter` for each pool's live
counts and provenance. `SpriteEmitter.feature` and `.node` stay local to the
owning parent model; the provenance component's IDs belong to its template
`ModelAsset`. The parent emitter carries `AdoptedParticleSource` with typed
readiness, accepted-slot count and template-scoped diagnostics. These resource
issues update when readiness or the compiled template generation changes.

## Budgets and limits

Each MDLX sprite emitter is bounded to 8,192 head/tail records; enabling both
uses two records per particle. Each M2 age-driven emitter is bounded to 8,192
particles, sharing one record between its head and tail. The age-driven
simulation also bounds births and long-update integration intervals, reporting
omitted births and skipped time through `SpriteEmitter`. Burst requests share
these limits. Nonfinite or unrepresentable burst counts consume the request and
report skipped emission time, without inventing an omitted-birth count. Budgets
discard work rather than creating an unbounded catch-up queue. Root despawn
removes simulation and render state. Hierarchy visibility gates rendering; effect
activation controls births. Implosion filtering also limits the combined
population to 32,768 particle-step checks per update. It examines a common
chronological prefix of the emission integration window; overlapping omitted
tails are reported once per emitter rather than once per particle. Clocks still
advance to the requested time, and omitted retirement checks are discarded
without a catch-up queue.

Adopted pools retain at most 4,096 children each. Per parent-emitter update,
they share 8,192 child construction attempts, 32,768 source examinations and
32,768 child-motion checks. They also consume the owning model tree's shared
emission-step allowance when present. Capacity omissions and skipped intervals
appear in each pool's `SpriteEmitter`; overlapping omitted source and motion
tails count once per pool. These limits constrain source driving independently
of the normal parent output and never queue catch-up work.

Game-specific ordering across effect and mesh families is not reproduced by the
shared transport. GPU sprites receive Bevy lighting and shadows when lit, but do
not cast shadows or participate in depth, normal, or motion-vector prepasses.
Opaque and cutout M2 sprites still write depth in the main view. Sprite effects
use Bevy scene fog and output processing, including unlit sprites. Additive and
modulation blends fade toward their neutral fog colors; authored unfogged
controls bypass scene fog. M2 unfogged is native bit 3; bit 20 selects
refraction independently. Classic-client game fidelity remains unverified.
