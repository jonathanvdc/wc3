# Model lights

[Documentation index](../README.md) · [Verification notes](../verification.md)

## Bevy integration

Model omnidirectional and directional records create ordinary Bevy `PointLight`
and `DirectionalLight` entities. They illuminate ordinary Bevy materials as well
as WC3 materials and shaded effects. No global shader or scene-lighting resource
is replaced. Bevy controls light accumulation, PBR response, shadows, exposure,
and tonemapping.

Each imported record has a public `Wc3Light` component, with its animation root,
an `enabled` switch, and a read-only `definition()` accessor. The light entity is
a child of its animated node, so pivots, parent transforms, inherited visibility,
attachments, and recursive cleanup use the existing hierarchy. Directional light
local +Z is the surface-to-light direction; Bevy emits along local -Z. Full
parent rotation is inherited. General node-flag limitations still apply.

Color, intensity, attenuation end, and visibility sample the existing
sequence/global clocks, including pause and seeking. Visibility is an on/off
gate (`> 0`), not an intensity multiplier. Authored RGB values are treated as
linear, consistent with the other model color controls.

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

## Source fields and scene lighting

Ambient records remain inspectable `Wc3Light` entities without a Bevy light.
They log an unsupported-mapping warning. Direct lights' ambient contribution is
also not applied. Importing an asset never modifies `GlobalAmbientLight`, nor
does it convert ambient illumination into emissive material color.

Attenuation start, Reforged quadratic/linear/damping falloff, shadow intensity,
and shadow-casting start/end are retained in the source definition but are not
mapped. Bevy uses its own distance falloff and shadow projections. An authored
attenuation-start distance is not a physical source radius. Unknown light types
remain inspectable and warn without creating an arbitrary Bevy light.
