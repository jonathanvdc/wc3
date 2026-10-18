# M2 texture composition

M2 surfaces retain their authored framebuffer blend and depth states while
combining supported texture inputs inside `ModelMaterial`. Diffuse composition
feeds Bevy albedo. Native additive texture contributions feed emissive, so they
remain visible alongside Bevy lighting without replacing its PBR pipeline. This
preserves the authored texture equations but approximates the game's lighting
response, especially effects described by native shaders as specular.

Native batches on each skin section are sorted by layer, retaining source order
for ties. Consecutive batches with the same auxiliary section word import as one
draw whose material steps follow that order. Each step
keeps independent color, opacity, and texture animation. The compiler emits passes
in step order; Bevy render phases may still reorder those passes, so this does
not guarantee the game's complete framebuffer ordering. Native priority planes
remain separate per-step composition state.

## Shader definitions and coordinates

The compiler consumes the shader-specific `m2::MaterialStep` enum. Modulation and
addition carry typed alpha choices; crossfades own named base, weighted, and mask
inputs; guild steps own their palettes. Native selectors are interpreted by the
model adapter and derived again on export. The renderer matches these definitions
to its shared GPU composition transport without decoding native selector bits.
Unknown selectors and incompatible input layouts remain unresolved definitions
with their complete authored inputs.

Modern low-bit selectors support one input and the common two-input multiplication,
addition and alpha combinations. The environment bits and explicit coordinate
source `-1` select view-space sphere environment mapping. Nonnegative sources
select the declared mesh coordinate stream; other negative sources remain
unsupported. Generated coordinates use the animated/skinned normal and current
view, bypassing texture transforms. Ordinary UV inputs retain independent
animated transforms.

The high-bit shader table interprets entries 0 through 28 and 32 through 35. The [programme inventory](m2-programmes.md) lists every established
entry, resource requirement and unresolved family. These include environment modulation, masked additive contributions,
weighted and masked crossfades, guild composition, glow and edge opacity. Input counts
must match the selected programme. Legacy selectors with surplus bindings use
their established first-two-input equation and diagnose the unsampled bindings.
Supported inputs retain independent transforms and can select mesh streams or
established environment coordinates. Four-input masking reuses at most three raw
coordinate streams; a fourth environment route or four distinct raw streams
produces a diagnosed fallback. The native UV0/UV0/UV0/UV1 routing is supported. Native
import resolves repeated routes into shared controls; see the [texture ownership
guide](../../../wc3/docs/materials.md#texture-ownership-and-animation).
Unknown table entries and unsupported input counts produce an explicit first-input
fallback diagnostic instead of wrapping the table index.

Shader 32768 uses base-map alpha to interpolate between doubled environment
modulation and the base diffuse color. That alpha is a texture-composition mask,
not surface transparency. Forward, depth, normal and shadow passes share the
same composition function, including the independent alpha rules of other
supported shaders. Generated coordinates also remain available in depth-only
and shadow vertex layouts.

## Animation and opacity

Surface RGB, opacity, up to four input weights and UV transforms sample the
model's selected animation programmes. The first input's weight normally
multiplies global opacity. Batch flag `0x40` suppresses that opacity multiplier
while retaining the weight as an independent shader control. Secondary and third
weights affect the particular equations that author them; they are not
unconditionally multiplied into diffuse opacity.

Guild colours are explicit application settings with neutral white defaults.
Palettes can be supplied on the semantic pass or changed through the live material
API described in the [programme inventory](m2-programmes.md#resource-and-api-requirements).

Material sampling updates assets only when an evaluated value changes. Texture
count is uniform across a draw, and unused additional image samples are
skipped. Generated-coordinate and edge-opacity layouts are specialized separately
from ordinary UV layouts. A third texture reuses the primary or secondary raw UV
stream when possible, while retaining its own transform. Only distinct or generated
third routes add a mesh stream. Coordinate choices participate in compiled mesh
sharing; source textures and unchanged material definitions retain their sharing.

## Remaining interpretation limits

Illumination and constant programmes, the modern six-binding guild resource layout,
and special projective/waterfall families remain unsupported. Historical
combiner and blend-override normalization also remains unresolved. The
[programme inventory](m2-programmes.md) defines these boundaries and their missing
information.

Edge programmes interpolate a per-vertex facing factor,
`clamp(2.7 * facing² - 0.4, 0, 1)`, into opacity. They use the skinned geometric
normal and the current rendering view, including prepasses. They do not sample
scene depth or provide soft intersections. The facing factor does not attenuate
RGB. This opacity-only behavior and its interaction with native pass ordering
remain unverified against the game.
Environment and additive composition preserve Bevy lighting rather than native
lighting. Controlled captures establish the implemented higher-table equations;
they do not establish native visual fidelity for programmes without representative
game assets.
