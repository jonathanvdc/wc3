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

The high-bit shader table currently interprets entries 0 through 17 and entry 23
(selectors 32768 through 32785, and 32791). These cover environment modulation,
masked additive contributions, texture interpolation and selected three-input
glow compositions. Input counts must match the selected programme. Three-input
entries 3, 9 and 17 currently require the third input to use the primary authored
UV stream; it retains an independent transform. Other coordinate combinations,
unknown table entries and unsupported input counts produce an explicit
first-input fallback diagnostic instead of wrapping the table index.

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
skipped. Generated-coordinate layouts are specialized separately from ordinary
UV layouts. Compiled meshes, source textures and unchanged material definitions
retain the renderer's existing sharing behavior.

## Remaining interpretation limits

Higher shader-table entries, three distinct coordinate routes, edge-fade and
crossfade programmes still require additional interpretation. Environment and
additive composition do not establish exact native lighting or transparent-pass
ordering. Unsupported surfaces retain source-associated diagnostics; successful
model loading alone does not establish shader coverage.
