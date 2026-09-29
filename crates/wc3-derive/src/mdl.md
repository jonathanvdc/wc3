# MDL derives

This reference is for maintaining the record codecs in `wc3`. Applications
using built-in records do not need these attributes.

The derives map Rust fields to MDL syntax. Start with a record shape, annotate
its fields, and define defaults for values that may be absent. Reading accepts
fields in any order and rejects duplicate or unknown names. Writing follows
field declaration order unless `write_order(...)` specifies another order.

## A simple record

`block = "Name"` gives a named-field struct a complete MDL block. A `header`
field appears before the opening brace; properties and flags appear inside it.

```rust
use wc3::model::{mdl, FixedText};
use wc3::model::mdl::{Read as _, Write as _};

#[derive(mdl::Read, mdl::Write)]
#[mdl(block = "Example")]
struct Example {
    #[mdl(header)]
    name: FixedText<80>,
    #[mdl(property = "Size")]
    size: f32,
    #[mdl(flag = "Visible", default)]
    visible: bool,
}

let record = Example::decode_mdl(r#"Example "Demo" { Size 2.0, Visible, }"#)?;
assert_eq!(record.name.text(), "Demo");
assert_eq!(record.size, 2.0);
assert!(record.visible);
assert_eq!(record.encode_mdl()?, "Example \"Demo\" {\n\tSize 2.0,\n\tVisible,\n}\n");
# Ok::<(), Box<dyn std::error::Error>>(())
```

Each field needs one representation:

| Field attribute | MDL form |
| --- | --- |
| `header` | Positional value before `{`; always required. |
| `property = "Size"` | `Size value,` |
| `static_property = "Alpha"` | `static Alpha value,` |
| `flag = "Visible"` | `Visible,`; the Rust field is `bool`. |
| `flags(Visible = 1, Unshaded = 2)` | Named flags stored in one bitfield. |
| `skip` | No text form; supply a default for reading. |

Animation, nested records, and custom storage use the forms below.

## Defaults and omission

Properties, static properties, and boolean flags are required by default.
Use field `default` for `Default::default()` or `default = "factory"` for a
zero-argument function returning the value. Boolean flags support only the
bare field `default`, which is false. A required flag must be true on output.

Record-level `#[mdl(default)]` takes omitted body fields from `Self::default()`.
Field defaults override that record default. Use field `required` to keep a
property, static property, nested block, or counted list required.

A default allows missing input; it does not by itself omit a property on output.
Use `skip_if = "predicate"`, with signature `fn(&T) -> bool`, to omit a defaulted
property, static property, or animatable field. The writer checks that reading
the omission would restore the same value and returns an error otherwise.
Headers cannot have defaults or omission predicates.

These checks use `mdl::ValueEq`. Floats compare by bits, including signed zero;
NaNs compare by class because text cannot retain their payload bits. Custom
value codecs should implement equality consistent with their text representation.
Skipped data needs a `validate_write` hook if discarding it could lose information.

### Optional properties

Use `property = "Name", delegate` with `Option<T>` when absence should mean
`None`. `Some(value)` always writes the property, even for a default value.

```rust
use wc3::model::mdl;
use wc3::model::mdl::{Read as _, Write as _};

#[derive(mdl::Read, mdl::Write)]
#[mdl(block = "Example")]
struct Example {
    #[mdl(property = "Value", delegate)]
    value: Option<f32>,
}

assert!(Example::decode_mdl("Example {}")?.value.is_none());
let record = Example { value: Some(0.0) };
assert!(record.encode_mdl()?.contains("Value 0.0,"));
# Ok::<(), Box<dyn std::error::Error>>(())
```

Other delegated types implement `mdl::ReadProperty` and `mdl::WriteProperty`.
They handle the payload punctuation, missing values, validation, and complete
property output. The enclosing record handles the name and duplicates.
Delegation cannot combine with `default`, `required`, `skip_if`, `read_with`,
or `write_with`.

## Nested records and lists

| Field attribute | Use |
| --- | --- |
| `flatten` | Put a field group's headers and body fields in the parent. |
| `block = "Target"` | Put a field group in one nested `Target { ... }` block. |
| `repeated = "Anim"` | Collect zero or more complete `Anim` records into a `Vec<T>`. |
| `counted = "Points"` | Read or write one `Points N { ... }` list as a `Vec<T>`. |

