//! Static and animated field-of-view placement in native camera families.
use super::graph::{Graph, GraphBuilder};
use super::*;
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
    fn logical_fov<'a,'data:'a,D:Dialect<Layout=Self>>(_: &'a Bundle<'data,D>,camera:&'a native::records::Camera<'data,Self>,_:FeatureId,_:&[ClipId],_:&[ClockId],builder:&mut GraphBuilder<'a>)->Result<Animated<f32>,Error>{Ok(builder.add_value(camera.field_of_view))}
    fn write_logical_fov(model:&Graph<'_>,value:&Animated<f32>,camera:&mut native::records::Camera<'static,Self>,_:&[u32],_:&mut Vec<u32>,context:&mut Context)->Result<(),Error>{if model.is_animated(*value){context.unsupported("m2.camera.fov","target dialect has only static field of view")?}camera.field_of_view=*model.values().get(*value).expect("validated slot");Ok(())}
})*};}
static_cameras!(V256, V257, V260, V263, V264);
macro_rules! animated_cameras {($($v:ty),*)=>{$(impl CameraConversion for $v {
    fn logical_fov<'a,'data:'a,D:Dialect<Layout=Self>>(source:&'a Bundle<'data,D>,camera:&'a native::records::Camera<'data,Self>,_feature:FeatureId,clips:&[ClipId],clocks:&[ClockId],builder:&mut GraphBuilder<'a>)->Result<Animated<f32>,Error>{let property=builder.add_value(0f32);let tables=sequence_tables(source,&source.key)?;let mut default=0.;for(seq,clip)in graph::animation::sequences(clips){let(start,_)=tables.sequences.get(seq).map_or((0,0),|s|s.timing.window());if let Some((c,g))=graph::animation::spline_curve(source,&camera.field_of_view_track,seq,start)?{default=c.values().get(0).unwrap_or(default);graph::animation::insert(builder,clip,Channel::Scalar(graph::animation::bind(c,g,property.clone(),clocks)?))?}}builder.values_mut().set(property,default)?;Ok(property)}
    fn write_logical_fov(model:&Graph<'_>,value:&Animated<f32>,camera:&mut native::records::Camera<'static,Self>,starts:&[u32],globals:&mut Vec<u32>,context:&mut Context)->Result<(),Error>{camera.field_of_view_track=graph::animation::animated_spline(model,value,starts,globals,context,|v,_|Ok(v))?;Ok(())}
})*};}
animated_cameras!(V265, V272, V272PreSplit, V273, V274);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::conversion::ExportOptions;
    use crate::model::{Clock, Curve, Model, Time, Track};

    #[test]
    fn static_fov_rejects_boolean_timeline_animation() {
        let mut builder = Model::builder("boolean camera source");
        let _node = builder.add_named_node("carrier").unwrap();
        let source = builder.add_value(true);
        let value = builder.values_mut().scalar_from_boolean(source).unwrap();
        builder
            .add_timeline_channel(Channel::Boolean(Track {
                target: source,
                curve: Curve::step([Time::default()], [true]).unwrap(),
                clock: Clock::Timeline,
            }))
            .unwrap();
        let model = builder.model();
        let mut camera = native::records::Camera::<V256>::new();
        let result = V256::write_logical_fov(
            model,
            &value,
            &mut camera,
            &[],
            &mut Vec::new(),
            &mut Context::export(&ExportOptions::default()),
        );
        assert!(result.is_err());
    }
}
