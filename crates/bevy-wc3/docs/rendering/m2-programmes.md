# M2 material programme coverage

This inventory separates the established regular shader table from legacy
selector bits and special material families. It records the renderer's supported
interpretations and the specific information still missing. A parsed native
material or a declared texture count alone does not establish rendering support.
For composition, animation and Bevy integration, see [M2 materials](m2-materials.md).

## Regular shader table

Selectors with the high bit set address the following 36 established entries.
The table index is the selector minus 32768. Counts below are sampled image
inputs for the established programme, not an assertion that every native batch
with that selector has the same binding layout. All supported entries retain
Bevy PBR lighting and native framebuffer blend/depth choices; additive texture
contributions enter emissive.

`U` denotes UV0 with native transform slot zero, `V` denotes UV1 with its native
transform, and `E` denotes sphere environment coordinates. Entries 20 and 24
read the UV1 transform from native slot two. Repeated `U` routes share their
controls during native import. Native lighting and exact game pass ordering
remain approximate even for implemented equations.

| Index | Composition | Sampled images / coordinates | Status and additional inputs |
| --- | --- | --- | --- |
| 0 | Opaque masked doubled modulation | 2: U, E | Implemented |
| 1 | Opaque additive alpha | 2: U, E | Implemented |
| 2 | Opaque additive alpha with inverse base mask | 2: U, E | Implemented |
| 3 | Masked doubled modulation and glow | 3: U, E, U | Implemented |
| 4 | Modulated additive alpha | 2: U, E | Implemented |
| 5 | Opaque additive alpha | 2: U, U | Implemented |
| 6 | Modulated additive alpha | 2: U, U | Implemented |
| 7 | Modulated additive alpha with inverse base mask | 2: U, E | Implemented |
| 8 | Double alpha interpolation | 2: U, E | Implemented |
| 9 | Doubled modulation and third-source interpolation | 3: U, E, U | Implemented |
| 10 | Weighted opaque additive alpha | 2: U, U | Implemented |
| 11 | Modulated additive inverse base mask | 2: U, E | Implemented |
| 12 | Opaque masked modulation | 2: U, E | Implemented |
| 13 | Weighted modulated additive alpha | 2: U, E | Implemented |
| 14 | Weighted modulated additive alpha | 2: U, U | Implemented |
| 15 | Weighted opaque additive alpha | 2: U, V | Implemented |
| 16 | Weighted modulation and additive contribution | 2: U, E | Implemented |
| 17 | Masked doubled modulation and unshaded alpha | 3: U, E, U | Implemented |
| 18 | Dual crossfade | 3: U, U, U | Implemented; weights of inputs one and two |
| 19 | Modulated edge opacity | 1: U | Implemented facing fade; no scene-depth intersection |
| 20 | Doubled modulation and double interpolation | 3: U, E, V | Implemented |
| 21 | Modulated product and edge opacity | 2: U, V | Implemented facing fade |
| 22 | Masked dual crossfade | 4: U, U, U, V | Implemented; fourth image masks interpolated alpha |
| 23 | Opaque alpha interpolation | 2: U, U | Implemented |
| 24 | Masked doubled modulation and unshaded alpha | 3: U, E, V | Implemented |
| 25 | Environment edge opacity | 1: E | Implemented facing fade; no scene-depth intersection |
| 26 | Guild base, emblem and border | 3: U, V, U | Implemented with application-supplied guild palette |
| 27 | Guild base and emblem | 2: U, V | Implemented with application-supplied guild palette |
| 28 | Opaque guild base, emblem and border | 3: U, V, U | Implemented with application-supplied guild palette; six-binding variant unresolved |
| 29 | Illumination | 2: U, U | Unsupported: established RGB interpretation missing |
| 30 | Three-image constant product | 3; third route unresolved | Unsupported: coordinate interpretation and constant sourcing missing |
| 31 | Three-image constant product with colour vertex programme | 3; routes unresolved | Unsupported: vertex colour/coordinate interpretation and constant sourcing missing |
| 32 | Opaque | 1: U | Implemented |
| 33 | Doubled modulated product and edge opacity | 2: U, V | Implemented facing fade |
| 34 | Modulated edge opacity | 1: U | Implemented facing fade |
| 35 | Modulated product and depth/edge opacity | 2: U, V | Implemented facing fade; no scene-depth intersection |

