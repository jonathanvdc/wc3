# bevy-wc3 documentation

## Visual fidelity

These notes track missing rendering behavior and implemented behavior that still
needs comparison against Warcraft III captures. Keep each rendering topic in its
own file under `visual-fidelity/` and link it here as it is documented.

- [General renderer gaps](visual-fidelity/renderer.md): Classic/Reforged
  materials, UV animation, node flags, pass ordering, skinning
  and bind poses, LOD, replaceable textures, lights, cameras, events, and
  Reforged effects, plus an overview of implemented effects and fidelity checks.
- [Reforged materials](visual-fidelity/reforged-materials.md): HD texture roles,
  animated controls, Bevy integration, captures, and remaining fidelity work.
- [Geoset animation](visual-fidelity/geoset-animation.md): alpha/color sampling,
  RGB codec normalization, material isolation, capture checks, and remaining fidelity work.
- [Classic PREM particles](visual-fidelity/prem.md): model spawning, motion,
  child animation, resource ownership, and remaining limits.
- [Ribbon emitters](visual-fidelity/ribbons.md): GPU trails, material layers,
  atlas animation, ballistic gravity, and capture verification.
- [PRE2 particle emitters](visual-fidelity/pre2.md): fog, lighting, textures,
  pass ordering, emission timing, spawn motion, and orientation.

Each topic should describe the current behavior, the remaining implementation or
verification work, and any limitations of completed visual checks. Update the
notes when a gap is resolved so they reflect the current renderer.
