use std::sync::atomic::{AtomicUsize, Ordering};
use wc3::model::animation::{AnimationTrack, GeosetAlpha, GeosetColor, GeosetTrack};
use wc3::model::mdl::{self, Read as _, ValueEq as _, Write as _, Writer};

fn refuses_before_output<T: mdl::Write>(value: &T) {
    let mut writer = Writer::new(Vec::new());
    assert!(writer.write(value).is_err());
    assert!(writer.finish().unwrap().is_empty());
}

// No record-specific validation hook: the derive must protect omitted bases.
#[derive(mdl::Read, mdl::Write)]
#[mdl(block = "ZeroBase")]
struct ZeroBase {
    #[mdl(animatable = "Alpha", track = "GeosetTrack::Alpha", default)]
    alpha: f32,
    #[mdl(animatable = "Color", track = "GeosetTrack::Color", default)]
    color: [f32; 3],
    #[mdl(tracks)]
    tracks: Vec<GeosetTrack>,
}

#[test]
fn linked_omitted_bases_preserve_signed_zero_and_vector_components() {
    let mut value = ZeroBase {
        alpha: 0.0,
        color: [0.0; 3],
        tracks: vec![AnimationTrack::<GeosetAlpha>::linear(Vec::new(), None)
            .unwrap()
            .into()],
    };
    assert!(value.encode_mdl().is_ok());
    value.alpha = -0.0;
    refuses_before_output(&value);
    value.tracks.clear();
    let decoded = ZeroBase::decode_mdl(&value.encode_mdl().unwrap()).unwrap();
    assert_eq!(decoded.alpha.to_bits(), (-0.0f32).to_bits());
    value.alpha = 0.0;
    value.tracks.push(
        AnimationTrack::<GeosetColor>::linear(Vec::new(), None)
            .unwrap()
            .into(),
    );
    value.color[1] = -0.0;
    refuses_before_output(&value);
    value.color = [0.0; 3];
    assert!(value.encode_mdl().is_ok());
    value.color[2] = 0.5;
    refuses_before_output(&value);
}

#[derive(mdl::Read, mdl::Write)]
#[mdl(block = "NanBase", default)]
struct NanBase {
    #[mdl(animatable = "Alpha", track = "GeosetTrack::Alpha")]
    alpha: f32,
    #[mdl(tracks)]
    tracks: Vec<GeosetTrack>,
}
impl Default for NanBase {
    fn default() -> Self {
        Self {
            alpha: f32::NAN,
            tracks: Vec::new(),
        }
    }
}

#[test]
fn omitted_nan_defaults_compare_by_class() {
    let mut value = NanBase {
        alpha: f32::from_bits(0xffc12345),
        tracks: vec![AnimationTrack::<GeosetAlpha>::linear(Vec::new(), None)
            .unwrap()
            .into()],
    };
    let output = value.encode_mdl().unwrap();
    assert!(!output.contains("static Alpha"));
    assert!(NanBase::decode_mdl(&output).unwrap().alpha.is_nan());
    for alpha in [0.0, f32::INFINITY, f32::NEG_INFINITY] {
        value.alpha = alpha;
        refuses_before_output(&value);
    }
    assert!(f32::NAN.eq_mdl(&f32::from_bits(0xffc12345)));
    assert!(!f32::INFINITY.eq_mdl(&f32::NEG_INFINITY));
    assert!([f32::NAN, -0.0].eq_mdl(&[f32::from_bits(0x7fc12345), -0.0]));
    assert!(![f32::NAN, -0.0].eq_mdl(&[f32::NAN, 0.0]));
}

fn always_skip<T>(_: &T) -> bool {
    true
}
#[derive(mdl::Read, mdl::Write)]
#[mdl(block = "Suppressed")]
struct Suppressed<T> {
    #[mdl(static_property = "Value", default, skip_if = "always_skip")]
    value: T,
}

#[test]
fn predicates_cannot_discard_nondefault_values() {
    assert_eq!(
        Suppressed { value: 0.0f32 }.encode_mdl().unwrap(),
        "Suppressed {\n}\n"
    );
    refuses_before_output(&Suppressed { value: -0.0f32 });
    refuses_before_output(&Suppressed { value: 1u32 });
    refuses_before_output(&Suppressed {
        value: [0.0f32, 1.0],
    });
}

static PREDICATE_CALLS: AtomicUsize = AtomicUsize::new(0);
fn counted_skip(value: &f32) -> bool {
    PREDICATE_CALLS.fetch_add(1, Ordering::Relaxed);
    value.to_bits() == 0
}
#[derive(mdl::Read, mdl::Write)]
#[mdl(block = "CountedPredicate")]
struct CountedPredicate {
    #[mdl(property = "Value", default, skip_if = "counted_skip")]
    value: f32,
}

#[test]
fn omission_decision_is_reused_during_output() {
    PREDICATE_CALLS.store(0, Ordering::Relaxed);
    assert!(CountedPredicate { value: 2.0 }
        .encode_mdl()
        .unwrap()
        .contains("Value 2.0,"));
    assert_eq!(PREDICATE_CALLS.load(Ordering::Relaxed), 1);
    CountedPredicate { value: 0.0 }.encode_mdl().unwrap();
    assert_eq!(PREDICATE_CALLS.load(Ordering::Relaxed), 2);
}
