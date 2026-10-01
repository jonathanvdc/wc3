# Model lights

[Documentation index](../README.md)

## Bevy integration

Model omnidirectional and directional records create ordinary Bevy `PointLight`
and `DirectionalLight` entities. They illuminate ordinary Bevy materials as well
as WC3 materials and shaded effects. No global shader or scene-lighting resource
is replaced. Bevy controls light accumulation, PBR response, shadows, exposure,
and tonemapping; this is not Warcraft's lighting equation.

Each imported record has a public `Wc3Light` component, with its animation root,
an `enabled` switch, and a read-only `definition()` accessor. The light entity is
a child of its animated node, so pivots, parent transforms, inherited visibility,
attachments, and recursive cleanup use the existing hierarchy. Directional light
local +Z is the surface-to-light direction; Bevy emits along local -Z. Full
parent rotation is inherited. General node-flag limitations still apply.

Color, intensity, attenuation end, and visibility sample the existing
sequence/global clocks, including pause and seeking. Visibility is an on/off
gate (`> 0`), not an intensity multiplier. Authored RGB values are treated as
linear, consistent with the other model color controls. These color-space and
direction conventions still need verification against Reforged captures.

`Wc3LightSettings` on an animation root controls that instance's imported lights.
It can be supplied before asynchronous spawning or changed at runtime. Absence
uses these defaults:

| Setting | Default and behavior |
| --- | --- |
| `enabled` | `true`; combines with each light's enabled switch and visibility track. |
| `point_intensity_scale` | 1,000 lumens per authored intensity unit. |
| `directional_intensity_scale` | 10,000 lux per authored intensity unit. |
| `range_scale` | 1 world unit per positive attenuation-end unit. |
| `fallback_range` | 1,000 world units when attenuation end is nonpositive/nonfinite. |
| `shadows_enabled` | `true`; allows Bevy shadow maps only when the source ShadowCasting flag is set. |

These are configurable integration defaults, not calibrated physical conversions.
Range does not automatically scale with node/root transforms, matching Bevy
light range behavior. For a model scaled into a different world-unit convention,
set `range_scale` accordingly and tune power for the scene's exposure. Child
model instances have their own animation roots and settings.

The animation system owns light visibility, Bevy light color, power, point range,
and the shadow-map enable flag. Consumer settings such as point radius, shadow biases, contact
shadows, directional cascades, and RenderLayers are preserved. Applications can
find a light by `definition().node.object_id` or node name and disable it with
`Wc3Light::enabled`. Removing `Wc3Light` hands ongoing control of the ordinary
Bevy light to the application; hierarchy-based lifetime remains unchanged.
Directional lights affect the whole scene unless the application scopes them.
Negative/nonfinite colors, powers, and scales are sanitized; point range has a
small positive floor to avoid degenerate Bevy light projections.

## Missing mappings and fidelity limits

Ambient records remain inspectable `Wc3Light` entities without a Bevy light.
They log an unsupported-mapping warning. Direct lights' ambient contribution is
also not applied. Importing an asset never modifies `GlobalAmbientLight`, nor
does it convert ambient illumination into emissive material color.

Attenuation start, Reforged quadratic/linear/damping falloff, shadow intensity,
and shadow-casting start/end are retained in the source definition but are not
mapped. Bevy uses its own distance falloff and shadow projections. An authored
attenuation-start distance is not a physical source radius. Unknown light types
remain inspectable and warn without creating an arbitrary Bevy light. This is
partial model-light support, not a complete Reforged light implementation.

HD portraits in `data/` (including Scarlet Footman, Grunt, and Lich) contain
both point and directional lights, with typical authored intensities around
0.5–1 and attenuation ends of 80. Separate directional/point scales and consumer
control allow these portrait lights to integrate into a larger scene.

## Verification

Unit tests cover animated color/range, global intensity clocks, visibility and
backward seeking, independent instances, parent position/rotation, inherited
visibility, owner cleanup, runtime settings, shadow gates, invalid values,
preservation of consumer radius, and unchanged scene ambient light.

The complete `tests/fixtures/light_capture.mdl` isolates a moving, colored point
light above an HD surface. Its other lights exercise directional/ambient import.
Use the existing capture example:

```sh
cargo run -p bevy-wc3 --example capture -- \
  crates/bevy-wc3/tests/fixtures/light_capture.mdl /tmp/wc3-light-captures \
  --times 0,0.5,1 --fps 60 --eye 0,-7,7 --target 0,1,0 \
  --no-default-light --bevy-reference --prepasses
```

`--no-default-light` omits the example's directional light, leaving scene ambient
unchanged. The offscreen camera supplies a Bevy shadow LOD origin.
GPU captures at 0, 0.5, and 1 seconds were inspected: the point light changes
from blue to red, moves across the surface, changes the ordinary Bevy sphere's
highlight, and stops contributing at the hidden endpoint. These verify Bevy
scene integration, not exact Warcraft appearance.

A temporary variant of the same complete fixture disables the point light and
rotates a white directional light from +Z through +X to -Z. At the same camera,
FPS, and times, inspected GPU images show an illuminated surface at the start,
a side-lit sphere at the midpoint, and ambient-only top-surface illumination at
the endpoint. Point and directional shadow/prepass pipelines complete without
render errors. Exact shadow silhouettes, biases, and game orientation remain
unverified.

Scarlet Footman's HD portrait was also captured at 0.25 and 1 seconds in sequence
0, with the default capture light disabled, an explicit camera framing the head,
and available diffuse/body normal/ORM DDS maps supplied through bitmap overrides.
The model renders with its imported light records; the result is relatively dim
at the default conversion scales. Missing helmet/background/game texture maps
prevent a complete appearance comparison. These captures exercise real portrait
asset integration and do not calibrate power or establish Reforged fidelity.
