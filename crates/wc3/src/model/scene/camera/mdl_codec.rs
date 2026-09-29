//! MDL camera target framing.
use super::{Camera, CameraVariant};
use crate::model::{mdl, ModelVersion, Track, Vec3};
use mdl::Span;
use std::io::Write as IoWrite;

#[derive(Clone, Debug, Default, mdl::Read)]
#[mdl(fields)]
pub(super) struct Target {
    #[mdl(property = "Position", default)]
    pub position: Vec3,
    #[mdl(property = "Translation")]
    pub translation: Option<Track<Vec3>>,
}
impl<V: ModelVersion> Camera<V> {
    pub(super) fn mdl_variant() -> CameraVariant {
        V::DEFAULT_VARIANT
    }
    pub(super) fn mdl_target(&self) -> TargetView<'_> {
        TargetView {
            position: &self.target_position,
            translation: &self.target_translation,
        }
    }
    pub(super) fn set_mdl_target(
        &mut self,
        target: Target,
        _: bool,
        _: Span,
    ) -> Result<(), mdl::ReadError> {
        self.target_position = target.position;
        self.target_translation = target.translation;
        Ok(())
    }
    pub(super) fn validate_mdl(&self) -> Result<(), mdl::WriteError> {
        if !matches!(
            self.variant,
            CameraVariant::Variant0 | CameraVariant::Variant3
        ) {
            return Err(mdl::WriteError::Unsupported("camera layout variant"));
        }
        Ok(())
    }
}

pub(super) struct TargetView<'a> {
    position: &'a Vec3,
    translation: &'a Option<Track<Vec3>>,
}
impl mdl::WriteFields for TargetView<'_> {
    type State = ();
    fn visit_mdl_names(visitor: &mut dyn FnMut(&'static str, bool)) {
        visitor("Position", false);
        visitor("Translation", false);
    }
    fn prepare_mdl_fields(&self, _: mdl::Dialect) -> Result<(), mdl::WriteError> {
        Ok(())
    }
    fn write_mdl_headers<W: IoWrite>(&self, _: &mut mdl::Writer<W>) -> Result<(), mdl::WriteError> {
        Ok(())
    }
    fn write_mdl_fields<W: IoWrite>(
        &self,
        _: (),
        writer: &mut mdl::Writer<W>,
    ) -> Result<(), mdl::WriteError> {
        writer.property("Position", self.position)?;
        if let Some(track) = self.translation {
            track.write_mdl_named(writer, "Translation")?;
        }
        Ok(())
    }
}

impl mdl::WriteFields for Target {
    type State = ();
    fn visit_mdl_names(visitor: &mut dyn FnMut(&'static str, bool)) {
        <TargetView as mdl::WriteFields>::visit_mdl_names(visitor);
    }
    fn prepare_mdl_fields(&self, _: mdl::Dialect) -> Result<(), mdl::WriteError> {
        Ok(())
    }
    fn write_mdl_headers<W: IoWrite>(&self, _: &mut mdl::Writer<W>) -> Result<(), mdl::WriteError> {
        Ok(())
    }
    fn write_mdl_fields<W: IoWrite>(
        &self,
        _: (),
        writer: &mut mdl::Writer<W>,
    ) -> Result<(), mdl::WriteError> {
        TargetView {
            position: &self.position,
            translation: &self.translation,
        }
        .write_mdl_fields((), writer)
    }
}
