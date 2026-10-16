# M3 material roles

M3 standard materials retain their authored shader inputs in the semantic model.
The renderer interprets a bounded set of these roles through Bevy's lighting
pipeline. Per-layer wrap flags select independent sampler variants without
changing shared source textures. Other M3 material families still require dedicated interpreters. See
the [renderer overview](renderer.md) for the wider rendering boundary.

Primary diffuse and emission inputs supply color, opacity and emission bindings.
The base normal input uses the native alpha/green packing: alpha supplies tangent
X, green supplies tangent Y, and positive Z is reconstructed from the unit
hemisphere. This changes the normal decoder without enabling Warcraft III HD
material roles. Base normals currently sample UV0; unsupported coordinate,
sampling or contribution programs produce a diagnostic while this approximation
is displayed.

Ambient occlusion maps use Bevy's red-channel occlusion input only when their
authored behavior can be represented directly. This includes untransformed,
unanimated UV0 sampling, RGB/RGBA/red channel selection, unit map alpha and no
noise, video, flipbook, Fresnel, color arithmetic, shared coordinate source or
unrecognized sampling flags. Other occlusion programs remain unresolved and
produce a diagnostic instead of assigning an unrelated channel to Bevy.

Static constant-color specular inputs receive an explicit PBR approximation.
Known channel selection and RGB versus alpha-only specular modes determine the
specular tint and bounded intensity. The Blinn-Phong exponent controls approximate
highlight width through Bevy's perceptual roughness. This does not reproduce the
native distribution or HDR energy and always produces a diagnostic stating that
limitation. Textured and animated specular programs remain unresolved. Gloss maps
are retained without guessing a roughness channel or transfer function.

Standard material priority, two-sided rendering and unshaded rendering are
applied alongside these roles. Layer composition, the two authored opacity
programs, supplementary normals and environment programs remain incomplete.
Implemented normal packing and role routing still need matched game-reference
validation; successful import alone does not establish material fidelity.
