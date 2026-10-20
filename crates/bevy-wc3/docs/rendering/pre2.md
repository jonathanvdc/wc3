# Sprite particles

Sprite emitters draw small textured heads and trails with authored motion and
lifetime appearance. MDLX `ParticleEmitter2` uses GPU birth records; M2
`ParticleEmitter` uses exactly sampled live records and GPU-generated quads.
Both paths have bounded populations and independent instance clocks, with
different geometry and material coverage. Inspect `SpriteEmitter` for live
populations and omitted-work counters.

## Births and motion

Continuous emission uses bounded 60 Hz midpoint integration. Supported staged
bursts follow crossed keys and cycles instead of observing only the update
endpoint. Birth parameters and hierarchy transforms sample occurrence times,
including transitions and nonlinear parent motion. Application root movement
interpolates observed frame endpoints. This integration approximates
continuously changing rates.

World particles retain their birth frame; local particles follow the current
emitter. Gravity and fixed-plane geometry follow authored axes after coordinate
conversion. Sprite lighting uses Bevy's PBR pipeline, so supported native motion
and materials do not imply exact game lighting.

## Geometry and appearance

Staged particles support view-facing heads, velocity tails, fixed-plane heads,
normalized lifetime size/color/alpha stages, and atlas frames. Shaded sprites
use Bevy lighting; unlit sprites retain texture/color directly. Authored blend
and depth states remain separate from lighting. Per-view particle ordering
changes draw indices rather than birth records.

M2 age-driven particles sample independent color, opacity, scale, and cell
curves. Import expands their native always-repeat addressing into each particle
image input, independently of mesh inputs sharing the same image dependency.
Motion and age curves sample once per particle into a compact live record shared
by its head, tail, and every camera. The GPU expands corners against each view;
camera changes update ordering without rebuilding vertex meshes or resampling
appearance. Paused, unchanged effects retain their GPU records. Unchanged views
also retain their sorted draw-order buffers across paused frames and temporary
culling. Native head and tail style bits become a semantic head/tail selection
during import. That selection can enable either geometry, both, or neither; it
does not change the emission direction. Selecting neither keeps births, motion,
retirement, and statistics active without sampling or uploading live geometry.
Velocity-oriented heads use the projected particle velocity, while fixed-plane
heads use the appropriate attached or detached XY frame. Tail
extent uses velocity multiplied by the authored tail time, optionally clamped to
particle age. Exact native tail geometry remains unverified.

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
particles can follow emitter translation through the authored speed/scale ramp.
The ramp interpolates between its two speed knots and clamps at the endpoints;
new births receive only translation after their occurrence. Attached particles
already follow their emitter and receive no extra displacement. Emitter speed
uses the observed update interval, so inherited velocity and follow response
remain approximations of the native simulation.

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
refraction flag selects a scene-color postprocess only when multi-texture is
inactive. This postprocess has no interpreter in the current Bevy pipeline;
those emitters retain their ordinary primary texture with a diagnostic. Texture
rotation programmes, bone generators, native keyed squirt emission, collisions,
external model particles and child emitters still require additional
interpretation or application integration.

## Budgets and limits

Each MDLX sprite emitter is bounded to 8,192 head/tail records; enabling both
uses two records per particle. Each M2 age-driven emitter is bounded to 8,192
particles, sharing one record between its head and tail. The age-driven
simulation also bounds births and long-update integration intervals, reporting
omitted births and skipped time through `SpriteEmitter`. Budgets discard work
rather than creating an unbounded catch-up queue. Root despawn removes
simulation and render state. Hierarchy visibility gates rendering; effect
activation controls births. Implosion filtering also limits the combined
population to 32,768 particle-step checks per update. It examines a common
chronological prefix of the emission integration window; overlapping omitted
tails are reported once per emitter rather than once per particle. Clocks still
advance to the requested time, and omitted retirement checks are discarded
without a catch-up queue.

Game-specific ordering across effect and mesh families is not reproduced by the
shared transport. GPU sprites receive Bevy lighting and shadows when lit, but do
not cast shadows or participate in depth, normal, or motion-vector prepasses.
Opaque and cutout M2 sprites still write depth in the main view. Sprite effects
use Bevy scene fog and output processing, including unlit sprites. Additive and
modulation blends fade toward their neutral fog colors; authored unfogged
controls bypass scene fog. M2 unfogged is native bit 3; bit 20 selects
refraction independently. Classic-client game fidelity remains unverified.
