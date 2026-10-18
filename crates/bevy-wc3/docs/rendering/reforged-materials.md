# Reforged DefaultUnit surfaces

DefaultUnit materials combine diffuse, normal, surface, emissive, team-color,
and reflection inputs within Bevy's PBR pipeline. This guide describes those roles,
animation, and the remaining native shading limits.

A WC3 `MaterialStep` keeps pass state separate from its typed shader inputs.
`Shader::Classic`, `Shader::ReforgedDefault`, and `Shader::Unresolved` distinguish
their interpretation. `MaterialInputs` supplies the primary texture, opacity,
and emissive gain; `ReforgedInputs` adds HD texture roles and Fresnel controls.
Native import chooses the shader, with numeric layer selection taking precedence
over the material shader name.

Modern native classic layers can carry mandatory neutral Fresnel storage and a
canonical diffuse slot zero. Import normalizes those fields into the classic
primary input, preserving its texture animation; export restores diffuse-slot
storage for versions 1100 and later. Nondefault or animated Fresnel, duplicate or
unknown slots, and conflicting legacy bindings remain unresolved rather than
losing authored information.

Native import currently retains one step per native layer. Older six-layer DefaultUnit layouts therefore do not yet combine
into one set of semantic role bindings. Unknown/Crystal shaders retain a diagnosed
diffuse fallback; they do not receive DefaultUnit slot rules merely because they
contain several textures.

## PBR integration and roles

The extension preserves Bevy forward PBR, dynamic lighting, shadows, fog controls,
and post-processing. It registers local shader assets without replacing Bevy's
global pipeline. Compiled HD opaque surfaces use forward rendering; forcing them
through deferred shading is unsupported.

| Role | Interpretation |
| --- | --- |
| Diffuse | Color and alpha, multiplied by surface and mesh tint/opacity. |
| Normal | Linear RG tangent normal; positive Z reconstructed with a clamped square root. |
| ORM | Linear red occlusion, green perceptual roughness, blue metallic, alpha team mask. |
| Emissive | Color multiplied by animated emissive gain and mesh tint. |
| Team | Diffuse multiplies white-to-team color according to ORM alpha. |
| Reflection | Equirectangular environment color with roughness mip bias and metallic/Schlick weighting. |

Reflection is an approximation of authored native shading. Bevy world directions are
mapped into the authored environment convention. Missing normal/ORM roles retain a
rough nonmetallic base; absent team/emissive/environment maps contribute nothing.
Ordinary Bevy reflectance remains active on shaded surfaces.

## Images and animated coordinates

Role-aware `TextureRequest` values request linear normal/ORM images and sRGB color
images unless an explicit semantic declaration overrides that choice. The loader
creates separately labeled sampler/color-space variants without modifying source
images. The same resource can safely appear as both color and numeric data. Custom
resolvers must return images with the requested interpretation.

Role texture choices, gain, Fresnel controls, and UV transforms animate through the
same semantic programme as nodes. Each layer selects its authored coordinate set.
Equal compiled coordinates share geometry, exact differences retain distinct meshes,
and authored tangents retain their source basis. Missing requested coordinates
produce compilation errors.

The native emissive-gain header admits NaN as an unspecified unity default.
Semantic import resolves that header to one and reports the representation
normalization while retaining its complete animation. Other nonfinite values
and invalid animation keys remain rejected.

Fresnel contributes an authored fifth-power rim with color/team influence, opacity,
and mesh tint, separate from Bevy's physical surface Fresnel. Unlit and unfogged
controls follow their Bevy equivalents. Surface properties sample the destination
directly during node-pose transitions.

## Passes and limits

HD Transparent uses a 0.75 alpha cutoff. Forward and supported prepasses retain
matching alpha/UV/skinning interpretation, including eight influences. Normal
prepass reconstructs the same RG normal. Independent blend/depth specialization
preserves authored states. A default extension used directly with an application's
StandardMaterial leaves its base blend/depth policy intact.

Priority is a Bevy sort/depth bias. Native primitive sorting is not applied to mesh
triangles, and a dedicated native mesh/effect ordering scheme is absent. Crystal,
unknown shader programmes, Popcorn, and facial animation require additional
interpreters. Implemented DefaultUnit behavior remains an authored PBR
interpretation, not a guarantee of exact game lighting.
