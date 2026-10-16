# M2 texture composition

M2 surfaces retain their authored framebuffer blend and depth states while
combining supported texture inputs inside `ModelMaterial`. Diffuse composition
feeds Bevy albedo. Native additive texture contributions feed emissive, so they
remain visible alongside Bevy lighting without replacing its PBR pipeline. This
preserves the authored texture equations but approximates the game's lighting
response, especially effects described by native shaders as specular.

## Shader selectors and coordinates

Legacy selectors support one input and the common two-input multiplication,
addition and alpha combinations. The environment bits and explicit coordinate
source `-1` select view-space sphere environment mapping. Nonnegative sources
select the declared mesh coordinate stream; other negative sources remain
unsupported. Generated coordinates use the animated/skinned normal and current
view, bypassing texture transforms. Ordinary UV inputs retain independent
animated transforms.

The high-bit shader table interprets entries 0 through 21, 23 through 25, and
32 through 35. These include environment modulation, masked additive contributions,
weighted three-texture crossfades, glow composition and edge opacity. Input counts
must match the selected programme. Legacy selectors with surplus bindings use
their established first-two-input equation and diagnose the unsampled bindings.
All three interpreted inputs can select independent mesh coordinate streams or
explicit environment coordinates and retain their own animated transforms. Native
import resolves repeated routes into shared controls; see the [M2 conversion
guide](../../../wc3/docs/m2.md#shader-coordinate-ownership).
Unknown table entries and unsupported input counts produce an explicit first-input
fallback diagnostic instead of wrapping the table index.

Shader 32768 uses base-map alpha to interpolate between doubled environment
modulation and the base diffuse color. That alpha is a texture-composition mask,
not surface transparency. Forward, depth, normal and shadow passes share the
same composition function, including the independent alpha rules of other
supported shaders. Generated coordinates also remain available in depth-only
and shadow vertex layouts.

## Animation and opacity

Surface RGB, opacity, all three input weights and UV transforms sample the
model's selected animation programmes. The first input's weight normally
multiplies global opacity. Batch flag `0x40` suppresses that opacity multiplier
while retaining the weight as an independent shader control. Secondary and third
weights affect the particular equations that author them; they are not
unconditionally multiplied into diffuse opacity.

Material sampling updates assets only when an evaluated value changes. Texture
count is uniform across a draw, and unused secondary/third image samples are
skipped. Generated-coordinate and edge-opacity layouts are specialized separately
from ordinary UV layouts. A third texture reuses the primary or secondary raw UV
stream when possible, while retaining its own transform. Only distinct or generated
third routes add a mesh stream. Coordinate choices participate in compiled mesh
sharing; source textures and unchanged material definitions retain their sharing.

## Remaining interpretation limits

Four-input masked crossfades, guild-colour programmes and the incompletely
established illumination/constant programmes remain unsupported. Guild-style
six-input batches retain a first-input fallback; they are not reduced to a
three-input guild equation.

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
