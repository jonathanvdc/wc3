# Classic PREM particles

[Documentation index](../README.md)

Model-based (`EmitterUsesMdl`) PREM emitters spawn independent child model
instances. Emission rate and visibility control births. Lifetime, speed, gravity,
latitude, and the animated node transform are sampled at each subframe birth.
Particles retain world-space origins, velocities, model scale, and random Z
headings, with analytic constant-gravity motion. Children animate sequence zero
using particle age; sequence looping follows the child model's sequence flags.
Pausing or changing the parent playback speed also changes particle time.
Changing sequence or seeking backward clears particles and emission phase.

Root visibility hides existing particles and stops new births; emitter visibility
only stops births. Despawning the owner removes all particle models. Recursive
child model paths are blocked. Child meshes and bind poses share the existing
prepared-model cache; materials and animation belong to each instance.

PREM latitude is interpreted directly in radians; longitude is ignored. Local
emission direction combines a random Z heading with a Y tilt within the latitude
range. World rotation is applied before componentwise world scale to
velocity. Gravity is multiplied by world Z scale, and each child receives a
random world Z heading. Position is evaluated analytically from age, initial
velocity, and constant gravity.

Remaining limits:

- `EmitterUsesTga` image particles are not rendered.
- Emission rate and visibility use the update endpoint. Updates crossing keys or
  sequence boundaries can miss births; interval integration remains work.
- Each emitter is capped at 1,024 live model particles and 1,024 attempted births
  per update. Excess births are discarded, rather than queued after a stall.
- Models use the existing mesh material pipeline, including its documented
  limitations in billboarding, shading, and effect/material pass ordering.
- Exact game fidelity, mirrored transforms, and attachment/particle combinations
  need comparison against Warcraft III captures.

Capture the fixture with the existing offscreen example:

```sh
cargo run -p bevy-wc3 --example capture -- \
  crates/bevy-wc3/tests/fixtures/prem_capture.mdl /tmp/wc3-prem-captures \
  --times 0,0.5,1,1.5,2 --fps 60 --size 640x480 \
  --eye 0,-18,9 --target 0,0,2
```

The fixture reuses `attachment_capture_child.mdl` and `capture_white.png`. It
isolates a moving emitter with visibility keys and a nonuniformly scaled cone.
GPU captures at the listed times were visually inspected for detached birth
positions, motion, model scale, and independently animated geometry. The current
`data/` corpus has no populated PREM chunks, so it cannot establish game-model
PREM fidelity.