The masked crossfade at entry 22 is the only established four-image equation in
this table. There is no additional five-image masked-crossfade entry in the
inventoried table. Unsupported selectors and incompatible counts retain explicit
fallback diagnostics rather than wrapping indices or guessing equations.

## Resource and API requirements

Every interpreted image uses the semantic texture binding and normal resource
provider, including native replacement categories. Each input retains its weight
and effective coordinate transform. Input zero normally also contributes to
surface opacity unless the native batch disables that multiplier. Four-input
masking uses the existing independent UV transport; simple materials do not gain
another vertex stream or sample unused images.

Guild programmes additionally require background, emblem and border colour
multipliers. `materials::m2::GuildPalette` supplies these through the `palette`
field of `GuildMaterialStep`, `GuildEmblemMaterialStep`, and
`OpaqueGuildMaterialStep`, with neutral white defaults. These settings belong to the application;
the native material record does not encode them. A live Bevy material exposes
`ModelMaterialExtension::guild_palette` and `set_guild_palette`, allowing colour
changes without rebuilding the model or changing the shader pipeline. Bevy material
asset sharing applies to those edits. Exporting a nondefault palette
to native M2 requires an explicit loss policy. Import does not infer palette
colours from unresolved external replacement resources.

The known modern six-binding guild batch declares skin, emblem, skin, background
colour, emblem colour and border colour replacements. The skin image is repeated.
The interpretation of the final colour resources is not established, so this
layout remains a diagnosed fallback rather than being silently treated as the
three-image programme. Its missing work is colour-resource interpretation, not a
promised new generic six-image shader stage.

## Other selector and material families

Modern low-bit selectors form a separate bitfield family. Their recognized mask
is `0x40ff`: the lower equation fields, environment selectors and UV1 control.
Other upper control bits produce a diagnosed fallback. The established equations
sample one or two images and select ordinary or environment coordinates. A batch
can declare surplus bindings; these remain retained and diagnosed while the
known first-two-input equation executes. Arbitrary negative coordinate selectors
are not assigned invented semantics. Historical combiner fields and native
blend-override table normalization need separate interpretation; the modern
nonzero-primary-field rule does not establish those older equations. Native
single-input UV1 selectors also retain raw transform-table addressing; separate
allocation of their second transform slot is not normalized by conversion.

Projective materials need projection, stencil and depth integration beyond a
regular mesh combiner. Waterfall markers and WFV3 controls also describe a
separate material family. Their native codecs preserve the records, but the
renderer does not interpret these special paths. Nonzero material TXAC controls
likewise require an explicit native import loss policy. Native fade, detail and
textured-light extension records do not become implemented effects merely by
being decoded; see the [native coverage guide](../../../wc3/docs/m2.md#coverage-and-preservation-boundaries).

The retained controls below also delimit material coverage. They are independent
of the regular table's texture arity and should not be counted as additional
masked-crossfade stages.

| Control family | Current boundary | Required interpretation or integration |
| --- | --- | --- |
| Material TXAC | Nonzero values require explicit import loss | Meanings of the material selector bytes |
| EDGF | Native records decoded; extended fade controls not rendered | Authored fade parameters and their camera-dependent evaluation |
| NERF / DBOC | Native records decoded; opacity controls not rendered | Native opacity/distance-control interpretation |
| DETL / TEXL | Native records decoded; lighting controls not rendered | Detail/textured illumination interpretation and Bevy lighting integration |
| Legacy input frame selection | Retained semantic frame controls not evaluated | Frame-to-image playback semantics |
| Batch layer and pass ordering | Priority/depth bias approximation | Exact ordering across Bevy render phases |
| Other retained batch/material extension bits | No general interpreter | Established meanings for the individual controls |

The remaining regular-table research set is exactly entries 29, 30 and 31, plus
the modern six-binding guild resource layout. Projective, waterfall, historical
selector normalization and unresolved extension behavior remain separate named
boundaries. Future native layouts or shader tables require their own
interpretation; they are not automatically covered
by this inventory.
