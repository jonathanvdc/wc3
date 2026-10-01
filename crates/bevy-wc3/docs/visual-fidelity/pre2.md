# PRE2 visual fidelity

[Documentation index](../README.md)

Spawn records use a growing ring, with chronological retirement and no free-slot
list. Capacity starts at 16 records and doubles with the live population, up to
8,192 records. Render extraction shares immutable blocks of up to 64 records,
so births copy only touched blocks. GPU preparation tracks the last uploaded
birth cursor and writes at most two contiguous ranges across wraparound. Growth
or a full ring lap between rendered frames uploads the entire current buffer.
Retirement alone does not upload spawn records. Per-view depth sorting still
uses a separate draw-order index buffer.

The following behavior is missing or still needs comparison against Warcraft III
captures. Verification tasks describe implemented behavior whose exact visual
match has not been established.

| Area | Current behavior and remaining work |
| --- | --- |
| Fog | The particle shader applies no fog, so all emitters behave as Unfogged. Apply scene fog to emitters without that flag. |
| Shaded lighting | Unshaded bypasses lighting, while shaded particles use Bevy scene lighting with a matte, zero-reflectance material. This differs from the Classic clamped lighting equation; brightness, color, and light response still need matching. |
| Replaceable textures and team color/glow | Explicit per-instance bindings work, but PRE2 replaceable IDs have no automatically loaded defaults. Team-specific texture selection and any required atlas selection remain unimplemented. |
| PriorityPlane and pass ordering | PriorityPlane is a bias in Bevy's transparent sorting. Verify Warcraft's ordering between emitters and its interaction with model materials and other effects. |
| Squirt timing | Only the current emission key is observed, so an update crossing multiple keys or loops can miss bursts. Schedule every crossed burst at its actual birth time. |
| Animated continuous emission | Continuous births have subframe timestamps and sample spawn parameters and transforms at birth. Emission rate and visibility are sampled at the update endpoint, so changes within an update can alter particle counts and birth times. Integrate rate and visibility over the interval, including sequence boundaries. |
| Spawn motion | Speed variation is multiplicative, and world-space velocity uses the emitter's full affine transform. Verify variation semantics and rotation/scale order with rotated, nonuniformly scaled emitters. |
| ModelSpace gravity | Gravity is multiplied by the emitter's Z-axis length at birth, then local motion is transformed by the current node transform. Verify the intended gravity space and scaling to avoid an extra scale factor. |
| XYQuad orientation | Heads stay in world XY and retain the initial XY velocity angle minus pi plus pi/8. Verify the exact facing offset and UV orientation against asymmetric textures and Warcraft captures. |
| Mirrored transforms | Quad size multipliers use transformed axis lengths, which discard scale signs. Verify mirrored emitters and determine whether quad or texture orientation must retain those signs. |

SortPrimsFarZ already sorts live particles by analytic center depth along each
camera's forward axis, rather than radial distance. Continuous births are also
already distributed within an update; they are not all grouped at its endpoint.
