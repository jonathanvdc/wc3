# M3 materials

M3 Standard materials use a fixed role interpreter within Bevy's PBR pipeline.
Native imports retain independent layers and their animation bindings; the renderer
resolves each image with its authored wrapping and an interpretation appropriate
to its role. A model's own animation binding identity does not require an external
`.m3a` file. Companion documents remain separate resources when actually declared.
Ordinary M3 bones compose parent transforms and retain their inverse bind matrices.
Historical inheritance flag bits remain preserved and diagnostic when populated;
they do not disable the ordinary hierarchy. See [node flags](node-flags.md) and
the [renderer overview](renderer.md) for the shared geometry and scene pipeline.

## Standard roles

The Standard family has eighteen layer roles. Fifteen have an interpreter; the
remaining three roles are retained but not interpreted. A supported
role can still contain unsupported coordinate generation or sampling controls.

| Role | Interpretation |
| --- | --- |
| Diffuse | Image or constant color, native channel selection and team mask; image alpha is not mesh coverage. |
| Decal | Composited over diffuse using the material's layer blend operation. |
| Specular | Animated image or constant tint/intensity, including alpha-only mode; bounded PBR approximation. |
| Gloss | Evaluated alpha squared scales the authored specular exponent before conversion to PBR roughness. |
| Emission 1 and 2 | Independent layers and blend operations, combined after Bevy lighting; HDR scales the additive accumulator. |
| Opacity 1 and 2 | Evaluated alpha values multiply external mesh, particle and composite-pass opacity. |
| Base normal | Native alpha/green tangent X/Y packing with reconstructed positive Z. |
| Additional normals 1 and 2 | Signed, squared mask-weight interpolation from the base normal. |
| Normal masks 1 and 2 | Four channels per mask, with four or eight authored animated weights and enabling flags. |
| Lightmap | Doubled diffuse contribution through Bevy emission; an approximation to native replacement lighting. |
| Ambient occlusion | Evaluated alpha controls Bevy diffuse and specular occlusion. |
| Height | Retained; parallax is not interpreted. |
| Environment and environment mask | Retained; native reflection composition is not interpreted. |

A layer's constant-color flag selects its color property instead of its image.
Unused constant colors do not tint image-driven layers. Supported channel selectors,
map alpha, inversion, enabled multiply/add controls and clamping run in their authored
order. Team masking uses the original sampled alpha. `TeamColors` supplies linear
RGB diffuse and emission colors and their mask exponents; the extension's
`team_colors` and `set_team_colors` methods expose these application-owned values.
Defaults are white with unit exponents. Team-emission blend operations are
implemented; separate team-mask conversion on emission layers remains unresolved.
Color layers decode sRGB unless the source
explicitly declares another transfer function; normals and numeric masks remain
linear. Shared images can therefore have independent sampler and transfer variants.
Application image resolvers must use matching filtering, mipmap and anisotropy
settings for requests with equal authored samplers; those roles share a GPU sampler.
An unresolved diffuse image displays white with a diagnostic. Other unresolved
image roles are disabled, retaining neutral opacity, occlusion and normals instead
of interpreting a placeholder image as authored data.

Explicit UV0 through UV3 have independent centered offset, Z rotation and tiling,
including sampled animation. Extra streams are projected only when used. Packed
normals bypass color contribution arithmetic. Normal blending requires the authored
mask-weight count; missing weights do not receive invented defaults. Historical
material revisions retain their raw flags, whose older normal-blending meanings
are not fully established.

Surface bindings are owned once by the compiled pass. Sampling updates changed
uniform values without rebuilding image tables or GPU geometry. The interpreter
samples active roles and avoids the ordinary StandardMaterial texture evaluation
for these repurposed bindings. Only M3 Standard pipelines compile this interpreter;
other material families retain their own shader paths. The GPU binding layout is
still shared: non-M3 surfaces also carry the additional uniform and image binding
slots, so shader specialization does not isolate their resource-binding overhead.
Fragment-only resources are excluded from vertex-stage visibility to avoid
redundant backend bindings in the color and prepass pipelines.
Matching layer samplers share a bounded pool,
leaving the existing sampler budget available for Bevy lighting and shadows. M3 Standard uses forward rendering because its
emission operations can modify the completed lighting result. Depth/normal and
shadow prepasses share the supported opacity and normal evaluation. Authored
casting and receiving flags independently control Bevy shadow participation.

## Composite and specialized families

Composite sections expand into independent material pass records in authored order.
Their weights multiply pass opacity, including animated and nested composites;
cycles fail preparation. Nonunit or animated weights enable alpha blending on
otherwise opaque sections. Sections share compiled geometry, and sprite/ribbon
passes share their emitter's population and generated mesh. Composite priority
becomes Bevy depth bias. Bevy's render phases and sorting can reorder both sibling
passes and unrelated objects. Native within-composite draw ordering remains partial
and multipass composites emit a diagnostic.

The material inventory is finite and assessed by native family:

| Family | Status |
| --- | --- |
| Standard | Common roles above; lighting approximations and advanced layer limits below. |
| Composite | Weighted pass expansion; native draw ordering remains partial. |
| Displacement | Diagnostic fallback; scene-color/depth distortion is missing. |
| Terrain | Diagnostic fallback; terrain-specific composition is missing. |
| Volume and VolumeNoise | Diagnostic fallbacks; volumetric rendering is missing. |
| Hair | Diagnostic fallback; anisotropic native shading is missing. |
| Creep | Diagnostic fallback; terrain interaction is missing. |
| SplatTerrainBake | Diagnostic fallback; terrain bake integration is missing. |
| Reflection | Diagnostic fallback; native reflection services are missing. |
| LensFlare | Diagnostic fallback; visibility and screen-space flare rendering are missing. |
| DataDriven | Diagnostic fallback; external shader interpretation is missing. |

## Fidelity and remaining limits

Specular tint, intensity and exponent are mapped to bounded Bevy PBR inputs; the
native highlight distribution and HDR energy are not reproduced. Native lightmaps
replace diffuse illumination and suppress specular, while this implementation adds
the baked contribution alongside Bevy lighting. These approximations emit
diagnostics. Bevy continues to supply ordinary lighting and interaction with other
scene assets.

Generated UV mappings, shared UV-source controls, XY texture rotation, Fresnel
presets, video, procedural noise, external image producers and layer flipbooks
remain partial or unsupported and diagnostic. Additional-normal coordinate
conventions have not been established for every native revision. Reflection,
parallax, depth blending and native material-class scene interactions need further
interpretation. Particle and ribbon simulation have separate limits in the
[sprite](pre2.md#m3-standard-billboard-subset) and [ribbon](ribbons.md) guides.
Implemented equations and successful captures do not establish matched game
fidelity across native revisions or all combinations of these controls.