Use record-level `#[mdl(fields)]` to derive a reusable field group without an
outer block. Block records also implement the field-group traits. Flattened
groups retain their own defaults; parent defaults do not replace them.
Overlapping field names are rejected.

Nested blocks and counted lists are required unless given a default. An empty
counted list writes a zero count. Repeated records preserve source order and
initialize an empty collection when absent. List items handle their own names,
headers, and punctuation. For scalar or vector entries, use a tuple wrapper
with `#[mdl(entry)]`.

`repeated(Translation, Rotation, Scaling)` accepts several record names in one
collection. Add `unique_by = "Type::key"` to reject duplicate keys on both read
and write. Structural fields do not support `read_with`, `write_with`, or
`skip_if`; flattened and repeated fields cannot take field defaults.

`write_order(...)` must list every body field once, including nested fields and
collections. It does not reorder headers. Nested blocks have no trailing comma.

## Animation

An animatable property has a base value and a track variant. Put its tracks in
the record's single `#[mdl(tracks)]` vector:

```rust
use wc3::model::animation::GeosetTrack;
use wc3::model::mdl;
use wc3::model::mdl::{Read as _, Write as _};

#[derive(mdl::Read, mdl::Write)]
#[mdl(block = "Example", default)]
struct Example {
    #[mdl(animatable = "Alpha", track = "GeosetTrack::Alpha")]
    alpha: f32,
    #[mdl(tracks)]
    tracks: Vec<GeosetTrack>,
}

impl Default for Example {
    fn default() -> Self { Self { alpha: 1.0, tracks: Vec::new() } }
}

let fixed = Example::decode_mdl("Example { static Alpha 0.5, }")?;
assert_eq!(fixed.alpha, 0.5);
let animated = Example::decode_mdl("Example { Alpha 0 { Linear, } }")?;
assert_eq!(animated.alpha, 1.0);
assert!(!animated.encode_mdl()?.contains("static Alpha"));
# Ok::<(), Box<dyn std::error::Error>>(())
```

The base needs a field or record default. Static input sets it; animated input
adds a track and leaves the base at its default. Static and animated forms count
as the same property for duplicate checks. Track variants must wrap readable
tracks with matching MDL names.

Writing uses the track when present, otherwise the static base. An animation
supersedes its base value; reading the output restores that base to its default.
Tracks start empty on read and retain their order; duplicate or unmapped variants are errors.
Animated properties are emitted at their base field’s position in `write_order`
or declaration order. Channels without base fields are emitted at the tracks
field’s position, in `channels(...)` declaration order. Writing leaves stored
track order unchanged.

| Modifier | Purpose |
| --- | --- |
| `tracks, channels(Visibility = "Track::Visibility")` | Add a channel with no base field; static input is rejected. |
| `animated_only` | Accept only a track for an animatable field; the base must remain at its default. |
| `bare_static` | Also accept `Name value,` as static input; output still uses `static Name value,`. |
| `enabled_if = "predicate", enable_with = "function"` | Tie a property's presence to a flag. |

The paired enable hooks take `&Self -> bool` and `&mut Self`. Reading calls the
enable hook if either form was present. Writing omits a disabled static property
and rejects a disabled track. Animatable fields cannot use value codec hooks.

## Flags and dialect aliases

Packed `flags(...)` mappings use unique, nonzero, single-bit `u32` masks.
Storage may be `u32` or a type implementing `BitRange<u32>`; reading also needs
`BitRangeMut<u32>` and a default. Storage starts at zero unless the record
supplies a default. Output follows mapping order and rejects unknown bits.
Factory defaults, value hooks, and `skip_if` are not supported on packed flags.

`allow_bits = MASK` permits bits represented by other properties or hooks.
Those bits are not printed and must not overlap the mapped flags. Validate them
in `validate_write` when the attributes do not fully describe their meaning.

Use `hive_name = "OtherName"` for an alternate HiveWorkshop property spelling.
`hive_flags(...)` overrides selected flag spellings by bit; `flags(...)` still
declares the complete set. Aliases share duplicate detection. The writer's
dialect selects the output spelling throughout nested records.

`hive_skip_bits = MASK` on packed flags omits the selected mapped bits in
HiveWorkshop output while retaining them in engine output and parsed storage.
This supports engine-only flags without changing dialect spellings for other bits.

## Tuple wrappers and enums

