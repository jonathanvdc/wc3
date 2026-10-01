# bevy-wc3 documentation

## Visual fidelity

These notes track missing rendering behavior and implemented behavior that still
needs comparison against Warcraft III captures. Keep each rendering topic in its
own file under `visual-fidelity/` and link it here as it is documented.

- [General renderer gaps](visual-fidelity/renderer.md): materials, geoset and UV
  animation, billboards, other effects, pass ordering, and skinning limits.
- [Classic PREM particles](visual-fidelity/prem.md): model spawning, motion,
  child animation, resource ownership, and remaining limits.
- [Ribbon emitters](visual-fidelity/ribbons.md): GPU trails, material layers,
  atlas animation, ballistic gravity, and capture verification.
- [PRE2 particle emitters](visual-fidelity/pre2.md): fog, lighting, textures,
  pass ordering, emission timing, spawn motion, and orientation.

Each topic should describe the current behavior, the remaining implementation or
verification work, and any limitations of completed visual checks. Update the
notes when a gap is resolved so they reflect the current renderer.
