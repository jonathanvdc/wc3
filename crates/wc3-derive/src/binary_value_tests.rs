use super::*;

#[test]
fn scalar_schema_rejects_ambiguous_or_lossy_definitions() {
    for (definition, expected) in [
        (
            quote!(
                #[wmo(value = u8)]
                enum Mode {
                    #[wmo(value = 0)]
                    A,
                    #[wmo(value = 0)]
                    B,
                    #[wmo(unknown)]
                    Unknown(u8),
                }
            ),
            "duplicate enum wire value",
        ),
        (
            quote!(
                #[wmo(value = u8)]
                enum Mode {
                    #[wmo(value = 0)]
                    A,
                }
            ),
            "unknown-value variant",
        ),
        (
            quote!(
                #[wmo(value = u8)]
                enum Mode {
                    #[wmo(unknown)]
                    Unknown(u32),
                }
            ),
            "wire type",
        ),
        (
            quote!(
                #[wmo(value = u8)]
                enum Mode {
                    #[wmo(value = 0)]
                    A(u8),
                    #[wmo(unknown)]
                    Unknown(u8),
                }
            ),
            "unit variants",
        ),
        (
            quote!(
                #[wmo(value = f32)]
                enum Mode {
                    #[wmo(unknown)]
                    Unknown(f32),
                }
            ),
            "integer scalar",
        ),
        (
            quote!(
                #[wmo(value = u8)]
                enum Mode {
                    #[wmo(value = 256)]
                    A,
                    #[wmo(unknown)]
                    Unknown(u8),
                }
            ),
            "integer range",
        ),
        (
            quote!(
                #[wmo(value = u8)]
                enum Mode {
                    #[wmo(value = -1)]
                    A,
                    #[wmo(unknown)]
                    Unknown(u8),
                }
            ),
            "integer range",
        ),
    ] {
        let input = syn::parse2(definition).unwrap();
        let error = match parse(&input, Format::Wmo) {
            Ok(_) => panic!("invalid schema accepted"),
            Err(error) => error,
        };
        assert!(error.to_string().contains(expected), "{error}");
    }
}

#[test]
fn scalar_schema_accepts_signed_and_full_width_codes() {
    let input = syn::parse2(quote!(
        #[wmo(value = i64)]
        enum Mode {
            #[wmo(value = -1)]
            Sentinel,
            #[wmo(value = 0x7fffffffffffffff)]
            Largest,
            #[wmo(unknown)]
            Unknown(i64),
        }
    ))
    .unwrap();
    assert!(parse(&input, Format::Wmo).is_ok());
}
