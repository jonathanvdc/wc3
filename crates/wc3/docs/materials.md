# Materials and texture inputs

Materials keep the shader and pass behavior authored by each model family. The
shared graph lets tools inspect and edit those definitions without replacing them
with a generic shader. Start with the [model guide](semantic.md) for dependency
ownership and [conversion](conversion.md) for export policies.

## Ordered material steps

A material owns a program whose authored steps describe shader execution or
material composition. Universal's `MaterialSurface` is a
`MaterialProgram<MaterialStep>`, with an ordered `steps` collection.

`MaterialStep` selects the dialect of one operation: `Mdlx`, `M2`, or `Wmo`.
Each dialect lives under the matching `model::materials` namespace and defines
its own typed shader inputs and rendering settings. A single program can combine
steps from different dialects. Shared texture bindings, blend operations, depth
state, and rasterization components are reused where their meanings agree.

## Warcraft III shaders

`mdlx::MaterialStep` owns pass state, draw declarations, and a `Shader` whose
variants are `Classic(MaterialInputs)`, `ReforgedDefault(ReforgedInputs)`, and
`Unresolved(UnresolvedInputs)`. Common `MaterialInputs` contain the primary
texture, opacity, and emissive gain. Blend, depth, and rasterization state belong
to the pass independently of shader selection. Reforged inputs add ordered
texture slots and Fresnel controls; unresolved inputs preserve those fields plus
the uninterpreted shader name and selector.

Native import interprets shader selection before constructing the semantic model.
Numeric layer selectors take precedence over material shader names. Known shader
variants carry neither IDs nor names; native export derives their selectors.
Texture slot order, duplicate selectors, and unknown roles remain intact. Raw
native layers remain available through the codecs for editing encoded declarations.

Each step carries priority, sorting, and additional draw flags in a
`MaterialDeclaration`. Checked native MDLX export requires those declarations to
agree, because the native material stores them once. Unresolved names must also
agree when several unresolved steps share a native material. Semantic programs
can retain known steps alongside unresolved names, but exact export requires a
layout that can encode every declaration. Layouts with only a material shader name cannot encode per-layer overrides.
Layouts with per-layer selectors cannot encode a nonempty unresolved shader name. An empty
program carries no shader declarations. Nondefault headers on empty native
materials remain editable through the native codecs; semantic import rejects
them unless an explicit loss policy accepts dropping them.

The MDLX profile stores `MaterialProgram<mdlx::MaterialStep>`. Widening wraps
each step without changing it; narrowing checks its dialect. Export then checks
whether the selected native layout can encode the complete program.

## M2 and WMO shaders

M2 and WMO use shader-specific `MaterialStep` enums in both their native profiles
and Universal. There is no general material input struct between those profiles.
The M2 native profile stores an ordered program of those steps; WMO retains one
step per native material. M2 steps distinguish modulation, additive equations, weighted interpolation,
masked crossfades, facing opacity, and guild composition. WMO steps distinguish
diffuse, reflection, layered, emission, and modulation equations. Each payload
names its active texture roles; a `MaskedDualCrossfadeMaterialStep`, for example,
owns `base`, `first`, `second`, and `alpha_mask` inputs. Guild palettes belong to
the guild payloads. Shared draw declarations remain on each step.

Known steps derive their native selectors from the equation and coordinate
choices. Native encoding aliases normalize to the same definition where their
semantics agree. Unresolved selectors and incompatible resource layouts retain
their raw selection and complete binding order on `UnresolvedMaterialStep`.
Legacy M2 surplus inputs and unused WMO slots remain explicit unused bindings,
preserving graph references without turning them into active shader inputs.
Animation follows the input-owned slot references through input and material-step
reordering. Native export resolves each consuming field to its current native
position.

M2 skin batches become ordered steps within draws; see the
[M2 guide](m2.md#import-for-shared-editing-or-rendering) for grouping and export rules.

## Texture ownership and animation

Texture inputs compose through `TextureBinding<Image, Coordinates, Sampling,
Contribution>`. The selected image representation can retain animated texture
choices, runtime replacement categories, external resource identities, or explicit
absent-image codes. `Coordinates<Source, Transform>` separates mesh UV or generated
coordinate selection from its input-owned transform. Animated TRS values retain their typed properties. Native transform conventions and units
are preserved. MDLX transform slots belong to each material input,
so editing one imported use does not affect another use of the same native record.

Native import expands texture and transform table references into complete
input values while sharing image declarations. Animated MDLX image choices carry
the selected image's resource handle and sampler with each key. Export rebuilds
native indirection tables and interns equal records; native table identity and
unused records are not semantic authoring objects. Uninterpreted native texture
flag bits are discarded during semantic import; U/V wrapping is represented by
the input sampler and reconstructed during export. Native codec documents retain
the original flag words when semantic conversion is unnecessary. `Model::sampled_images()`
collects distinct sampled-image values from materials, effects, allocated value
slots, and animation tracks. This does not introduce shared editable texture
objects.

Sampling belongs to the input that uses it. WMO steps retain two independently
authored scrolling slots even when their image bindings are absent. These
distinctions do not require optional fields on every texture binding.

The following M2 step names its modulation input while reusing the shared texture
binding vocabulary:

```rust
use wc3::model::materials::m2::{MaterialStep, ModulateMaterialStep, TextureUnit};
use wc3::model::{Construct, PropertyValues};

let mut values = PropertyValues::default();
let mut modulation = TextureUnit::construct(&mut values);
modulation.coordinates.source = 1;
values.set(modulation.contribution, 0.75)?;
let step = MaterialStep::Modulate(ModulateMaterialStep {
    modulation,
    ..ModulateMaterialStep::construct(&mut values)
});
let MaterialStep::Modulate(value) = step else { unreachable!() };
assert_eq!(values[value.modulation.contribution], 0.75);
# Ok::<(), wc3::model::Error>(())
```

## Draw state

A binding supplies a shader input; a step describes an operation.
`PassState` owns draw state, and `Rasterization` separates depth, face, lighting,
and additional controls. `DepthState` distinguishes comparison, writes, and bias.
Explicit blend equations retain factors, operation, enablement, and constant color.
M2 helpers expose known rasterization controls while retaining unresolved flag bits.
