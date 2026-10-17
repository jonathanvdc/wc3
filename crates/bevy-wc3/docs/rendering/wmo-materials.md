# WMO materials and regional illumination

World surfaces combine native texture programmes with Bevy PBR illumination.
The renderer interprets established diffuse, layered and emissive equations while
retaining ordinary Bevy lights, shadows, fog and material assets. This guide
separates those equations from specialized native rendering paths that still need
an interpreter. Loading a WMO successfully does not establish full game fidelity.

## Texture programmes

The compiler consumes `wmo::MaterialStep` variants with named texture inputs.
`ReflectionEmissionMaterialStep` owns diffuse and reflection bindings, while
`LayeredDiffuseModulationMaterialStep` owns two diffuse bindings and a modulation
binding. Variants determine the equation, sampled input count, and coordinate
routes; known steps carry no native shader selector. Unused authored slots remain
separate bindings so native projection and graph traversal retain their resources.
Native and Universal WMO profiles use these same definitions.

Specialized or incompletely understood contracts remain `UnresolvedMaterialStep`
definitions. Terrain and LOD paths retain their identity even when their current
rendering approximation uses only the primary diffuse image. The shared GPU
transport does not determine the semantic shader vocabulary.

The regular surface interpreter supports selectors 0, 3, 4, 5, 6, 9, 11, 12,
13, 15, 17, 18, 19 and 20. Texture slots follow their programme's authored UV
routes. Selectors 3 and 5 generate view-space reflection coordinates; selectors
11, 12 and 17 use sphere coordinates for their reflection input. Other layered
programmes use independently authored UV sets, including a third stream where
needed. Missing required coordinate streams fail compilation rather than silently
sampling the first set.

The first two authored UV scroll velocities use absolute instance shared time,
so seeking and repeated sampling do not accumulate error. A reflection input stays
view-derived; the second scroll velocity moves its accompanying diffuse layer.
Unused scroll controls do not make a basic single-texture surface dynamic.
Each WMO image binding owns its sampler, including authored empty slots. Import
expands native material clamp flags into those bindings. Native export requires
the bindings to agree on repeat/clamp addressing because a WMO material stores
one shared pair of address selectors.

Native modulate-add and inverse-source-alpha-add blends retain their separate
RGB and alpha factors in Bevy's transparent phase, without premultiplying the
shader output. Native batch sequence is not enforced across Bevy render phases
and pipeline bins. Coplanar surfaces and overlapping transparent batches can
therefore resolve differently when material or pipeline grouping changes; exact
native overlay ordering remains a rendering limitation.

The second vertex-color stream's alpha controls layer mixtures independently of
the first stream's lighting alpha. Its default is one when the stream is absent.
Emission enters Bevy's emissive contribution; diffuse composition happens before
PBR illumination. Unlit materials combine their supported emission directly.
The same composition equations determine alpha in color, depth and shadow passes.
Shader 4 ignores map alpha while preserving explicit material and instance opacity.

The table below bounds coverage across the known selector range. An implemented
composition still uses Bevy lighting, rather than reproducing native specular or
scene-light calculations.

| Selector | Interpretation | Status |
| --- | --- | --- |
| 0 | Primary diffuse | Implemented |
| 1–2 | Specular and metallic response | First diffuse with diagnostic |
| 3 | Diffuse plus reflected emission | Implemented |
| 4 | Primary diffuse, map alpha ignored | Implemented |
| 5 | Diffuse-weighted reflected emission | Implemented |
| 6 | Layered diffuse | Implemented |
| 7 | Layered diffuse plus emission | First diffuse: third-input coordinate routing unresolved |
| 8 | Terrain combination | First diffuse with diagnostic |
| 9 | Diffuse plus masked emission | Implemented |
| 10 | Window path | First diffuse with diagnostic |
| 11 | Masked diffuse, modulation and reflection | Implemented |
| 12 | Diffuse and multiple emission contributions | Implemented |
| 13 | Opaque layered diffuse | Implemented |
| 14 | Window path | First diffuse with diagnostic |
| 15 | Layered diffuse and emission | Implemented |
| 16 | Terrain blending path | Partial: primary diffuse only, diagnostic |
| 17 | Masked diffuse with added modulation | Implemented |
| 18 | Layered diffuse and doubled modulation | Implemented |
| 19 | Diffuse with doubled modulation | Implemented |
| 20 | Texture-masked layers and doubled modulation | Implemented |
| 21 | LOD path | Partial: primary diffuse only, diagnostic |
| 22–23 | Extended multi-layer paths and overrides | First diffuse with diagnostic |
| 24–25 | Additional modern paths | First diffuse with diagnostic |
| Absent or unknown | Historical or unresolved selector | First diffuse with diagnostic |

Shader 7 has a known composition but unresolved coordinate routing. Its native
resource contract declares two raw UV sets; the third sample's generated-coordinate
mapping is not established.

The extended paths need additional texture resources, coordinate and color streams,
and override interpretation. Specialized terrain and window paths also need scene
inputs beyond the regular material equation. A familiar selector number alone is
not evidence that every native version shares the same resource contract.

## Vertex-light preparation

Native WMO vertex colors remain unchanged in the semantic model. During compilation,
the renderer prepares the first color stream using the owning region's transition
vertex range and the root's ambient and vertex-color options. This happens before
vertex projection and interning, while original vertex indices are available.
Interior and exterior draw classes remain distinct even when they share one source
material. Equal lighting states can share material assets across prepared instances.

Transition vertices subtract the authored root ambient contribution and attenuate
baked RGB according to transition alpha. Other vertices use their authored alpha
for the native RGB preparation before receiving the region's interior/exterior
classification. `WorldOptions::skip_ambient_in_vertex_color` omits root ambient
subtraction; `vertex_color_alpha_only` preserves RGB while correcting the lighting
alpha outside the transition range.

When enabled by the root, portal-distance attenuation prepares transition vertices
against authored portal polygons. The result is static compilation data, independent
of the camera-driven portal visibility policy. Native portal adjustments and
uncertain geometry remain fidelity limits.

The prepared RGB supplies doubled baked diffuse illumination. Root ambient, or an
explicit region ambient override, contributes on the interior side of the lighting
transition. Exterior-lit region and material controls select the exterior state.
These terms enter the final composed diffuse surface through emission, preserving
Bevy's direct and ambient lighting alongside them. Consequently native sunlight
suppression and native indoor/outdoor specular interpolation are not reproduced;
applications retain authority over their Bevy lighting configuration. The first
vertex stream's alpha is lighting data and never becomes surface transparency.

## Scene integration and remaining limits

World groups, regional metadata, placement sets, portal-window visibility and
liquid grids use the shared [static-world integration](universal.md#static-worlds).
Regional fog and ambient-volume sampling expose authored weights to applications;
they do not overwrite a camera's fog or global ambient resources. Baked lightmap
textures and packed rectangles remain metadata requiring an interpreter.

Liquid grids preserve authored heights and tile holes. Database-selected liquid
shaders, water flow and animated liquid response require an application material
provider. Optional doodad and effect models require resource resolution before
their own supported renderer behavior can run. A stone fountain rendering correctly
does not establish that its external water programme is implemented.

Root skybox resources remain source-model metadata and require application scene
integration. Placement illumination and linked-light controls remain hooks rather
than automatic child-material changes. Dedicated native shadow batches retain
their geometry, but their separate shadow-only pass role is not interpreted.

The implemented equations, portal attenuation and native color preparation still
need broader validation across game versions. Native falloff, regional light sets,
lightmap programmes, complex modern overrides, occluder planes and supplemental
portal adjustments remain separate limits.
