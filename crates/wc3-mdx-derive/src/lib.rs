//! Field-order codecs for fixed-layout MDX structs.
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, Data, DeriveInput, Fields, Index};

/// Reads each struct field in declaration order.
/// Every field must implement `wc3_mdx::Readable`.
#[proc_macro_derive(Readable)]
pub fn derive_readable(input: TokenStream) -> TokenStream {
    expand(parse_macro_input!(input as DeriveInput), true).into()
}

/// Writes each struct field in declaration order from a shared reference.
/// Every field must implement `wc3_mdx::Writable`.
#[proc_macro_derive(Writable)]
pub fn derive_writable(input: TokenStream) -> TokenStream {
    expand(parse_macro_input!(input as DeriveInput), false).into()
}

fn expand(input: DeriveInput, reading: bool) -> proc_macro2::TokenStream {
    let name = input.ident;
    let fields = match input.data {
        Data::Struct(data) => data.fields,
        _ => {
            return syn::Error::new_spanned(name, "codec derives support structs only")
                .to_compile_error()
        }
    };
    let mut generics = input.generics;
    for field in &fields {
        let ty = &field.ty;
        let predicate = if reading {
            parse_quote!(#ty: ::wc3_mdx::Readable)
        } else {
            parse_quote!(#ty: ::wc3_mdx::Writable)
        };
        generics.make_where_clause().predicates.push(predicate);
    }
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let (constructor, writes) = match fields {
        Fields::Named(fields) => {
            let names: Vec<_> = fields
                .named
                .iter()
                .map(|field| field.ident.as_ref().unwrap())
                .collect();
            (
                quote!(Self { #(#names: cursor.read()?,)* }),
                quote!(#(encoder.write(&self.#names)?;)*),
            )
        }
        Fields::Unnamed(fields) => {
            let indices: Vec<_> = (0..fields.unnamed.len()).map(Index::from).collect();
            let reads: Vec<_> = indices.iter().map(|_| quote!(cursor.read()?)).collect();
            (
                quote!(Self( #(#reads,)* )),
                quote!(#(encoder.write(&self.#indices)?;)*),
            )
        }
        Fields::Unit => (quote!(Self), quote!()),
    };
    if reading {
        quote! {
            impl #impl_generics ::wc3_mdx::Readable for #name #ty_generics #where_clause {
                fn read_from(cursor: &mut ::wc3_mdx::Cursor<'_>) -> Result<Self, ::wc3_mdx::DecodeError> {
                    Ok(#constructor)
                }
            }
        }
    } else {
        quote! {
            impl #impl_generics ::wc3_mdx::Writable for #name #ty_generics #where_clause {
                fn write_to(&self, encoder: &mut ::wc3_mdx::Encoder<'_>) -> Result<(), ::wc3_mdx::EncodeError> {
                    #writes
                    Ok(())
                }
            }
        }
    }
}
