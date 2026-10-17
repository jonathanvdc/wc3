# Reforged DefaultUnit surfaces

The WC3 material vocabulary distinguishes `MaterialStep::Classic`,
`MaterialStep::ReforgedDefault`, and `MaterialStep::Unresolved`.
`ReforgedDefaultMaterialStep` owns the DefaultUnit texture slots and Fresnel inputs;
`ClassicMaterialStep` contains the classic inputs without those HD fields.
Shader selection is interpreted before rendering, with a numeric layer selector
taking precedence over the material shader name. The known variants then own
shader identity without storing redundant names or IDs. Only unresolved steps
retain raw shader selection. Native output derives known selectors from the
variant, while texture slot order remains intact for structural animation targets.

Reforged default materials combine diffuse, normal, surface, emissive, team-color,
and reflection roles in `ModelMaterial`. Native adapters normalize older
six-layer forms into semantic roles before rendering. Unknown/Crystal shaders retain
a diagnosed diffuse fallback; they do not receive DefaultUnit slot rules merely
because they contain several textures.

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
