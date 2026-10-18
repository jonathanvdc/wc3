# External-model particles

MDLX model-based PREM emitters spawn independent animated child models at sampled
birth poses. Their geometry and bind palettes share prepared assets, while clocks
and changing materials remain independent. Universal stores this family as
`ParticleEmitter::MdlxEmitter1`.

Image-based PREM and external effect-only resources need other interpreters.
The resource variant distinguishes a model from an image; an absent resource
retains the authored mode without declaring a dependency.

## Emission and motion

Continuous rates integrate in bounded 60 Hz intervals. The shared occurrence sampler
evaluates activation, parameters, and hierarchy at subframe births. Children retain
world-space origins, velocities, sampled scale, and random headings. Analytic
constant-gravity motion advances by effect age, with gravity following the authored
model-up axis after coordinate conversion.

The current MDLX PREM interpretation retains its established latitude direction
behavior; native latitude/longitude fidelity remains unverified. Unsupported
controls produce diagnostics rather than implying an exact directional model.
Application root motion interpolates observed update endpoints.

## Playback and ownership

Children start their first clip and sample it by particle age, following the child's
authored looping/clamping policy. Parent pause and playback speed control effect
age. Immediate programme changes and backwards traversal clear particles; blended
changes can retain live particles. Each child retains its own animation root and
authored-coordinate placement policy.

Root hierarchy visibility hides living particles and suppresses new births. Emitter
activation stops births without destroying existing particles. Owner despawn removes
detached world-space children, and cyclic model dependencies are rejected. Native
resources must resolve through the application provider.

## Limits

Each emitter is limited to 1,024 live child models and a bounded per-update birth
budget. `ModelEmitter` exposes population, omitted births, and
skipped simulation time. Excess work is discarded rather than queued after a stall.
This is a CPU hierarchy/model simulation and can cost substantially more than sprite
particles at comparable populations.