Single-field tuple structs support `#[mdl(property = "Duration")]` for a complete
named property or `#[mdl(entry)]` for an anonymous list entry. Both include a
trailing comma. They accept validation hooks, but no field attributes or
`write_order`.

Enums have two forms:

| Record attribute | Variant syntax | Meaning |
| --- | --- | --- |
| `value` | Unit variants; optional `name = "Blend"` | One keyword, without punctuation. |
| `tagged` | `flag = "Ready"` | Complete flag and comma. |
| `tagged` | `property = "Duration"` on one payload | Complete named property. |
| `tagged` | `block = "Target"` on one payload | Nested block using the payload's field codecs. |
| `tagged` | `name = "Child", delegate` on one payload | Complete record handled by the payload codec. |

Keyword names default to the Rust variant name and are case-sensitive.
A value enum may mark one payload variant `unknown`; it has no text spelling
and is rejected on output. Tagged payloads must be single unnamed fields;
use a record type to group multiple values. Delegated payloads must use the
declared name. Tagged enums can appear in counted lists.

Name expressions can use constant paths, including associated constants.
Literal names are checked during derivation; constant names are checked before
I/O. Invalid or duplicate names are errors.

## Custom codecs and validation

Function attributes accept ordinary or associated function paths.

| Attribute | Signature | When it runs |
| --- | --- | --- |
| Field `read_with` | `fn(&mut Parser<'_>) -> Result<T, mdl::ReadError>` | Reads a field value. |
| Field `write_with` | `fn<W: std::io::Write>(&T, &mut Writer<W>) -> Result<(), mdl::WriteError>` | Writes a field value. |
| Record `after_read` | `fn(&mut Self, Span) -> Result<(), mdl::ReadError>` | Restores implicit values before validation. |
| Record `validate_read` | `fn(&Self, Span) -> Result<(), mdl::ReadError>` | Validates the completed record. |
| Record `validate_write` | `fn(&Self) -> Result<(), mdl::WriteError>` | Checks the record before output. |

Value hooks work on headers, properties, and static properties. They handle
only the value; the derive handles the name and property comma. Record hook
spans cover the complete record. Enum validation hooks use the same signatures.

## Adapting existing storage

These forms are for records whose Rust layout differs from their MDL fields.

### Projected members

A field annotation such as
`project(#[mdl(property = "Id")] id: u32, ...)` maps members of a nested Rust
struct into the enclosing MDL record. List every member, marking binary-only
members `skip`. Animatable members use the parent's tracks. Record defaults
supply nested defaults, and `write_order` lists the containing field name.

For flags stored inside a flattened record, use
`flatten, extra_flags(get = "Type::flags", set = "Type::set_flags", Extra = 1)`.
The getter returns a bitfield value; the setter stores it. Reading adds these
bits to the flattened record. Validate implicit object-kind and unknown bits
in the enclosing record's write hook.

### Virtual fields

Record-level `virtual_fields(...)` describes MDL fields without corresponding
Rust members. Each needs `get` and either `slot` or `set`. Real members still
need annotations; storage populated by these hooks is usually `skip` with a
default. List virtual fields individually in `write_order`.

| Access pattern | Hook signatures | Behavior |
| --- | --- | --- |
| Optional storage | `get: fn(&Self) -> Option<T>`; `slot: fn(&mut Self) -> Option<&mut T>` | Missing storage omits output and rejects explicit input. Requires a field default. |
| Custom mapping | `get` supplies the output value; `set: fn(&mut Self, T, bool, Span) -> Result<(), mdl::ReadError>` | The setter maps parsed values into the record. The boolean indicates explicit presence. |

Explicit presence includes empty blocks or tracks and default-valued input.
Slot-backed animation rejects unavailable tracks and exports animation in place
of its base value.
Custom setters are responsible for mapping-specific validation.

Getters may borrow collections or property adapters. A delegated output view
must implement `WriteProperty`; a structural view must implement `WriteFields`
with the read type's `State`. Getters must remain stable for an unchanged record.

Reading constructs real members, applies virtual setters in declaration order,
runs enable hooks, then `after_read` and validation. Input order does not change
hook order. Virtual scalar fields need explicit defaults; virtual packed flags
start at zero. Headers and skipped fields cannot be virtual.

## Limits

Derives support generics and add the bounds required by each field. Each field
group supports at most 64 body names, counting individual packed flags.
Conflicting attributes and duplicate literal names are compile-time errors.
