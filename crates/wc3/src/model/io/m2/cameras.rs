//! Static and animated field-of-view placement in native camera families.
use super::graph::{Graph, GraphBuilder};
use super::*;
use crate::model::profiles::m2 as profile;
use crate::model::{Animated, Channel, ClipId, ClockId, FeatureId};

/// Selects field-of-view representation through the native dialect's field types.
pub trait CameraConversion: ModelLayout {
    /// Imports a logical field-of-view property, borrowing its native curves.
    fn logical_fov<'a, 'data: 'a, D: Dialect<Layout = Self>>(
        source: &'a Bundle<'data, D>,
        camera: &'a native::records::Camera<'data, Self>,
        feature: FeatureId,
        clips: &[ClipId],
        clocks: &[ClockId],
        builder: &mut GraphBuilder<'a>,
    ) -> Result<Animated<f32>, Error>
    where
        Self::Timing: SequenceClock;
    /// Writes a logical property into the native dialect-selected field.
    fn write_logical_fov(
        model: &Graph<'_>,
        value: &Animated<f32>,
        camera: &mut native::records::Camera<'static, Self>,
        starts: &[u32],
        globals: &mut Vec<u32>,
        context: &mut Context,
    ) -> Result<(), Error>
    where
        Self::Timeline<'static>: TimelineFactory,
        Self::TrackValues<'static, f32>: ValuesFactory<f32>,
        Self::SplineValues<'static, f32>: SplineFactory<f32>;
}
macro_rules! static_cameras {($($v:ty),*)=>{$(impl CameraConversion for $v {
    fn logical_fov<'a,'data:'a,D:Dialect<Layout=Self>>(_: &'a Bundle<'data,D>,camera:&'a native::records::Camera<'data,Self>,_:FeatureId,_:&[ClipId],_:&[ClockId],_:&mut GraphBuilder<'a>)->Result<Animated<f32>,Error>{Ok(Animated::constant(camera.field_of_view))}
    fn write_logical_fov(model:&Graph<'_>,value:&Animated<f32>,camera:&mut native::records::Camera<'static,Self>,_:&[u32],_:&mut Vec<u32>,context:&mut Context)->Result<(),Error>{if value.property.as_ref().is_some_and(|target|model.clips().any(|(_,clip)|clip.channels().iter().any(|c|matches!(c,Channel::Scalar(t)if t.target==*target)))){context.unsupported("m2.camera.fov","target dialect has only static field of view")?}camera.field_of_view=value.default;Ok(())}
})*};}
static_cameras!(V256, V257, V260, V263, V264);
macro_rules! animated_cameras {($($v:ty),*)=>{$(impl CameraConversion for $v {
    fn logical_fov<'a,'data:'a,D:Dialect<Layout=Self>>(source:&'a Bundle<'data,D>,camera:&'a native::records::Camera<'data,Self>,feature:FeatureId,clips:&[ClipId],clocks:&[ClockId],builder:&mut GraphBuilder<'a>)->Result<Animated<f32>,Error>{let property=profile::properties::camera::FIELD_OF_VIEW.bind(feature);let tables=sequence_tables(source,&source.key)?;let mut default=0.;for(seq,clip)in graph::animation::sequences(clips){let(start,_)=tables.sequences.get(seq).map_or((0,0),|s|s.timing.window());if let Some((c,g))=graph::animation::spline_curve(source,&camera.field_of_view_track,seq,start)?{default=c.values().get(0).unwrap_or(default);graph::animation::insert(builder,clip,Channel::Scalar(graph::animation::bind(c,g,property.clone(),clocks)?))?}}Ok(Animated::new(default,property))}
    fn write_logical_fov(model:&Graph<'_>,value:&Animated<f32>,camera:&mut native::records::Camera<'static,Self>,starts:&[u32],globals:&mut Vec<u32>,context:&mut Context)->Result<(),Error>{camera.field_of_view_track=graph::animation::animated_spline(model,value,starts,globals,context,|v,_|Ok(v))?;Ok(())}
})*};}
animated_cameras!(V265, V272, V272PreSplit, V273, V274);
