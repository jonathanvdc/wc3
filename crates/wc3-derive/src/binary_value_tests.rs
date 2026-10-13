use super::*;

#[test]
fn scalar_schema_rejects_ambiguous_or_lossy_definitions() {
    for (definition, expected) in [
        (
            quote!(
                #[wem(value = u8)]
                enum Mode {
                    #[wem(value = 0)]
                    A,
                    #[wem(value = 0)]
                    B,
                    #[wem(unknown)]
                    Unknown(u8),
                }
            ),
            "duplicate enum wire value",
        ),
        (
            quote!(
                #[wem(value = u8)]
                enum Mode {
                    #[wem(value = 0)]
                    A,
                }
            ),
            "unknown-value variant",
        ),
        (
            quote!(
                #[wem(value = u8)]
                enum Mode {
                    #[wem(unknown)]
                    Unknown(u32),
                }
            ),
            "wire type",
        ),
        (
            quote!(
                #[wem(value = u8)]
                enum Mode {
                    #[wem(value = 0)]
                    A(u8),
                    #[wem(unknown)]
                    Unknown(u8),
                }
            ),
            "unit variants",
        ),
        (
            quote!(
                #[wem(value = f32)]
                enum Mode {
                    #[wem(unknown)]
                    Unknown(f32),
                }
            ),
            "integer scalar",
        ),
        (
            quote!(
                #[wem(value = u8)]
                enum Mode {
                    #[wem(value = 256)]
                    A,
                    #[wem(unknown)]
                    Unknown(u8),
                }
            ),
            "integer range",
        ),
        (
            quote!(
                #[wem(value = u8)]
                enum Mode {
                    #[wem(value = -1)]
                    A,
                    #[wem(unknown)]
                    Unknown(u8),
                }
            ),
            "integer range",
        ),
    ] {
        let input = syn::parse2(definition).unwrap();
        let error = match parse(&input, Format::Wem) {
            Ok(_) => panic!("invalid schema accepted"),
            Err(error) => error,
        };
        assert!(error.to_string().contains(expected), "{error}");
    }
}

#[test]
fn scalar_schema_accepts_signed_and_full_width_codes() {
    let input = syn::parse2(quote!(
        #[wem(value = i64)]
        enum Mode {
            #[wem(value = -1)]
            Sentinel,
            #[wem(value = 0x7fffffffffffffff)]
            Largest,
            #[wem(unknown)]
            Unknown(i64),
        }
    ))
    .unwrap();
    assert!(parse(&input, Format::Wem).is_ok());
}
