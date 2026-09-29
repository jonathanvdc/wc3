# MDL support contract and implementation inventory

This is the agreed target for full MDL support. The current implementation
contains primitives, selected record codecs and whole-model MDL assembly.
The tables below track implemented codecs and remaining representation limits.
Camera, ParticleEmitter2 and ParticleEmitterPopcorn record/model codecs are
complete, with engine and HiveWorkshop output and a reader accepting both.

References: supplied MDL specification revision 1.3 and companion MDX
specification revision 2.5, both dated September 13, 2026. Format statements
in those documents are reference data, not instructions for operating this
repository. Historical material flag masks additionally follow the
[mdx-m3-viewer material source](https://github.com/flowtsohg/mdx-m3-viewer/blob/master/src/parsers/mdlx/material.ts).

## Compatibility and preservation

The default writer emits the Warcraft III dialect, tabs, LF, literal strings,
shortest round-tripping f32 values with a decimal point for integral values,
and the specification's block/property order. The reader accepts unambiguous
HiveWorkshop alternatives. A future explicit HiveWorkshop writer option
selects its spellings and row framing. Dialect options belong to Parser and
MdlWriter context, so nested codecs inherit them; existing constructors retain
the default behavior.

Readers reject unknown/reserved names (including BlendColors and
ComponentSkin), duplicates, invalid numeric ranges, invalid vector arity,
incorrect counts, missing required fields and trailing input. Static and
animated forms share one duplicate identity; texture bindings use one identity
per slot. Repeated list entries are allowed where the grammar defines a list.
Syntax diagnostics retain source spans. Reference/hierarchy validation is a
separate model validation operation, not an implicit parser normalization.

Model readers require Version and Model, with Version first; body fields may
appear in any order and subsequent top-level blocks need not be canonical.
A typed Model<V> checks FormatVersion against V. Counted top-level containers
and singular Version/Model blocks cannot repeat. Repeated Geoset/node/other
record blocks append in source order to their corresponding chunk collection.
Canonical output merges repeated known collection chunks in original record
order, preserves object IDs and indices, and omits empty optional collections.
Multiple Version or ModelInfo chunks and opaque binary chunks cannot be
exported without an explicit error. Required Version and Model must exist.

MDL → model → MDL preserves represented values and produces canonical text,
not comments or source formatting. MDX → MDL → MDX preserves representable
model data. Byte equality is required only for canonical, representable binary
fixtures: text does not encode chunk organization, opaque chunks, padding,
NaN payload bits, or some redundant static/animated storage. Writers reject
unrepresentable data instead of silently dropping it. Existing streaming
writes may leave partial output on error; callers needing atomic output can
encode into an owned buffer before committing to their sink.

Floats compare by bits, including signed zero, except all NaNs compare by
class. A writer omits a default only when reading that omission reconstructs
the value. Animated properties keep their documented default base on read;
a nondefault binary base hidden by a track causes a write error. Fixed text
must be valid UTF-8, terminated, and have zero padding. Quotes and NUL in
strings are unrepresentable; backslashes and CR/LF are literal.

The library preserves historical material flags with known binary meanings;
it does not imitate the client's discard of those flags. Material TwoSided
also enables TwoSided on every layer when reading text, regardless of order.
Material Unfogged is accepted as the documented ignored client directive:
neither supplied specification defines a stored material bit for it. It must
not be assigned a guessed bit or interpreted as layer Unfogged. Unknown raw
material bits remain binary data and are rejected by MDL writers.

Camera scalar DOFDistance, FocalLength and FStop construct a stepped key at
frame zero with no global sequence. The default reader uses their intended
meaning and does not emulate the client's swapped FocalLength/FStop defect.
Writers always use FocusDistanceKeys/FocalLengthKeys/FStopKeys. Unknown shader
names are errors in layer text codecs, not silent downgrades. Raw unnamed
shader IDs remain binary data and cannot be emitted as an invented name.

Glider/DILG and the additional camera tracks were introduced by the 3.0.0
client without a binary layout version gate. They remain representable at
all supported versions. DILG retains all IDs in order; the client's slot-zero
overwrite defect is not emulated. This is distinct from fields whose binary
layout is explicitly gated below.

## Block and property mapping

Defaults below apply to omitted optional properties. Headers and fields marked
required must be present. Unlisted numeric base defaults are zero, flags false,
strings empty, references absent (u32::MAX where the binary format uses it),
and tracks/collections empty. Examples do not imply defaults. Future codecs
must retain these defaults explicitly rather than infer them from example data.

| MDL block / properties | Model storage | Defaults and special handling | Gate / status |
| --- | --- | --- | --- |
| Version: FormatVersion | VersionChunk<V> | FormatVersion required; no binary extension spelling | Supported V: 800, 900, 1000, 1100, 1200, 1300, 1400, 1600, 1800; derived codec complete |
| Model name; BlendTime; MinimumExtent, MaximumExtent, BoundsRadius | ModelInfo | Name required; extents/blend zero; animation-file data has no spelling | Derived codec; canonical order and BlendTime omission complete |
| Sequences → Anim name; Interval, NonLooping, MoveSpeed, Rarity, SyncPoint, extent | Sequence | Name/Interval required; flags/numeric options zero | Existing record codec |
| GlobalSequences → Duration | GlobalSequence | Counted durations | Existing entry codec |
| Textures → Bitmap: Image, ReplaceableId, WrapWidth, WrapHeight | Texture | Empty image, zero ID/flags | Existing record codec |
| Materials → Material: PriorityPlane, ConstantColor, TwoSided, SortPrimsNearZ, SortPrimsFarZ, FullResolution, Unfogged, Shader, Layer | Material<V> | Signed priority; historical flag masks 1,2,8,16,32; Unfogged ignored; TwoSided also applied to layers | Shader storage only 900/1000; codec complete |
| Layer: FilterMode | Layer<V>.filter_mode | IDs 0–6: None, Transparent, Blend, Additive, AddAlpha, Modulate, Modulate2x | All versions; codec complete |
| Layer: Unshaded, SphereEnvMap, WrapWidth, WrapHeight, TwoSided, Unfogged, NoDepthTest, NoDepthSet, Unlit, BackFacesForShadows, AmbientOcclusion | LayerShadingFlags | Bits 0–10 in that order; false | Ungated flags; codec complete |
| Layer: Shader / ShaderTypeId | ShaderType wrapper / shader_type | SD default 0; named IDs 0,1,2,24; case-insensitive | Per-layer storage ≥1100; codec complete |
| Layer: TextureID, TVertexAnimId, CoordId, Alpha | Layer<V>, LayerTrack, LayerTextureSlot | Texture 0; absent texture-animation sentinel; coordinate 0; alpha 1 | Slot records ≥1100; both dialect codecs complete |
| Layer: EmissiveGain | Version-selected field + LayerTrack | 1 | ≥900 |
| Layer: FresnelColor, FresnelOpacity, FresnelTeamColor | Version-selected field + LayerTrack | White color; scalars zero | ≥1000 |
| TextureAnims → TVertexAnim: Translation, Rotation, Scaling | TextureAnimation | Track-only transform channels | Existing record codec |
| Geoset: Vertices, Normals, repeated TVertices, VertexGroup, Faces/Triangles, Groups/Matrices | Geoset<V> | Exact array/count/group structure; no implicit mesh repair | Derived record codec complete |
| Geoset: Tangents, SkinWeights | Geoset extra sections | Empty; skin coexists with legacy groups; bare engine rows | ≥900; u16 bone indices ≥1400 cannot fit the spec's 0–255 MDL form and must be rejected unless representable |
| Geoset: MaterialID, SelectionGroup, Unselectable, LevelOfDetail, extent, repeated Anim extents | Geoset<V>, GeosetExtent | Selection bit 4; zero extents; LOD 0 | LOD storage ≥900; derived record codec complete |
| GeosetAnim: Alpha, DropShadow, GeosetId, Color | GeosetAnimation | Alpha 1, white color; required GeosetId; Color presence enables use-color bit | Existing record codec |
| Bone: node fields, GeosetId, GeosetAnimId | Bone | Multiple / None map to u32::MAX | Derived codec complete |
| Helper | Node | Shared node only | Derived codec complete |
| Light: node fields, Omnidirectional/Directional/Ambient, AttenuationStart/End, Color, Intensity, AmbColor, AmbIntensity, Visibility | Light<V>, LightTrack | White colors; other base values zero | Derived codec complete |
| Light: ShadowIntensity | Version-selected field | Zero; static only | ≥1200 storage; codec complete |
| Light: ShadowCasting, ShadowCastingStart/End | Version-selected fields + LightTrack | False, zero | ≥1300 |
| Light: QuadraticFalloff, LinearFalloff, Damping | LightFalloff + LightTrack | 0.0005, 0, 0.00001, also effective below 1600 | ≥1600 storage |
| Attachment: node fields, AttachmentID, Path, Visibility | Attachment | Zero ID, empty path, no track | Derived codec complete |
| PivotPoints | PivotPoint | Counted Vec3 entries indexed by ObjectId | Existing entry codec |
| ParticleEmitter: node fields, EmitterUsesMdl/Tga, EmissionRate, Gravity, Longitude, Latitude, Path, LifeSpan, InitVelocity, Visibility | ParticleEmitter, ParticleTrack | Zero bases; emitter flags preserved exactly; reserved word unrepresentable | Derived codec complete |
| ParticleEmitter2: node fields, SortPrimsFarZ, LineEmitter, Unfogged, ModelSpace, Unshaded, XYQuad; Speed, Variation, Latitude, Gravity, LifeSpan, EmissionRate, Length, Width | ParticleEmitter2, Particle2Fields/Track | Zero bases, absent tracks | Derived codec complete |
| ParticleEmitter2: Blend/Additive/Modulate/Modulate2x/AlphaKey; Rows, Columns; Head/Tail/Both; TailLength, Time; SegmentColor, Alpha, ParticleScaling; LifeSpanUVAnim, DecayUVAnim, TailUVAnim, TailDecayUVAnim; TextureID, Squirt, PriorityPlane, ReplaceableId | Particle2Fields | Exactly 3 colors/alphas/scales and 3 integers per UV interval; unsigned PRE2 priority | Derived codec complete |
| RibbonEmitter: node fields; HeightAbove/Below, Alpha, Color, TextureSlot, Visibility; LifeSpan, EmissionRate, Rows, Columns, MaterialID, Gravity | RibbonEmitter, RibbonFields/Track | Zero bases, no tracks; unsigned integer emission rate | Derived codec complete |
| ParticleEmitterPopcorn: node fields; SortPrimsFarZ, Unshaded, Unfogged, PopcornScaling; LifeSpan, EmissionRate, Speed, Color, Alpha, ReplaceableId, Path, AnimVisibilityGuide | PopcornEmitter, PopcornTrack | Scalars 1, white color; literal multiline strings; flags are emitter-specific aliases | Derived codec complete; model chunk capability ≥900 |
| EventObject: node fields, EventTrack | EventObject | Signed i32 frames; global-sequence ID has no documented MDL spelling | Derived codec complete; reject non-absent global sequence on write until a spelling is verified |
| Camera: Position, Translation, Rotation, FieldOfView, FarClip, NearClip, Target { Position, Translation }, Visibility | Camera<V>, CameraTrack | FOV/FarClip required; zero positions/near clip; scalar roll | Derived codec complete; default binary variant and canonical channel order required |
| Camera: DOFDistance/FocusDistanceKeys, FocalLength/FocalLengthKeys, FStop/FStopKeys | CameraTrack | Track-only storage; scalar makes one time-zero step key | Ungated; derived enclosing codec complete |
| CollisionShape: node fields, Box/Plane/Sphere/Cylinder, Vertices, BoundsRadius | CollisionShape | Vertex arity 2/2/1/2; radius only sphere/cylinder | Derived codec complete |
| FaceFX name; Path | FaceFx | Required header; empty path | Derived codec complete; library chunk capability ≥900 |
| BindPose → Matrices | BindPoseChunk, BindPoseMatrix | Counted 12-float matrices | Derived codec complete; library chunk capability ≥900 |
| Glider: GeosetId | Glider, GlidersChunk | GeosetId required; repeated uncounted blocks; empty list omitted | Ungated; MDX and derived MDL codecs complete |

Shared node fields are name (header), required ObjectId, optional Parent,
DontInheritTranslation/Rotation/Scaling, Billboarded/LockX/LockY/LockZ,
CameraAnchored, and Translation/Rotation/Scaling tracks. Missing parent is
u32::MAX. Static transform channels have no node spelling. Object-kind bits
are not printed as flags; codecs must make preservation checks against the
owning record's chosen reconstruction convention instead of normalizing
arbitrary binary flags silently. Emitter aliases must be interpreted in their
own chunk context, especially CORN bits 17/18.

All ordinary tracks carry count, interpolation (DontInterp/Linear/Hermite/
Bezier), optional GlobalSeqId, signed i32 times, and typed values; spline
tracks require both tangents per key. Counts and ordering are retained.
EventTrack has signed times only, no interpolation or value. Color ordering
is property-specific: GeosetAnim Color is BGR; do not reverse all colors.

ShadowIntensity is treated as static only under the agreed support contract.
The supplied MDL spec describes animation, but the companion MDX track inventory
defines no corresponding tag and the inspected WhiteoutLib implementation
provides only the static binary field. Animated ShadowIntensity is rejected;
it is not pending codec work. The documented engine animated TextureID form has no slot
selector: an HD non-diffuse slot track cannot be exported through that spelling
without verified context. HiveWorkshop named slot tracks retain slot identity.
These are explicit unsupported-representation errors until resolved, not
permission to discard animation.

## HiveWorkshop alternatives

| Alternative | Mapping / preservation |
| --- | --- |
| Layer ShaderTypeId | Raw shader ID, ≥1100; engine output requires a known name |
| TextureID, NormalTextureID, ORMTextureID, EmissiveTextureID, TeamColorTextureID, ReflectionsTextureID | Slots 0–5; static and animated; conflicts with engine spelling share slot identity |
| SortPrimitives | Alias for material SortPrimsFarZ |
| Braced SkinWeights rows | Same payload as bare rows |
| SelectionFlags | Raw geoset selection flags; engine output only represents 0 or Unselectable bit 4 |
| LevelOfDetailName | Existing geoset LOD name; engine output rejects a nonempty name |
| Missing engine-only flags | Both dialects preserve the full known flag set per user preference, including material near-Z and layer WrapWidth/WrapHeight/Unlit |

## Stage 2 API changes and defaults audit

EventObject::new/set_frames/frames now use i32 instead of u32. Material
priority_plane/set_priority_plane now use i32 instead of u32. The four-byte
binary layout is unchanged. Callers migrating raw words must reinterpret bits
with i32::from_le_bytes(raw.to_le_bytes()); do not clamp or reject negative
values. PRE2 priority remains u32 per its binary definition.

New camera kinds: KCVS Visibility, IDUF FocusDistanceKeys, ELAF FocalLengthKeys,
PTSF FStopKeys. Constant helpers use a time-zero key; MDL scalar reading follows
the intended names and keyed writing avoids the client defect. Existing camera
variant bytes and conversion behavior remain preserved.

Glider/GlidersChunk integrate with typed and runtime model access, chunk
recognition, and conversion. New named layer bits cover wrap and shadow/AO
flags. ShaderType wraps every raw u32 ID and supplies constants for the four named
pipelines. It is the actual version-selected field storage, replacing
LayerShaderType. Layer shader_type/set_shader_type (and checked equivalents)
use the wrapper; MaterialLayout::ShaderType selects it from version 1100.
ShaderType::new/id preserve raw IDs and name returns None for unnamed IDs. Historical material TwoSided and near-Z bits gain accessors.

Light::new now uses white direct/ambient colors. PopcornEmitter::new now uses
unit lifespan/emission/speed/alpha and white color. Existing binary decoders
continue reading actual stored values, including zeros. Layer alpha/emissive
and Fresnel white defaults, GeosetAnimation defaults, absent-reference sentinels,
and LightFalloff defaults were already correct. Other emitter defaults remain
zero where the supplied specifications do not define a nonzero default.

## Field-owned MDL property codecs

Ordinary named fields can use `#[mdl(property = "Name", delegate)]` with
`mdl::ReadProperty` / `mdl::WriteProperty`. The field type owns availability,
missing-field reconstruction, payload framing, and full-property omission.
The derive still owns dispatch, duplicates and output ordering, and calls
representability validation before emitting the record. This mechanism has
no version numbers or version attributes; version-selected storage types
supply different codecs. Static/animated channel delegation remains a separate
extension. The artificial-version regression test exercises required available
storage and unavailable storage without value codecs or Default bounds.

## Completion checks for later stages

Every row needs an independent text fixture, canonical output fixture, omitted
default and duplicate/error coverage. Test supported/unsupported version
boundaries, spline tangents, signed times, exact finite floats, skin widths,
shader names, slots and both dialect forms. Whole-model tests must include the
spec's minimal quad and independent models spanning all versions. Compare
semantic data across formats; compare binary bytes only for representable
canonical fixtures. Optional asset-corpus checks supplement synthetic fixtures.
Run workspace tests, formatting, clippy and allocation regressions.

## Geoset record codecs

`Geoset<V>` now derives MDL Read/Write directly. Virtual fields adapt existing
mesh storage without cloning on write. Ordinary dispatch, duplicate handling,
extents and flags use derives; property adapters handle count-prefixed rows,
uncounted vertex groups, the two-count Faces/Groups headers, and bare/braced
skin rows. Repeated UV sets and animation extents retain their order.

Per-vertex section lengths and both mesh group counts are checked. Version
availability comes from selected storage. Explicit empty unavailable sections
are rejected. Warcraft III output rejects non-triangle primitive groups,
unknown selection flags, nonempty or noncanonical LOD names, inconsistent
binary group counts, and skin bone indices above 255. HiveWorkshop
SelectionFlags/LevelOfDetailName input is retained, with errors when engine
output cannot represent it. Reference validation remains separate.

Independent quad text, canonical output and binary fixtures exercise the
record codec; whole-model assembly is complete. Geoset selection accessors
now inspect and change mask 4, preserving other raw bits instead of treating
the complete flags word as a boolean.

## Whole-model MDL assembly

`Model<V>` and `DynamicModel` now implement MDL Read/Write. Typed readers verify
FormatVersion; runtime readers dispatch all nine supported layouts. Version
must be first; Model may follow other optional blocks but must exist. Counted
containers and BindPose are singular. Repeated record blocks append to their
corresponding typed collection without changing IDs or references.

Writers merge repeated known collection chunks in record order, emit canonical
block order and omit empty optional collections. Required Version/Model chunks
must appear exactly once, with no extensions. Opaque binary chunks fail explicitly. Camera/PRE2 collections are supported
in every version; CORN requires version 900 or newer.
Record-level representability checks remain active. ModelInfo now writes
BlendTime first when nonzero, then minimum/maximum extent and radius.

The supplied complete quad has independent input, canonical-text and MDX
fixtures. Model tests cover every supported version, noncanonical input order,
count/duplicate diagnostics, merged chunk collections and preservation errors.

## Camera and remaining emitters

Camera fields and nested Target use derives with borrowed channel adapters.
Scalar lens aliases become time-zero stepped tracks; output uses keyed forms.
Duplicate aliases/target tracks fail. Nondefault binary camera variants and
noncanonical channel ordering fail rather than silently changing the record.

PRE2 uses derived projected fields, exclusive filter/head-tail choices, and
a focused exactly-three-color adapter. UV intervals and alphas are fixed-size
arrays. MDX now stores length before width, correcting the old swapped order.
CORN reconstructs its own emitter flag aliases and preserves literal multiline
visibility guides. Unit/white animation bases are checked before writing.

Independent MDX fixtures exercise the three supplied examples (PRE2 uses
unequal width/length). Tests cover all nine model versions, tracks/tangents,
negative times, global sequences, duplicate/range/count errors, preservation
failures and allocation-free output.

## Dialect output

`Dialect::Warcraft3` is the default. `Dialect::HiveWorkshop` is selected on the
streaming writer or `encode_mdl_with_dialect`; model and nested codecs inherit
it. Readers accept mixed input without silently choosing an output dialect.
Property and flag aliases share duplicate identity. All known flags are
accepted and emitted in both dialects; only their alternate spellings differ.

Hive output preserves numeric shader IDs (including unnamed IDs), all six named
static/animated HD slots, raw selection flags and nonempty LOD names. Engine
output rejects those values when its spelling cannot preserve them. Skin row
payloads are identical, with braces selected only for Hive output. Unknown bits,
invalid slots, duplicate assignments and hidden animation bases remain errors.

Derives support hive_name and hive_flags spelling overrides. The complete flag
set belongs in flags; Hive overrides may cover only names that differ. Both
prepare_mdl_fields and validate_mdl_property take Dialect directly. Named slot
animations borrow the common track parser/writer rather than copying payloads.
Tests include independent engine/Hive text and MDX fixtures, all nine versions,
whole-model Hive-only data, aliases, failure spans and allocation-free output.
