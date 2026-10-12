//! Codec derives for Warcraft III, World of Warcraft, and StarCraft II model records in `wc3`.
//!
//! These macros are for maintaining or extending the record implementations.
//! Applications using the built-in model types only need the codec traits in
//! `wc3::model::mdlx::{mdx, mdl}`, `wc3::model::m2`, `wc3::model::wmo`, and `wc3::model::m3`.
//!
//! Import macros through those format modules, for example
//! `#[derive(mdl::Read, mdl::Write)]`. Ownership is shared through
//! `wc3::model::common::Ownership`. The format references follow.
#![doc = include_str!("mdx.md")]
#![doc = include_str!("mdl.md")]
#![doc = include_str!("m2.md")]
#![doc = include_str!("wmo.md")]
#![doc = include_str!("m3.md")]
#![deny(missing_docs)]
mod mdl;
mod mdx;
mod mdx_owned;

use mdl::expand as expand_mdl;
use mdx::{expand as expand_mdx, expand_value};
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

/// Reads fields in declaration order. A sized record's final `Vec<T>` consumes its body.
///
/// Numeric enums use `#[mdx(value = u32)]` with explicit `#[mdx(value = N)]`
/// unit variants and an optional `#[mdx(unknown)]` single-u32 fallback variant.
#[proc_macro_derive(MdxRead, attributes(mdx))]
pub fn derive_mdx_read(input: TokenStream) -> TokenStream {
    expand_mdx(parse_macro_input!(input as DeriveInput), true).into()
}

/// Writes fields in declaration order. A sized record's final `Vec<T>` is written elementwise.
/// Numeric enums write their explicit wire value or the unknown payload verbatim.
#[proc_macro_derive(MdxWrite, attributes(mdx))]
pub fn derive_mdx_write(input: TokenStream) -> TokenStream {
    expand_mdx(parse_macro_input!(input as DeriveInput), false).into()
}

/// Reads an MDL block, single-value property, anonymous entry, or keyword enum.
/// Value and choice enums may mark one payload variant `#[mdl(unknown)]`; it has no text spelling.
#[proc_macro_derive(MdlRead, attributes(mdl))]
pub fn derive_mdl_read(input: TokenStream) -> TokenStream {
    expand_mdl(parse_macro_input!(input as DeriveInput), true).into()
}

/// Writes an MDL block, single-value property, anonymous entry, or keyword enum.
/// Writing a value or choice enum's `#[mdl(unknown)]` variant returns an unsupported-value error.
#[proc_macro_derive(MdlWrite, attributes(mdl))]
pub fn derive_mdl_write(input: TokenStream) -> TokenStream {
    expand_mdl(parse_macro_input!(input as DeriveInput), false).into()
}

/// Generates const `raw()` and `from_raw()` methods from numeric MDX mappings.
/// With an unknown variant, `from_raw()` returns `Self`; without it, `Option<Self>`.
#[proc_macro_derive(MdxValue, attributes(mdx))]
pub fn derive_mdx_value(input: TokenStream) -> TokenStream {
    expand_value(parse_macro_input!(input as DeriveInput)).into()
}

mod ownership;
/// Compares versioned model records across different buffer lifetimes.
#[proc_macro_derive(ModelPartialEq)]
pub fn derive_model_partial_eq(input: TokenStream) -> TokenStream {
    ownership::expand_eq(parse_macro_input!(input as DeriveInput)).into()
}

/// Implements buffer ownership operations without requiring any format codec.
///
/// Records use a `'data` lifetime for borrowed buffers. Named and tuple structs
/// and enums are supported. Fields containing that lifetime delegate to the
/// shared ownership traits; other metadata fields clone for views and move when
/// detached. Type and const parameters remain unchanged.
#[proc_macro_derive(Ownership)]
pub fn derive_ownership(input: TokenStream) -> TokenStream {
    ownership::expand(parse_macro_input!(input as DeriveInput)).into()
}

mod binary;
/// Reads M2 record fields in declaration order within a shared address space.
#[proc_macro_derive(M2Read)]
pub fn derive_m2_read(input: TokenStream) -> TokenStream {
    binary::expand(
        parse_macro_input!(input as DeriveInput),
        binary::Operation::Read,
    )
    .into()
}
/// Writes M2 fields after reserving the record's complete inline metadata.
#[proc_macro_derive(M2Write)]
pub fn derive_m2_write(input: TokenStream) -> TokenStream {
    binary::expand(
        parse_macro_input!(input as DeriveInput),
        binary::Operation::Write,
    )
    .into()
}
/// Sums the inline metadata widths of an M2 record's fields.
#[proc_macro_derive(M2RecordLayout)]
pub fn derive_m2_record_layout(input: TokenStream) -> TokenStream {
    binary::expand(
        parse_macro_input!(input as DeriveInput),
        binary::Operation::Layout,
    )
    .into()
}

