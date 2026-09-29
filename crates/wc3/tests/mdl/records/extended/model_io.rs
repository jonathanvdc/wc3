use super::*;

#[test]
fn model_io_supports_all_three_with_popcorn_version_gate() {
    whole_model::<V800>();
    whole_model::<V900>();
    whole_model::<V1000>();
    whole_model::<V1100>();
    whole_model::<V1200>();
    whole_model::<V1300>();
    whole_model::<V1400>();
    whole_model::<V1600>();
    whole_model::<V1800>();
    assert_eq!(
        Model::<V800>::decode_mdl(&format!(
            "Version {{ FormatVersion 800, }} Model \"a\" {{}} {POPCORN}"
        ))
        .unwrap_err()
        .kind,
        mdl::ReadErrorKind::UnsupportedField
    );
}
