# Sprite particles

Staged atlas particles, including MDLX PRE2, use immutable GPU birth records. M2
normalized-age particles use reusable CPU mesh buffers. Both paths retain semantic
emitter identities, bounded populations, authored motion, and independent instance
clocks; they have different geometry and material capabilities.

## Births and motion

Continuous emission uses bounded 60 Hz midpoint integration. Keyed bursts follow
crossed keys and cycles instead of observing only the update endpoint. Birth
parameters and hierarchy transforms sample occurrence times, including transitions
and nonlinear parent motion. Application root movement interpolates observed frame
endpoints. This integration approximates continuously changing rates.

World particles retain their birth frame; local particles follow the current
emitter. Gravity and fixed-plane geometry follow authored model axes after frame
conversion. M2 motion additionally evaluates supported wind and linear drag
analytically. World-space M2 particles apply the authored emitter-follow speed/scale
ramp to emitter translation, interpolating between the two speed knots and clamping
at the endpoints. New births receive only translation after their occurrence; local
particles already follow the emitter transform and receive no extra displacement.
Emitter speed uses the observed update interval; native client fidelity for follow
response remains unverified. Packed gravity and spline/bone generators need dedicated
interpretation and produce diagnostics instead of a guessed scalar substitute.

## Geometry and appearance

Staged particles support view-facing heads, velocity tails, fixed-plane heads,
normalized lifetime size/color/alpha stages, and atlas frames. Shaded sprites use
Bevy lighting; unlit sprites retain texture/color directly. Authored blend and depth
states remain separate from lighting. Per-view particle ordering changes draw
indices rather than birth records.

M2 age-driven particles sample independent color, opacity, scale, and cell curves.
Reusable meshes upload when population, appearance, or view changes; paused,
unchanged frames avoid uploads. Native tail time-versus-distance interpretation
remains unverified. Secondary texture/composition, texture scroll/rotation,
twinkle controls, external child effects, and several native generators
remain incomplete and produce diagnostics.

## Budgets and limits

Each sprite emitter is bounded to 8,192 living particles. The CPU age path also
bounds births and long-update integration intervals, reporting omitted births and
skipped time through `SpriteEmitter`. Budgets discard work rather than
creating an unbounded catch-up queue. Root despawn removes simulation and render
state. Hierarchy visibility gates rendering; effect activation controls births.

Fog and game-specific ordering across effect and mesh families are not reproduced by
the shared effect transport. M3 programmed particles use the same bounded CPU
transport with a separate interpreter, described below.

## M3 standard billboard subset

Standard billboards support point, plane, box, and sphere distributions with
constant, Z-axis, radial, or random velocity selection. Continuous emission and
independent squirt-count keys sample authored occurrence poses. Squirt keys follow
their selected clip, timeline, or shared clock, including repeating clock crossings;
seek jumps skip crossed births. A nonanimated count emits at the clock origin.
Occurrence scratch is reused and bounded to 4,096 keys per update, and squirt births
share the continuous emitter's population and birth budgets. Optional speed/lifespan
ranges randomize at birth; color,
alpha, size, and rotation freeze their start/middle/end controls at birth, with
independent normalized midpoints. Linear and smooth interpolation and two-phase
flipbooks are implemented. The primary compiled PBR material pass animates on the
model clocks; supplementary passes are not instantiated.

Authored maximum population tightens the shared 8,192 cap. Unsupported geometry,
shape cutouts, encoded XY gravity, and Bezier/hold or unknown interpolation
selectors reject that particle interpretation with diagnostics. Forces, noise,
collision, secondary effects, specialized time controls, and kill-radius
rules remain incomplete and are diagnosed. Endpoint-anchored programmed beams use
a separate partial [ribbon interpreter](ribbons.md#m3-endpoint-beams). This subset's
exact native motion and lighting fidelity is unverified.
