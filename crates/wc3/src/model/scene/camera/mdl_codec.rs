//! Camera channels use canonical order and a derived nested Target field group.
use super::{Camera, CameraTrack, CameraVariant};
use crate::model::animation::{AnimationTrack, CameraTargetTranslation};
use crate::model::{mdl, ModelVersion, Vec3};
use mdl::{Dialect, Span, Writer};
use std::io::Write as IoWrite;

#[derive(Default, mdl::Read, mdl::Write)]
#[mdl(fields)]
pub(super) struct Target {
    #[mdl(property = "Position", default)]
    position: Vec3,
    #[mdl(repeated = "Translation", unique_by = "target_key")]
    tracks: Vec<AnimationTrack<CameraTargetTranslation>>,
}
fn target_key(_: &AnimationTrack<CameraTargetTranslation>) -> u8 {
    0
}
fn order(track: &CameraTrack) -> u8 {
    match track {
        CameraTrack::Translation(_) => 0,
        CameraTrack::Rotation(_) => 1,
        CameraTrack::FocusDistance(_) => 2,
        CameraTrack::FocalLength(_) => 3,
        CameraTrack::FStop(_) => 4,
        CameraTrack::TargetTranslation(_) => 5,
        CameraTrack::Visibility(_) => 6,
    }
}
pub(super) struct Channels<'a> {
    tracks: &'a [CameraTrack],
    start: u8,
    end: u8,
}
impl Channels<'_> {
    pub(super) fn iter(&self) -> impl Iterator<Item = &CameraTrack> {
        self.tracks
            .iter()
            .filter(|track| (self.start..=self.end).contains(&order(track)))
    }
}
pub(super) struct TargetRef<'a> {
    position: Vec3,
    tracks: &'a [CameraTrack],
}
impl TargetRef<'_> {
    fn fields(&self) -> Target {
        Target {
            position: self.position,
            tracks: Vec::new(),
        }
    }
}
impl mdl::WriteFields for TargetRef<'_> {
    type State = <Target as mdl::WriteFields>::State;
    fn prepare_mdl_fields(&self, dialect: Dialect) -> Result<Self::State, mdl::WriteError> {
        if self.tracks.iter().filter(|track| order(track) == 5).count() > 1 {
            return Err(mdl::WriteError::Unsupported(
                "duplicate camera target translation",
            ));
        }
        self.fields().prepare_mdl_fields(dialect)
    }
    fn visit_mdl_names(visitor: &mut dyn FnMut(&'static str, bool)) {
        <Target as mdl::WriteFields>::visit_mdl_names(visitor);
    }
    fn write_mdl_headers<W: IoWrite>(&self, writer: &mut Writer<W>) -> Result<(), mdl::WriteError> {
        self.fields().write_mdl_headers(writer)
    }
    fn write_mdl_fields<W: IoWrite>(
        &self,
        state: Self::State,
        writer: &mut Writer<W>,
    ) -> Result<(), mdl::WriteError> {
        self.fields().write_mdl_fields(state, writer)?;
        for track in self.tracks {
            if let CameraTrack::TargetTranslation(track) = track {
                writer.write(track)?;
            }
        }
        Ok(())
    }
}
impl<V: ModelVersion> Camera<V> {
    pub(super) fn mdl_variant() -> CameraVariant {
        V::DEFAULT_VARIANT
    }
    pub(super) fn mdl_transforms(&self) -> Channels<'_> {
        Channels {
            tracks: &self.tracks,
            start: 0,
            end: 1,
        }
    }
    pub(super) fn mdl_lens(&self) -> Channels<'_> {
        Channels {
            tracks: &self.tracks,
            start: 2,
            end: 4,
        }
    }
    pub(super) fn mdl_visibility(&self) -> Channels<'_> {
        Channels {
            tracks: &self.tracks,
            start: 6,
            end: 6,
        }
    }
    fn append_mdl(&mut self, mut value: Vec<CameraTrack>) {
        value.sort_by_key(order);
        self.tracks.extend(value);
    }
    pub(super) fn set_mdl_transforms(
        &mut self,
        value: Vec<CameraTrack>,
        _: bool,
        _: Span,
    ) -> Result<(), mdl::ReadError> {
        self.append_mdl(value);
        Ok(())
    }
    pub(super) fn set_mdl_lens(
        &mut self,
        value: Vec<CameraTrack>,
        _: bool,
        _: Span,
    ) -> Result<(), mdl::ReadError> {
        self.append_mdl(value);
        Ok(())
    }
    pub(super) fn set_mdl_visibility(
        &mut self,
        value: Vec<CameraTrack>,
        _: bool,
        _: Span,
    ) -> Result<(), mdl::ReadError> {
        self.append_mdl(value);
        Ok(())
    }
    pub(super) fn mdl_target(&self) -> TargetRef<'_> {
        TargetRef {
            position: self.target_position,
            tracks: &self.tracks,
        }
    }
    pub(super) fn set_mdl_target(
        &mut self,
        value: Target,
        _: bool,
        _: Span,
    ) -> Result<(), mdl::ReadError> {
        self.target_position = value.position;
        self.tracks
            .extend(value.tracks.into_iter().map(CameraTrack::TargetTranslation));
        Ok(())
    }
    pub(super) fn validate_mdl(&self) -> Result<(), mdl::WriteError> {
        if self.variant != V::DEFAULT_VARIANT {
            return Err(mdl::WriteError::Unsupported("nondefault camera variant"));
        }
        if self
            .tracks
            .windows(2)
            .any(|pair| order(&pair[0]) >= order(&pair[1]))
        {
            return Err(mdl::WriteError::Unsupported(
                "duplicate or noncanonical camera track order",
            ));
        }
        Ok(())
    }
}
