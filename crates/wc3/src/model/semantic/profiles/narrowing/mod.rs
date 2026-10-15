//! Exact checked projections of complete owner payloads.
use super::Profile;
use crate::model::semantic::{Error, PropertyPayload};
mod m2;
mod m3;
mod mdlx;
mod wmo;
/// A target vocabulary that can check and consume payloads from another profile.
/// Implementations must preserve every accepted authored value and relationship.
/// Returning an error is required when an exact projection is unavailable.
pub trait Narrows<Source: Profile>: Profile {
    /// Projects structural selectors when their exact vocabulary is supported.
    fn property(value: Source::Property) -> Result<Self::Property, Error> {
        Self::Property::from_identity(&value.identity().to_owned())
            .ok_or_else(|| incompatible("animation property"))
    }

    /// Projects asset-wide authoring controls without changing their meaning.
    fn asset<'a>(value: Source::Asset<'a>) -> Result<Self::Asset<'a>, Error>;
    /// Projects the complete appearance vocabulary while preserving identities.
    fn appearance<'a>(value: Source::Appearance<'a>) -> Result<Self::Appearance<'a>, Error>;
    /// Projects the authored joint binding without changing its interpretation.
    fn binding<'a>(value: Source::Binding<'a>) -> Result<Self::Binding<'a>, Error>;
    /// Projects complete geometry without changing its logical values.
    fn geometry<'a>(value: Source::Geometry<'a>) -> Result<Self::Geometry<'a>, Error>;
    /// Projects model-wide controls without changing their meaning.
    fn model<'a>(value: Source::Model<'a>) -> Result<Self::Model<'a>, Error>;
    /// Projects an independent scene system without changing its meaning.
    fn system<'a>(value: Source::System<'a>) -> Result<Self::System<'a>, Error>;
    /// Projects a surface without changing its meaning.
    fn surface<'a>(value: Source::Surface<'a>) -> Result<Self::Surface<'a>, Error>;
    /// Projects an attached feature without changing its meaning.
    fn feature<'a>(value: Source::Feature<'a>) -> Result<Self::Feature<'a>, Error>;
    /// Projects node controls without changing meaning.
    fn node<'a>(value: Source::Node<'a>) -> Result<Self::Node<'a>, Error>;
    /// Projects texture controls without changing meaning.
    fn texture<'a>(value: Source::Texture<'a>) -> Result<Self::Texture<'a>, Error>;
    /// Projects mesh controls without changing meaning.
    fn mesh<'a>(value: Source::Mesh<'a>) -> Result<Self::Mesh<'a>, Error>;
    /// Projects instance controls without changing meaning.
    fn instance<'a>(value: Source::Instance<'a>) -> Result<Self::Instance<'a>, Error>;
    /// Projects draw-pass behavior without changing meaning.
    fn draw(value: Source::Draw) -> Result<Self::Draw, Error>;
    /// Projects clip controls without changing meaning.
    fn clip<'a>(value: Source::Clip<'a>) -> Result<Self::Clip<'a>, Error>;
    /// Projects controller controls without changing meaning.
    fn controller<'a>(value: Source::Controller<'a>) -> Result<Self::Controller<'a>, Error>;
}
impl<P: Profile> Narrows<P> for P {
    fn property(value: P::Property) -> Result<Self::Property, Error> {
        Ok(value)
    }
    fn asset<'a>(value: P::Asset<'a>) -> Result<Self::Asset<'a>, Error> {
        Ok(value)
    }
    fn appearance<'a>(value: P::Appearance<'a>) -> Result<Self::Appearance<'a>, Error> {
        Ok(value)
    }
    fn binding<'a>(value: P::Binding<'a>) -> Result<Self::Binding<'a>, Error> {
        Ok(value)
    }
    fn geometry<'a>(value: P::Geometry<'a>) -> Result<Self::Geometry<'a>, Error> {
        Ok(value)
    }
    fn model<'a>(value: P::Model<'a>) -> Result<Self::Model<'a>, Error> {
        Ok(value)
    }
    fn system<'a>(value: P::System<'a>) -> Result<Self::System<'a>, Error> {
        Ok(value)
    }
    fn surface<'a>(value: P::Surface<'a>) -> Result<Self::Surface<'a>, Error> {
        Ok(value)
    }
    fn feature<'a>(value: P::Feature<'a>) -> Result<Self::Feature<'a>, Error> {
        Ok(value)
    }
    fn node<'a>(value: P::Node<'a>) -> Result<Self::Node<'a>, Error> {
        Ok(value)
    }
    fn texture<'a>(value: P::Texture<'a>) -> Result<Self::Texture<'a>, Error> {
        Ok(value)
    }
    fn mesh<'a>(value: P::Mesh<'a>) -> Result<Self::Mesh<'a>, Error> {
        Ok(value)
    }
    fn instance<'a>(value: P::Instance<'a>) -> Result<Self::Instance<'a>, Error> {
        Ok(value)
    }
    fn draw(value: P::Draw) -> Result<Self::Draw, Error> {
        Ok(value)
    }
    fn clip<'a>(value: P::Clip<'a>) -> Result<Self::Clip<'a>, Error> {
        Ok(value)
    }
    fn controller<'a>(value: P::Controller<'a>) -> Result<Self::Controller<'a>, Error> {
        Ok(value)
    }
}
fn incompatible(owner: &str) -> Error {
    Error::new(
        "profile.narrow",
        format!("{owner} payload is absent or belongs to an incompatible vocabulary"),
    )
}
