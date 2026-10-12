//! Ownership operations for the model's lifetime-bearing record graph.
use proc_macro2::{TokenStream, TokenTree};
use quote::{quote, ToTokens};
use syn::{parse_quote, Data, DeriveInput, Fields, Index, Type};

pub(crate) fn view_type(ty: &Type, lifetime: &str) -> Type {
    syn::parse_str(
        &ty.to_token_stream()
            .to_string()
            .replace("'data", lifetime)
            .replace("' data", lifetime),
    )
    .expect("record type")
}
fn field_value(ty: &Type, value: TokenStream, borrowing: bool) -> TokenStream {
    let text = ty.to_token_stream().to_string();
    if text.contains("'data") || text.contains("' data") {
        if borrowing {
            quote!(::wc3::model::common::Borrowed::as_borrowed(#value))
        } else {
            quote!(::wc3::model::common::Detached::into_detached(#value))
        }
    } else if borrowing {
        quote!(Clone::clone(#value))
    } else {
        value
    }
}
pub(crate) fn expand(input: DeriveInput) -> TokenStream {
    if matches!(input.data, Data::Union(_)) {
        return syn::Error::new_spanned(&input.ident, "ownership cannot be derived for a union")
            .to_compile_error();
    }
    let name = &input.ident;
    let (_, ty_generics, _) = input.generics.split_for_impl();
    let ty: Type = parse_quote!(#name #ty_generics);
    let borrowed_ty = view_type(&ty, "'view");
    let owned_ty = view_type(&ty, "'owned");
    let mut borrowed_generics = input.generics.clone();
    borrowed_generics.params.insert(0, parse_quote!('view));

    let mut owned_generics = input.generics.clone();
    owned_generics.params.insert(0, parse_quote!('owned));
    // All dialect and layout parameters are preserved when detaching buffers.
    for p in input.generics.type_params() {
        let ident = &p.ident;
        borrowed_generics
            .make_where_clause()
            .predicates
            .push(parse_quote!(#ident: Clone));
        owned_generics
            .make_where_clause()
            .predicates
            .push(parse_quote!(#ident: 'owned));
    }
    let construct = |fields: &Fields, borrowing: bool, prefix: TokenStream| {
        let mut names = Vec::new();
        let mut values = Vec::new();
        for (index, field) in fields.iter().enumerate() {
            let member = field.ident.as_ref().map(|n| quote!(#n)).unwrap_or_else(|| {
                let n = Index::from(index);
                quote!(#n)
            });
            let source = if borrowing {
                quote!(&self.#member)
            } else {
                quote!(self.#member)
            };
            names.push(member);
            values.push(field_value(&field.ty, source, borrowing));
        }
        match fields {
            Fields::Named(_) => quote!(#prefix { #(#names: #values),* }),
            Fields::Unnamed(_) => quote!(#prefix(#(#values),*)),
            Fields::Unit => prefix,
        }
    };
    let body = |borrowing: bool| match &input.data {
        Data::Struct(data) => construct(&data.fields, borrowing, quote!(#name)),
        Data::Enum(data) => {
            let arms = data.variants.iter().map(|variant| {
                let v = &variant.ident;
                let names: Vec<_> = variant
                    .fields
                    .iter()
                    .enumerate()
                    .map(|(i, f)| {
                        f.ident
                            .clone()
                            .unwrap_or_else(|| syn::Ident::new(&format!("field_{i}"), name.span()))
                    })
                    .collect();
                let values: Vec<_> = variant
                    .fields
                    .iter()
                    .zip(&names)
                    .map(|(f, n)| field_value(&f.ty, quote!(#n), borrowing))
                    .collect();
                match &variant.fields {
                    Fields::Named(_) => {
                        let fields: Vec<_> = names
                            .iter()
                            .zip(&values)
                            .map(|(name, value)| {
                                let mut tokens = value.clone().into_iter();
                            if matches!(tokens.next(), Some(TokenTree::Ident(ident)) if ident == *name) && tokens.next().is_none() {
                                    quote!(#name)
                                } else {
                                    quote!(#name: #value)
                                }
                            })
                            .collect();
                        quote!(Self::#v { #(#names),* } => #name::#v { #(#fields),* })
                    }
                    Fields::Unnamed(_) => quote!(Self::#v(#(#names),*)=>#name::#v(#(#values),*)),
                    Fields::Unit => quote!(Self::#v=>#name::#v),
                }
            });
            quote!(match self { #(#arms),* })
        }
        _ => unreachable!(),
    };
    let borrow_body = body(true);
    let own_body = body(false);
    let (borrow_impl, _, borrow_where) = borrowed_generics.split_for_impl();
    let (own_impl, _, own_where) = owned_generics.split_for_impl();
    let mut methods_generics = input.generics.clone();
    for p in input.generics.type_params() {
        let ident = &p.ident;
        methods_generics
            .make_where_clause()
            .predicates
            .push(parse_quote!(#ident: Clone));
    }
    let (methods_impl, _, methods_where) = methods_generics.split_for_impl();
    let static_ty = view_type(&ty, "'static");
    let static_bounds: Vec<_> = input
        .generics
        .type_params()
        .map(|p| {
            let name = &p.ident;
            quote!(#name:'static)
        })
        .collect();
    quote! {
        impl #methods_impl #ty #methods_where {
            /// Borrows all buffers, including buffers currently owned by this record.
            pub fn as_borrowed<'view>(&'view self)->#borrowed_ty {
                ::wc3::model::common::Borrowed::as_borrowed(self)
            }
            /// Creates an independently owned copy of this record and all of its buffers.
            pub fn to_owned(&self)->#static_ty where #(#static_bounds),* {
                self.as_borrowed().into_owned()
            }
            /// Moves owned buffers and copies borrowed buffers into independent storage.
            pub fn into_owned(self)->#static_ty where #(#static_bounds),* {
                ::wc3::model::common::Detached::into_detached(self)
            }
        }
        impl #borrow_impl ::wc3::model::common::Borrowed<'view> for #ty #borrow_where {
            type View = #borrowed_ty;
            fn as_borrowed(&'view self) -> Self::View { #borrow_body }
        }
        impl #own_impl ::wc3::model::common::Detached<'owned> for #ty #own_where {
            type Detached = #owned_ty;
            fn into_detached(self) -> Self::Detached { #own_body }
        }
    }
}

pub(crate) fn expand_eq(input: DeriveInput) -> TokenStream {
    let name = &input.ident;
    let (_, ty_generics, _) = input.generics.split_for_impl();
    let ty: Type = parse_quote!(#name #ty_generics);
    let other_ty = view_type(&ty, "'other");
    let mut generics = input.generics.clone();
    generics.params.insert(0, parse_quote!('other));
    let (eq_impl, _, eq_where) = generics.split_for_impl();
    let (same_impl, _, same_where) = input.generics.split_for_impl();
    let Data::Struct(data) = &input.data else {
        return syn::Error::new_spanned(name, "model equality requires a struct")
            .to_compile_error();
    };
    let comparisons = data.fields.iter().enumerate().map(|(index, field)| {
        let member = field
            .ident
            .as_ref()
            .map(|name| quote!(#name))
            .unwrap_or_else(|| {
                let n = Index::from(index);
                quote!(#n)
            });
        quote!(self.#member==other.#member)
    });
    quote! {
        impl #same_impl #ty #same_where {
            fn fields_equal(&self,other:&Self)->bool {true #(&& #comparisons)*}
        }
        impl #eq_impl PartialEq<#other_ty> for #ty #eq_where {
            fn eq(&self,other:&#other_ty)->bool {
                self.as_borrowed().fields_equal(&other.as_borrowed())
            }
        }
    }
}
