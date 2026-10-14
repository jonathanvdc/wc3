use crate::model::semantic::Property;
pub(in crate::model::conversion::wem) enum Bound {
    Scalar(Property<f32>),
    Vector2(Property<[f32; 2]>),
    Vector3(Property<[f32; 3]>),
    Vector4(Property<[f32; 4]>),
    Unsigned(Property<u32>),
}
mod bind;
mod selectors;
pub(in crate::model::conversion::wem) use bind::bind_property;
pub(in crate::model::conversion::wem) use selectors::property_code;
