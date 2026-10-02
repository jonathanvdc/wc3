# Classic model particles (PREM)

[Documentation index](../README.md) · [Verification notes](../verification.md)

## Emission and child animation

Model-based (`EmitterUsesMdl`) PREM emitters spawn independent child model
instances. Emission rate and visibility control births. Lifetime, speed, gravity,
latitude, and the animated node transform are sampled at each subframe birth.
Particles retain world-space origins, velocities, model scale, and random Z
headings, with analytic constant-gravity motion. Children animate sequence zero
using particle age; sequence looping follows the child model's sequence flags.
Pausing or changing the parent playback speed also changes particle time.
Immediate sequence changes or backward seeks clear particles and emission phase.
Blended changes retain live model particles; see [animation blending](animation-blending.md).

## Visibility and ownership

Root visibility hides existing particles and stops new births; emitter visibility
only stops births. Despawning the owner removes all particle models. Recursive
child model paths are blocked. Child meshes and bind poses share the existing
prepared-model cache; materials and animation belong to each instance.

## Birth transforms and motion

PREM latitude is interpreted directly in radians; longitude is ignored. Local
emission direction combines a random Z heading with a Y tilt within the latitude
range. World rotation is applied before componentwise world scale to
velocity. Gravity is multiplied by world Z scale, and each child receives a
random world Z heading. Position is evaluated analytically from age, initial
velocity, and constant gravity.

## Resource support and emission limits

- `EmitterUsesTga` image particles are not rendered.
- Emission rate and visibility use the update endpoint. Updates crossing keys or
  sequence boundaries can miss births because emission is not integrated across
  the interval.
- Each emitter is capped at 1,024 live model particles and 1,024 attempted births
  per update. Excess births are discarded, rather than queued after a stall.
- Child models use the [mesh material pipeline](renderer.md#materials-and-textures). Node billboarding uses
  the configured driving camera; see [node flags](node-flags.md).
