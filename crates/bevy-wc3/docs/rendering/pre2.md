# Sprite particles

MDLX `mdlx::ParticleEmitter2` definitions use immutable GPU birth records. M2
`m2::ParticleEmitter` definitions use reusable CPU mesh buffers. Universal retains
these complete families in `ParticleEmitter::MdlxEmitter2` and `ParticleEmitter::M2`. Both paths retain semantic
emitter identities, bounded populations, authored motion, and independent instance
clocks; they have different geometry and material capabilities.

## Births and motion

Continuous emission uses bounded 60 Hz midpoint integration. Supported staged bursts follow crossed keys and cycles instead of observing only the
update endpoint. Birth
parameters and hierarchy transforms sample occurrence times, including transitions
and nonlinear parent motion. Application root movement interpolates observed frame
endpoints. This integration approximates continuously changing rates.

World particles retain their birth frame; local particles follow the current
emitter. Gravity and fixed-plane geometry follow authored axes after coordinate
conversion. Sprite lighting uses Bevy's PBR pipeline, so supported native motion
and materials do not imply exact game lighting.

## Geometry and appearance

Staged particles support view-facing heads, velocity tails, fixed-plane heads,
normalized lifetime size/color/alpha stages, and atlas frames. Shaded sprites use
Bevy lighting; unlit sprites retain texture/color directly. Authored blend and depth
states remain separate from lighting. Per-view particle ordering changes draw
indices rather than birth records.

M2 age-driven particles sample independent color, opacity, scale, and cell curves.
Import expands their native always-repeat addressing into each particle image
input, independently of mesh inputs sharing the same image dependency.
Their reusable meshes upload when population, appearance, or view changes; paused,
unchanged frames avoid uploads. Native head and tail style bits become a semantic
head/tail selection during import. That selection can enable either geometry,
both, or neither; it does not change the emission direction. Velocity-oriented
heads use the projected particle velocity, while fixed-plane heads use the
appropriate attached or detached XY frame. Tail extent uses velocity multiplied
by the authored tail time, optionally clamped to particle age. Exact native tail
geometry remains unverified.

## M2 motion and coordinate frames

Plane emitters sample a rectangular region and angular velocity around their
positive Z axis. Sphere emitters sample a radial interval with latitude measured
from the XY plane; their zero-angle direction is positive X. The sphere-up flag
selects positive-Z initial velocity when the source-height aim is inactive. M2 emitter frames include the native quarter
turn around Z relative to the carrying bone, before conversion into the Bevy
scene basis.

The attached flag, native bit `0x10`, keeps particle motion in the current emitter
frame. Detached particles transform their birth position and velocity once.
Gravity and static wind for detached particles pass through the model basis,
excluding the emitter bone's rotation and scale. Attached particles retain those
controls in their local simulation frame and follow the current emitter when
rendered. Scalar gravity and already decoded directional gravity vectors use the
same simulation path; packed native keys are decoded by the import adapter before
animation sampling. Wind timing and dynamic wind callbacks remain unsupported.

Motion evaluates constant acceleration and linear drag analytically. Rendered
size inherits bone scale only when its authored flag requests it; emitter
position and initial velocity still follow the emitter transform. Detached
particles can follow emitter translation through the authored speed/scale ramp.
The ramp interpolates between its two speed knots and clamps at the endpoints;
new births receive only translation after their occurrence. Attached particles
already follow their emitter and receive no extra displacement. Emitter speed
uses the observed update interval, so inherited velocity and follow response
remain approximations of the native simulation.

## M2 texture composition and twinkle

Basic multi-texture emitters combine three independently selected texture inputs.
The primary image retains its atlas cells. The two additional images have their
own UV scales, randomized starting coordinates and authored scrolling velocity
ranges. Their UV motion depends on particle age and wraps independently. Two-color
and three-color combinations preserve their corresponding alpha masks and use
unlit materials. Single-texture particles honor the authored lighting flag. The
native texture ordering word supplies a material priority bias; this does not
establish exact mesh/effect ordering across Bevy render phases.

Particle materials distinguish all eight native blend codes: opaque, alpha test,
alpha blend, unweighted additive, alpha-weighted additive, modulation, double
modulation and premultiplied blending. Opaque and alpha-test particles write depth;
the blended modes retain depth testing without writing depth. These states remain
independent of whether the particle receives Bevy lighting.

Twinkle samples a deterministic per-emitter table with per-particle phases to gate
visibility and vary scale over time. This renderer's random streams and table are
independent implementation choices. Native random-number sequences, twinkle
patterns and stochastic distribution fidelity remain unverified.

Container-level texture-program selectors are retained on each emitter. Nonzero
TXAC selectors require advanced UV/shader interpretation and display the primary
input with diagnostics instead of being treated as a basic multi-texture
programme. The modulation-gain flag at bit 29 also remains unresolved. Texture
rotation programmes, spline and bone generators, the sphere implosion filter,
native keyed squirt emission, collisions, external model particles and child
emitters still require additional interpretation or application integration.

## Budgets and limits

Each sprite emitter is bounded to 8,192 living particles. The CPU age path also
bounds births and long-update integration intervals, reporting omitted births and
skipped time through `SpriteEmitter`. Budgets discard work rather than
creating an unbounded catch-up queue. Root despawn removes simulation and render
state. Hierarchy visibility gates rendering; effect activation controls births.

Game-specific ordering across effect and mesh families is not reproduced by the
shared transport. The staged GPU sprite path does not reproduce native effect fog.
