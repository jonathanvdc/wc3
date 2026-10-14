//! Layer features retain their local identities and declared shader operations.
use crate::model::conversion::wem::materials::{one, put, rows};
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wem::materials::{Feature, LayerFeature, ShaderLayerId};
use crate::model::wem::{records, RawDocument, Ref};
use std::collections::BTreeMap;
pub(super) fn read(
    document: &RawDocument<'_>,
    reference: Ref<records::targets::MaterialFeature>,
    layers: &[ShaderLayerId],
) -> Result<Vec<LayerFeature>, Error> {
    rows::<records::MaterialFeature>(document, reference.cast())?
        .into_iter()
        .map(|value| {
            let operation = match value.payload {
                records::FeaturePayload::Fresnel(reference) => {
                    let body: records::FresnelFeature = one(document, reference)?;
                    Feature::Fresnel {
                        color: body.color,
                        exponent: body.exponent,
                        range: [body.out_min, body.out_max],
                        team_color: body.team_color,
                    }
                }
                records::FeaturePayload::UvAnimation(reference) => {
                    let body: records::UvAnimationFeature = one(document, reference)?;
                    Feature::UvAnimation {
                        scroll_rate: body.scroll_rate,
                        rotate_rate: body.rotate_rate,
                        scale_rate: body.scale_rate,
                    }
                }
                records::FeaturePayload::LayerShading(reference) => {
                    let body: records::LayerShadingFeature = one(document, reference)?;
                    Feature::Shading {
                        unlit: body.unlit.get(),
                        two_sided: body.two_sided.get(),
                        unfogged: body.unfogged.get(),
                        no_depth_test: body.no_depth_test.get(),
                        no_depth_write: body.no_depth_write.get(),
                    }
                }
                records::FeaturePayload::Count => {
                    return Err(Error::new(
                        "wem.material.feature",
                        "feature count sentinel has no operation",
                    ));
                }
            };
            Ok(LayerFeature {
                id: value.id,
                layer: *layers.get(value.layer as usize).ok_or_else(|| {
                    Error::new(
                        "wem.material.feature.layer",
                        "shader input index exceeds the authored body",
                    )
                })?,
                operation,
            })
        })
        .collect()
}
pub(super) fn write(
    document: &mut RawDocument<'_>,
    features: &[LayerFeature],
    layers: &BTreeMap<ShaderLayerId, u32>,
) -> Result<Ref<records::targets::MaterialFeature>, Error> {
    let rows = features
        .iter()
        .map(|value| {
            let payload = match value.operation {
                Feature::Fresnel {
                    color,
                    exponent,
                    range,
                    team_color,
                } => records::FeaturePayload::Fresnel(put(
                    document,
                    &[records::FresnelFeature {
                        color,
                        exponent,
                        out_min: range[0],
                        out_max: range[1],
                        team_color,
                    }],
                )?),
                Feature::UvAnimation {
                    scroll_rate,
                    rotate_rate,
                    scale_rate,
                } => records::FeaturePayload::UvAnimation(put(
                    document,
                    &[records::UvAnimationFeature {
                        scroll_rate,
                        rotate_rate,
                        scale_rate,
                    }],
                )?),
                Feature::Shading {
                    unlit,
                    two_sided,
                    unfogged,
                    no_depth_test,
                    no_depth_write,
                } => records::FeaturePayload::LayerShading(put(
                    document,
                    &[records::LayerShadingFeature {
                        unlit: unlit.into(),
                        two_sided: two_sided.into(),
                        unfogged: unfogged.into(),
                        no_depth_test: no_depth_test.into(),
                        no_depth_write: no_depth_write.into(),
                    }],
                )?),
            };
            Ok(records::MaterialFeature {
                id: value.id,
                layer: *layers.get(&value.layer).ok_or_else(|| {
                    Error::new(
                        "wem.material.feature.layer",
                        "shader feature selects another surface input",
                    )
                })?,
                payload,
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    put(document, &rows)
}
