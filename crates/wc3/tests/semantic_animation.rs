use wc3::model::common::ZeroVec;
use wc3::model::profiles::mdlx::properties::{light, Layer, LightField, Target as MdlxTarget};
use wc3::model::profiles::{Mdlx, Universal};
use wc3::model::{
    BezierKeyframe, ContinuousDomain, Curve, CurveValue, HermiteKeyframe, Interpolation, Material,
    Model, OwnerId, PropertyTarget, Quaternion, TcbKeyframe, TcbParameters, Time, ValueKeyframe,
};
use wc3::model::{Construct as SlotConstruct, PropertyValues as SlotValues};
use zerovec::ule::AsULE;

fn time(seconds: f64) -> Time {
    Time::seconds(seconds).unwrap()
}

#[test]
fn separate_spline_arrays_retain_encoded_borrowed_storage() {
    let times = [time(-1.), time(0.), time(0.), time(1.)];
    let values = [0f32, 1., 2., 3.];
    let incoming = [0f32; 4];
    let outgoing = [1f32; 4];
    let encoded_times = ZeroVec::alloc_from_slice(&times);
    let encoded_values = ZeroVec::alloc_from_slice(&values);
    let encoded_incoming = ZeroVec::alloc_from_slice(&incoming);
    let encoded_outgoing = ZeroVec::alloc_from_slice(&outgoing);
    let curve = Curve::hermite(
        encoded_times.as_slice().as_zerovec(),
        encoded_values.as_slice().as_zerovec(),
        encoded_incoming.as_slice().as_zerovec(),
        encoded_outgoing.as_slice().as_zerovec(),
    )
    .unwrap();
    assert!(!curve.values().is_owned());
    assert_eq!(
        curve.values().as_bytes().as_ptr(),
        encoded_values.as_bytes().as_ptr()
    );
    assert_eq!(curve.times().to_vec(), times);
    assert_eq!(curve.values().to_vec(), values);
    let derivatives = curve.hermite_derivatives().unwrap();
    assert_eq!(derivatives.incoming.to_vec(), incoming);
    assert_eq!(derivatives.outgoing.to_vec(), outgoing);
    assert!(curve.bezier_controls().is_none());
    assert!(curve.tcb_parameters().is_none());
    // Duplicate keys keep both sides of the discontinuity in authored order.
    assert_eq!(curve.values().to_vec(), values);
    assert_eq!(curve.times().to_vec(), times);
}

#[test]
fn typed_keyframes_materialize_separate_encoded_arrays() {
    let keys = [HermiteKeyframe {
        time: time(0.),
        value: Quaternion::IDENTITY,
        incoming: [0.; 4],
        outgoing: [0.; 4],
    }];
    let curve = Curve::hermite_keys(keys).unwrap();
    assert!(curve.values().is_owned());
    assert_eq!(curve.values().get(0), Some(Quaternion::IDENTITY));
    assert_eq!(
        curve.hermite_derivatives().unwrap().incoming.get(0),
        Some([0.; 4])
    );
    assert!(Curve::step([time(0.)], [Quaternion([0.; 4])]).is_err());
    assert!(Curve::hermite(
        [time(0.)],
        [Quaternion::IDENTITY],
        [[f32::NAN; 4]],
        [[0.; 4]]
    )
    .is_err());
}

#[test]
fn bezier_controls_and_tcb_parameters_keep_their_authored_domains() {
    let curve = Curve::bezier_keys([BezierKeyframe {
        time: time(0.),
        value: Quaternion::IDENTITY,
        incoming: [0.; 4],
        outgoing: [0., 0., 0., 2.],
    }])
    .unwrap();
    assert_eq!(curve.interpolation(), Interpolation::Bezier);
    assert!(curve.hermite_derivatives().is_none());
    assert_eq!(
        curve.bezier_controls().unwrap().outgoing.get(0),
        Some([0., 0., 0., 2.])
    );
    let parameters = TcbParameters {
        tension: 0.1,
        continuity: -0.2,
        bias: 0.3,
    };
    let curve = Curve::tcb_keys([TcbKeyframe {
        time: time(0.),
        value: 1f32,
        parameters,
    }])
    .unwrap();
    assert_eq!(curve.interpolation(), Interpolation::Tcb);
    assert_eq!(curve.tcb_parameters().unwrap().get(0), Some(parameters));
    assert!(curve.bezier_controls().is_none());
}