/// Compares borrowed model structs across different source-buffer lifetimes.
///
/// Requires `Ownership`; comparisons use shared borrowed views rather
/// than constraining the original address-space lifetimes to be identical.
#[proc_macro_derive(CommonPartialEq)]
pub fn derive_common_partial_eq(input: TokenStream) -> TokenStream {
    ownership::expand_eq(parse_macro_input!(input as DeriveInput)).into()
}

/// Decodes MDX values into independently owned storage.
///
/// Lifetime-bearing values must implement the shared `IntoOwned` contract;
/// this derive does not generate ownership operations or require a read derive.
#[proc_macro_derive(MdxReadOwned)]
pub fn derive_mdx_read_owned(input: TokenStream) -> TokenStream {
    mdx_owned::expand(&parse_macro_input!(input as DeriveInput)).into()
}

mod binary_traversal;
/// Validates struct fields or the active enum payload in declaration order.
/// Returns the first field error; cross-field invariants use handwritten impls.
#[proc_macro_derive(M2Validate)]
pub fn derive_m2_validate(input: TokenStream) -> TokenStream {
    binary_traversal::expand(
        parse_macro_input!(input as DeriveInput),
        binary_traversal::Operation::Validate,
    )
    .into()
}
/// Visits struct fields or the active enum payload in declaration order.
/// Context-dependent leaves provide their own traversal implementations.
#[proc_macro_derive(M2VisitArrays)]
pub fn derive_m2_visit_arrays(input: TokenStream) -> TokenStream {
    binary_traversal::expand(
        parse_macro_input!(input as DeriveInput),
        binary_traversal::Operation::VisitArrays,
    )
    .into()
}

/// Derives field-order WMO binary decoding.
#[proc_macro_derive(WmoRead)]
pub fn derive_wmo_read(input: TokenStream) -> TokenStream {
    binary::expand_format(
        syn::parse_macro_input!(input as syn::DeriveInput),
        binary::Operation::Read,
        binary::Format::Wmo,
    )
    .into()
}
/// Derives field-order WMO binary encoding.
#[proc_macro_derive(WmoWrite)]
pub fn derive_wmo_write(input: TokenStream) -> TokenStream {
    binary::expand_format(
        syn::parse_macro_input!(input as syn::DeriveInput),
        binary::Operation::Write,
        binary::Format::Wmo,
    )
    .into()
}
/// Derives fixed inline WMO record width.
#[proc_macro_derive(WmoRecordLayout)]
pub fn derive_wmo_record_layout(input: TokenStream) -> TokenStream {
    binary::expand_format(
        syn::parse_macro_input!(input as syn::DeriveInput),
        binary::Operation::Layout,
        binary::Format::Wmo,
    )
    .into()
}
/// Derives compositional WMO record validation.
#[proc_macro_derive(WmoValidate)]
pub fn derive_wmo_validate(input: TokenStream) -> TokenStream {
    binary_traversal::expand_format(
        syn::parse_macro_input!(input as syn::DeriveInput),
        binary_traversal::Operation::Validate,
        binary::Format::Wmo,
    )
    .into()
}

/// Derives envelope-generic M3 record decoding.
#[proc_macro_derive(M3Read)]
pub fn derive_m3_read(input: TokenStream) -> TokenStream {
    binary::expand_format(
        syn::parse_macro_input!(input as syn::DeriveInput),
        binary::Operation::Read,
        binary::Format::M3,
    )
    .into()
}
/// Derives envelope-generic M3 record encoding.
#[proc_macro_derive(M3Write)]
pub fn derive_m3_write(input: TokenStream) -> TokenStream {
    binary::expand_format(
        syn::parse_macro_input!(input as syn::DeriveInput),
        binary::Operation::Write,
        binary::Format::M3,
    )
    .into()
}
/// Derives the inline M3 record width under either reference envelope.
#[proc_macro_derive(M3RecordLayout)]
pub fn derive_m3_layout(input: TokenStream) -> TokenStream {
    binary::expand_format(
        syn::parse_macro_input!(input as syn::DeriveInput),
        binary::Operation::Layout,
        binary::Format::M3,
    )
    .into()
}
/// Derives compositional M3 semantic validation.
#[proc_macro_derive(M3Validate)]
pub fn derive_m3_validate(input: TokenStream) -> TokenStream {
    binary_traversal::expand_format(
        syn::parse_macro_input!(input as syn::DeriveInput),
        binary_traversal::Operation::Validate,
        binary::Format::M3,
    )
    .into()
}
/// Derives immutable traversal of native M3 section references.
#[proc_macro_derive(M3VisitReferences)]
pub fn derive_m3_visit_references(input: TokenStream) -> TokenStream {
    binary_traversal::expand_format(
        syn::parse_macro_input!(input as syn::DeriveInput),
        binary_traversal::Operation::VisitReferences,
        binary::Format::M3,
    )
    .into()
}
