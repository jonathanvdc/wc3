//! Static and animated native field-of-view placement.
use super::*;

/// Selects static or animated field-of-view placement without runtime layout flags.
pub trait CameraConversion: ModelLayout {
    /// Returns the static camera field of view, when present.
    fn static_fov(camera: &native::records::Camera<'_, Self>) -> f32;
    /// Reads an animated field-of-view channel in layouts that include it.
    fn import_fov<'data, D: Dialect<Layout = Self>>(
        source: &Bundle<'data, D>,
        camera: &native::records::Camera<'data, Self>,
        sequence: usize,
        start: u32,
        target: sm::Target,
    ) -> Result<Option<sm::Track>, Error>;
    /// Installs the field of view in its selected native location.
    fn export_fov(
        camera: &mut native::records::Camera<'static, Self>,
        field: (f32, sm::Target),
        model: &sm::Model<'_>,
        starts: &[u32],
        globals: &mut Vec<u32>,
        context: &mut Context,
    ) -> Result<(), Error>;
}

macro_rules! static_cameras{($($v:ty),*)=>{$(impl CameraConversion for $v{fn static_fov(camera:&native::records::Camera<'_,Self>)->f32{camera.field_of_view}fn import_fov<'data,D:Dialect<Layout=Self>>(_: &Bundle<'data,D>,_:&native::records::Camera<'data,Self>,_:usize,_:u32,_:sm::Target)->Result<Option<sm::Track>,Error>{Ok(None)}fn export_fov(camera:&mut native::records::Camera<'static,Self>,field:(f32,sm::Target),model:&sm::Model<'_>,_:&[u32],_:&mut Vec<u32>,context:&mut Context )->Result<(),Error>{let(value,target)=field;if model.clips.iter().any(|clip|clip.tracks.iter().any(|track|track.target==target&&!track.keys.is_empty())){context.unsupported("cameras.fov","target layout carries only static field of view")?;}camera.field_of_view=value;Ok(())}})*};}

static_cameras!(V256, V257, V260, V263, V264);

macro_rules! animated_cameras{($($v:ty),*)=>{$(impl CameraConversion for $v{fn static_fov(_: &native::records::Camera<'_,Self>)->f32{0.}fn import_fov<'data,D:Dialect<Layout=Self>>(source:&Bundle<'data,D>,camera:&native::records::Camera<'data,Self>,sequence:usize,start:u32,target:sm::Target)->Result<Option<sm::Track>,Error>{import_spline(source,&camera.field_of_view_track,sequence,start,target,|v|vec![v])}fn export_fov(camera:&mut native::records::Camera<'static,Self>,field:(f32,sm::Target),model:&sm::Model<'_>,starts:&[u32],globals:&mut Vec<u32>,context:&mut Context )->Result<(),Error>{let(value,target)=field;camera.field_of_view_track=export_spline::<Self,f32>(model,target,starts,globals,context,|v,_,path|v.first().copied().filter(|_|v.len()==1).ok_or_else(||Error::new(path,"field of view needs one component")))?;if camera.field_of_view_track.base.timeline.inline(0).is_none_or(|t|t.is_empty()){let index=globals.iter().position(|v|*v==1000).unwrap_or_else(||{globals.push(1000);globals.len()-1});camera.field_of_view_track.base.global_sequence=index as u16;camera.field_of_view_track.base.timeline=TimelineFactory::build(vec![vec![0]],true);camera.field_of_view_track.values=SplineFactory::build(vec![vec![[value;3]]]);}Ok(())}})*};}

animated_cameras!(V265, V272, V272PreSplit, V273, V274);
