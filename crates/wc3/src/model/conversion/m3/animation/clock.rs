//! Signed key clocks retain the selected sequence's native origin.
use super::super::effects::metadata as native_metadata;
use super::*;
pub(super) fn start(raw: &wem::RawDocument<'_>, clip: &wr::Clip<wr::V7>) -> Result<u32, Error> {
    let values = native_metadata::read(raw, &clip.native)?;
    Ok(values.get("m3.sequence.start_frame").copied().unwrap_or(0) as u32)
}
pub(super) fn frame(
    time: f32,
    start: u32,
    context: &mut Context,
    path: &str,
) -> Result<nr::I32, Error> {
    if !time.is_finite() {
        return Err(Error::new(path, "non-finite animation time"));
    }
    let relative = (f64::from(time) * 1000.).round();
    let absolute = relative + f64::from(start);
    if absolute < f64::from(i32::MIN) || absolute > f64::from(i32::MAX) {
        return Err(Error::new(
            path,
            "key exceeds native signed millisecond clock",
        ));
    }
    context.approximate(
        path,
        (relative as f32 / 1000. - time).abs(),
        "animation time quantized to milliseconds",
    )?;
    Ok(nr::I32 {
        value: absolute as i32,
    })
}
