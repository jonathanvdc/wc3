# External-model particles

Model-particle emitters instantiate independent child models at sampled birth poses.
MDLX model-based PREM imports use this path; image-based PREM and external
effect-only resources require other interpreters. Child meshes and bind palettes
share prepared assets while playback and changing materials remain independent.

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
budget. `UniversalModelParticleEmitter` exposes population, omitted births, and
skipped simulation time. Excess work is discarded rather than queued after a stall.
This is a CPU hierarchy/model simulation and can cost substantially more than sprite
particles at comparable populations.