#[test]
fn constructors_check_order_cardinality_and_finite_controls() {
    let mut slot_values = SlotValues::default();
    assert!(Curve::linear([time(1.), time(0.)], [0f32, 1.]).is_err());
    assert!(Curve::step([time(0.)], Vec::<bool>::new()).is_err());
    assert!(Curve::hermite([time(0.)], [1f32], Vec::<f32>::new(), [0.]).is_err());
    assert!(Curve::bezier([time(0.)], [1f32], [0.], [f32::INFINITY]).is_err());
    assert!(Curve::tcb(
        [time(0.)],
        [1f32],
        [TcbParameters {
            tension: f32::NAN,
            ..SlotConstruct::construct(&mut slot_values)
        }]
    )
    .is_err());
    assert!(Curve::<f32>::linear(ZeroVec::default(), ZeroVec::default()).is_ok());
    assert!(Time::seconds(f64::INFINITY).is_err());
}

#[test]
fn spherical_keyframes_and_discrete_keys_select_their_shapes() {
    let keys = [ValueKeyframe {
        time: time(0.),
        value: Quaternion::IDENTITY,
    }];
    assert_eq!(
        Curve::slerp_keys(keys.clone()).unwrap().interpolation(),
        Interpolation::Slerp
    );
    assert_eq!(
        Curve::nlerp_keys(keys).unwrap().interpolation(),
        Interpolation::Nlerp
    );
    let curve = Curve::step_keys([ValueKeyframe {
        time: time(0.),
        value: true,
    }])
    .unwrap();
    assert_eq!(curve.interpolation(), Interpolation::Step);
    assert_eq!(curve.values().get(0), Some(true));
}

#[test]
fn descriptors_preserve_structural_fields_and_typed_owner_identities() {
    let mut slot_values = SlotValues::default();
    let mut builder = Model::<Universal>::builder("properties");
    let node = builder.add_named_node("light").unwrap();
    let feature = builder.reserve_feature(node);
    let intensity = light::INTENSITY.bind(feature);
    assert_eq!(
        intensity.target(),
        &PropertyTarget::<Mdlx>::Profile {
            owner: OwnerId::Feature(feature),
            key: MdlxTarget::Light(LightField::Intensity),
        }
    );
    let material = builder
        .add_material(Material {
            name: "".into(),
            surface: SlotConstruct::construct(&mut slot_values),
        })
        .unwrap();
    let texture = Layer(2).texture_slot(3).bind(material);
    assert_eq!(
        texture.target(),
        &PropertyTarget::<Mdlx>::Profile {
            owner: OwnerId::Material(material),
            key: MdlxTarget::LayerTexture { layer: 2, slot: 3 },
        }
    );
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Probability(f32);
impl AsULE for Probability {
    type ULE = <f32 as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        self.0.to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        Self(f32::from_unaligned(value))
    }
}
impl CurveValue for Probability {
    type Domain = ContinuousDomain;
    type Derivative = f32;
    type ControlPoint = f32;
    fn is_valid(&self) -> bool {
        (0. ..=1.).contains(&self.0)
    }
    fn into_derivative(self) -> f32 {
        self.0
    }
    fn into_control_point(self) -> f32 {
        self.0
    }
    fn derivative_value(value: f32) -> Self {
        Self(value)
    }
    fn control_point_value(value: f32) -> Self {
        Self(value)
    }
}

#[test]
fn custom_domains_derive_capabilities_and_keep_controls_outside_the_key_domain() {
    let curve = Curve::hermite([time(0.)], [Probability(0.5)], [-2f32], [3f32]).unwrap();
    assert_eq!(
        curve.hermite_derivatives().unwrap().incoming.get(0),
        Some(-2.)
    );
    assert_eq!(curve.values().get(0), Some(Probability(0.5)));
    assert!(Curve::linear([time(0.)], [Probability(-0.5)]).is_err());
    assert!(Curve::linear([time(0.)], [Probability(0.5)]).is_ok());
}
