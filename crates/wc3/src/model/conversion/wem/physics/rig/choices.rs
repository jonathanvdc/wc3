//! Native selectors map to independently defined preparation choices.
use crate::model::semantic::profiles::wem::physics::{
    BeforeActivation, FractureMethod, GrainAxis, InteriorUvs, JointRange, OpenParts, PushMode,
    ReleaseOrder, SliceAlong,
};
pub(super) fn read_joint_range(value: u8) -> JointRange {
    match value {
        0 => JointRange::Stiff,
        1 => JointRange::Normal,
        2 => JointRange::Loose,
        value => JointRange::Unknown(value),
    }
}
pub(super) fn write_joint_range(value: JointRange) -> u8 {
    match value {
        JointRange::Stiff => 0,
        JointRange::Normal => 1,
        JointRange::Loose => 2,
        JointRange::Unknown(value) => value,
    }
}
pub(super) fn read_grain_axis(value: u8) -> GrainAxis {
    match value {
        0 => GrainAxis::None,
        1 => GrainAxis::X,
        2 => GrainAxis::Y,
        3 => GrainAxis::Z,
        value => GrainAxis::Unknown(value),
    }
}
pub(super) fn write_grain_axis(value: GrainAxis) -> u8 {
    match value {
        GrainAxis::None => 0,
        GrainAxis::X => 1,
        GrainAxis::Y => 2,
        GrainAxis::Z => 3,
        GrainAxis::Unknown(value) => value,
    }
}
pub(super) fn read_open_parts(value: u8) -> OpenParts {
    match value {
        0 => OpenParts::ClosedSolid,
        1 => OpenParts::Thicken,
        value => OpenParts::Unknown(value),
    }
}
pub(super) fn write_open_parts(value: OpenParts) -> u8 {
    match value {
        OpenParts::ClosedSolid => 0,
        OpenParts::Thicken => 1,
        OpenParts::Unknown(value) => value,
    }
}
pub(super) fn read_fracture_method(value: u8) -> FractureMethod {
    match value {
        0 => FractureMethod::Fracture,
        1 => FractureMethod::Slices,
        value => FractureMethod::Unknown(value),
    }
}
pub(super) fn write_fracture_method(value: FractureMethod) -> u8 {
    match value {
        FractureMethod::Fracture => 0,
        FractureMethod::Slices => 1,
        FractureMethod::Unknown(value) => value,
    }
}
pub(super) fn read_push_mode(value: u8) -> PushMode {
    match value {
        0 => PushMode::Blast,
        1 => PushMode::Implode,
        2 => PushMode::None,
        value => PushMode::Unknown(value),
    }
}
pub(super) fn write_push_mode(value: PushMode) -> u8 {
    match value {
        PushMode::Blast => 0,
        PushMode::Implode => 1,
        PushMode::None => 2,
        PushMode::Unknown(value) => value,
    }
}
pub(super) fn read_release_order(value: u8) -> ReleaseOrder {
    match value {
        0 => ReleaseOrder::Together,
        1 => ReleaseOrder::BottomUp,
        2 => ReleaseOrder::TopDown,
        3 => ReleaseOrder::Outward,
        value => ReleaseOrder::Unknown(value),
    }
}
pub(super) fn write_release_order(value: ReleaseOrder) -> u8 {
    match value {
        ReleaseOrder::Together => 0,
        ReleaseOrder::BottomUp => 1,
        ReleaseOrder::TopDown => 2,
        ReleaseOrder::Outward => 3,
        ReleaseOrder::Unknown(value) => value,
    }
}
pub(super) fn read_interior_uvs(value: u8) -> InteriorUvs {
    match value {
        0 => InteriorUvs::Tiled,
        1 => InteriorUvs::Square,
        value => InteriorUvs::Unknown(value),
    }
}
pub(super) fn write_interior_uvs(value: InteriorUvs) -> u8 {
    match value {
        InteriorUvs::Tiled => 0,
        InteriorUvs::Square => 1,
        InteriorUvs::Unknown(value) => value,
    }
}
pub(super) fn read_before_activation(value: u8) -> BeforeActivation {
    match value {
        0 => BeforeActivation::Auto,
        1 => BeforeActivation::Whole,
        2 => BeforeActivation::Pieces,
        value => BeforeActivation::Unknown(value),
    }
}
pub(super) fn write_before_activation(value: BeforeActivation) -> u8 {
    match value {
        BeforeActivation::Auto => 0,
        BeforeActivation::Whole => 1,
        BeforeActivation::Pieces => 2,
        BeforeActivation::Unknown(value) => value,
    }
}
pub(super) fn read_slice_along(value: u8) -> SliceAlong {
    match value {
        0 => SliceAlong::X,
        1 => SliceAlong::Y,
        2 => SliceAlong::Z,
        3 => SliceAlong::Direction,
        value => SliceAlong::Unknown(value),
    }
}
pub(super) fn write_slice_along(value: SliceAlong) -> u8 {
    match value {
        SliceAlong::X => 0,
        SliceAlong::Y => 1,
        SliceAlong::Z => 2,
        SliceAlong::Direction => 3,
        SliceAlong::Unknown(value) => value,
    }
}
