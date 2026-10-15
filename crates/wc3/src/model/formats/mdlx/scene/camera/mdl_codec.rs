//! MDL camera target framing.
use super::{Camera, CameraVariant};
use crate::model::formats::mdlx::IoError;
use crate::model::formats::mdlx::{mdl, FormatVersion, Track, Vec3};
use mdl::Span;
use std::io::Write as IoWrite;

#[derive(Clone, Debug, Default, mdl::Read)]
#[mdl(fields)]
pub(super) struct Target<'data> {
    #[mdl(property = "Position", default)]
    pub position: Vec3,
    #[mdl(property = "Translation")]
    pub translation: Option<Track<'data, Vec3>>,
}
impl<'data, V: FormatVersion> Camera<'data, V> {
    pub(super) fn mdl_variant() -> CameraVariant {
        V::DEFAULT_VARIANT
    }
    pub(super) fn mdl_target(&self) -> TargetView<'data, '_> {
        TargetView {
            position: &self.target_position,
            translation: &self.target_translation,
        }
    }
    pub(super) fn set_mdl_target(
        &mut self,
        target: Target<'data>,
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
            return Err(mdl::WriteError::Unrepresentable {
                field: "camera layout variant",
            });
        }
        Ok(())
    }
}

pub(super) struct TargetView<'data, 'a> {
    position: &'a Vec3,
    translation: &'a Option<Track<'data, Vec3>>,
}
impl<'data> mdl::WriteFields for TargetView<'data, '_> {
    type State = ();
    fn visit_mdl_names(visitor: &mut dyn FnMut(&'static str, bool)) {
        visitor("Position", false);
        visitor("Translation", false);
    }
    fn prepare_mdl_fields(&self, _: mdl::Dialect) -> Result<(), mdl::WriteError> {
        Ok(())
    }
    fn write_mdl_headers<W: IoWrite>(
        &self,
        _: &mut mdl::Writer<W>,
    ) -> Result<(), IoError<mdl::WriteError>> {
        Ok(())
    }
    fn write_mdl_fields<W: IoWrite>(
        &self,
        _: (),
        writer: &mut mdl::Writer<W>,
    ) -> Result<(), IoError<mdl::WriteError>> {
        writer.property("Position", self.position)?;
        if let Some(track) = self.translation {
            track.write_mdl_named(writer, "Translation")?;
        }
        Ok(())
    }
}

impl<'data> mdl::WriteFields for Target<'data> {
    type State = ();
    fn visit_mdl_names(visitor: &mut dyn FnMut(&'static str, bool)) {
        <TargetView as mdl::WriteFields>::visit_mdl_names(visitor);
    }
    fn prepare_mdl_fields(&self, _: mdl::Dialect) -> Result<(), mdl::WriteError> {
        Ok(())
    }
    fn write_mdl_headers<W: IoWrite>(
        &self,
        _: &mut mdl::Writer<W>,
    ) -> Result<(), IoError<mdl::WriteError>> {
        Ok(())
    }
    fn write_mdl_fields<W: IoWrite>(
        &self,
        _: (),
        writer: &mut mdl::Writer<W>,
    ) -> Result<(), IoError<mdl::WriteError>> {
        TargetView {
            position: &self.position,
            translation: &self.translation,
        }
        .write_mdl_fields((), writer)
    }
}
