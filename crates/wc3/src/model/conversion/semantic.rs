//! Ordered graph rewrites followed by an exact target projection.
use super::{Conversion, Error, Issue, IssueKind, Report};
use crate::model::error::Location;
use crate::model::profiles::{Mdlx, Narrows, Universal, M2};
use crate::model::{FeatureId, Light, LightActivation, Model, ModelBuilder, NodeFeature};

/// One explicit transformation used to prepare a graph for a target vocabulary.
///
/// Rules may inspect the current draft through `draft.model()` and edit it using
/// the regular builder API. Record every approximation or discarded meaning in
/// `report`. The completed draft must be a valid universal graph. Exact target
/// representability is checked after all rules have run.
pub trait Rewrite<'a, Q: Narrows<Universal>> {
    /// Applies this rule to an unpublished draft and records its decisions.
    fn apply(&self, draft: &mut ModelBuilder<'a>, report: &mut Report) -> Result<(), Error>;
}
impl<'a, Q, F> Rewrite<'a, Q> for F
where
    Q: Narrows<Universal>,
    F: Fn(&mut ModelBuilder<'a>, &mut Report) -> Result<(), Error>,
{
    fn apply(&self, draft: &mut ModelBuilder<'a>, report: &mut Report) -> Result<(), Error> {
        self(draft, report)
    }
}

/// A caller-assembled, ordered set of transformations for one target vocabulary.
///
/// An empty policy requires exact representability. Conversion clones the source;
/// neither failed rules nor a failed final projection publish any source edits.
/// Rules run once, in insertion order, without implicit fallback or retries.
pub struct Policy<'rules, 'a, Q: Narrows<Universal>> {
    rules: Vec<Box<dyn Rewrite<'a, Q> + 'rules>>,
}
impl<'rules, 'a, Q: Narrows<Universal>> Default for Policy<'rules, 'a, Q> {
    fn default() -> Self {
        Self { rules: Vec::new() }
    }
}
impl<'rules, 'a, Q: Narrows<Universal>> Policy<'rules, 'a, Q> {
    /// Creates a policy requiring an exact target projection.
    pub fn new() -> Self {
        Self::default()
    }
    /// Appends a reusable rule or an application-defined closure.
    pub fn with_rule(mut self, rule: impl Rewrite<'a, Q> + 'rules) -> Self {
        self.rules.push(Box::new(rule));
        self
    }
    /// Converts and validates a target model without changing the source.
    pub fn convert(&self, source: &Model<'a>) -> Result<Conversion<Model<'a, Q>>, Error> {
        let mut current = source.clone();
        let mut report = Report::default();
        for rule in &self.rules {
            let mut draft = ModelBuilder::from_model(current);
            rule.apply(&mut draft, &mut report)?;
            current = draft.finish()?;
        }
        let target = current.try_narrow::<Q>()?;
        Ok(Conversion {
            value: target,
            report,
        })
    }
}

/// Drops only explicitly selected features that the target cannot represent.
///
/// Supported selected features are retained. References to a dropped feature
/// are not silently repaired: graph validation rejects dangling relationships.
/// Resources and animation slots remain owned by the graph and are not pruned.
#[derive(Clone, Debug)]
pub struct DropUnsupportedFeatures {
    selected: Vec<FeatureId>,
}
impl DropUnsupportedFeatures {
    /// Selects the complete attached behaviors that may be discarded.
    pub fn new(selected: impl IntoIterator<Item = FeatureId>) -> Self {
        Self {
            selected: selected.into_iter().collect(),
        }
    }
}
impl<'a, Q: Narrows<Universal>> Rewrite<'a, Q> for DropUnsupportedFeatures {
    fn apply(&self, draft: &mut ModelBuilder<'a>, report: &mut Report) -> Result<(), Error> {
        for &id in &self.selected {
            let Some(feature) = draft.model().feature(id) else {
                return Err(Error::requirement(
                    Location::for_owner(id),
                    "selected feature is absent",
                ));
            };
            let compatibility =
                Q::validate_feature(draft.model(), &feature.payload).and_then(|()| {
                    Q::feature(feature.payload.clone(), draft.model().values()).map(|_| ())
                });
            let Err(reason) = compatibility else {
                continue;
            };
            draft.remove_feature(id);
            report.issues.push(Issue {
                path: format!("features[{id:?}]"),
                kind: IssueKind::Dropped,
                description: format!("discarded selected behavior: {reason}"),
            });
        }
        Ok(())
    }
}

/// Expresses selected constant light switches as fractional visibility for MDLX.
///
/// Animated switches are rejected rather than sampled at a single time. Existing
/// fractional visibility is retained. Other light semantics still undergo the
/// final exact target projection; this rule does not authorize unrelated loss.
#[derive(Clone, Debug)]
pub struct ConstantLightVisibility {
    selected: Vec<FeatureId>,
}
impl ConstantLightVisibility {
    /// Selects the lights whose constant activation may be re-expressed.
    pub fn new(selected: impl IntoIterator<Item = FeatureId>) -> Self {
        Self {
            selected: selected.into_iter().collect(),
        }
    }
}
impl<'a> Rewrite<'a, Mdlx> for ConstantLightVisibility {
    fn apply(&self, draft: &mut ModelBuilder<'a>, report: &mut Report) -> Result<(), Error> {
        for &id in &self.selected {
            let path = Location::for_owner(id).field("activation");
            let feature = draft
                .model()
                .feature(id)
                .ok_or_else(|| Error::requirement(path.clone(), "selected feature is absent"))?;
            let NodeFeature::Light(Light::AmbientDiffuse(light)) = &feature.payload else {
                return Err(Error::requirement(
                    path,
                    "selected feature is not an ambient/diffuse light",
                ));
            };
            let LightActivation::Enabled(enabled) = light.activation else {
                continue;
            };
            if draft.model().is_animated(enabled) {
                return Err(Error::requirement(
                    path,
                    "animated light activation requires curve conversion",
                ));
            }
            let enabled =
                *draft.model().values().get(enabled).ok_or_else(|| {
                    Error::requirement(path.clone(), "activation value is absent")
                })?;
            let mut light = light.clone();
            let visibility = draft.add_value(if enabled { 1.0 } else { 0.0 });
            light.activation = LightActivation::Visibility(visibility);
            draft.set_feature(id, NodeFeature::Light(Light::AmbientDiffuse(light)))?;
            report.issues.push(Issue {
                path: path.to_string(),
                kind: IssueKind::Normalized,
                description: "expressed a constant light switch as zero-or-one visibility".into(),
            });
        }
        Ok(())
    }
}

/// Approximates selected constant fractional light visibility by an M2 switch.
///
/// The error bound applies to the dimensionless activation value, not rendered
/// pixels. Animated inputs and visibility outside zero to one are rejected.
#[derive(Clone, Debug)]
pub struct ConstantLightSwitch {
    selected: Vec<FeatureId>,
    threshold: f32,
    max_error: f32,
}
impl ConstantLightSwitch {
    /// Chooses an explicit switching threshold and maximum absolute activation error.
    ///
    /// Visibility at or above `threshold` becomes enabled. The threshold must
    /// be finite and between zero and one; `max_error` must be finite and
    /// nonnegative. Every selected value is checked against this bound.
    pub fn new(
        selected: impl IntoIterator<Item = FeatureId>,
        threshold: f32,
        max_error: f32,
    ) -> Result<Self, Error> {
        if !threshold.is_finite() || !(0.0..=1.0).contains(&threshold) {
            return Err(Error::requirement(
                Location::new().field("light_switch").field("threshold"),
                "expected a finite value from zero to one",
            ));
        }
        if !max_error.is_finite() || max_error < 0.0 {
            return Err(Error::requirement(
                Location::new().field("light_switch").field("max_error"),
                "expected a finite nonnegative activation error",
            ));
        }
        Ok(Self {
            selected: selected.into_iter().collect(),
            threshold,
            max_error,
        })
    }
}
impl<'a> Rewrite<'a, M2> for ConstantLightSwitch {
    fn apply(&self, draft: &mut ModelBuilder<'a>, report: &mut Report) -> Result<(), Error> {
        for &id in &self.selected {
            let path = Location::for_owner(id).field("activation");
            let feature = draft
                .model()
                .feature(id)
                .ok_or_else(|| Error::requirement(path.clone(), "selected feature is absent"))?;
            let NodeFeature::Light(Light::AmbientDiffuse(light)) = &feature.payload else {
                return Err(Error::requirement(
                    path,
                    "selected feature is not an ambient/diffuse light",
                ));
            };
            let LightActivation::Visibility(visibility) = light.activation else {
                continue;
            };
            if draft.model().is_animated(visibility) {
                return Err(Error::requirement(
                    path,
                    "animated light visibility requires curve conversion",
                ));
            }
            let visibility =
                *draft.model().values().get(visibility).ok_or_else(|| {
                    Error::requirement(path.clone(), "visibility value is absent")
                })?;
            if !visibility.is_finite() || !(0.0..=1.0).contains(&visibility) {
                return Err(Error::requirement(
                    path,
                    "switch approximation requires visibility from zero to one",
                ));
            }
            let enabled = visibility >= self.threshold;
            let error = (f64::from(visibility) - if enabled { 1.0 } else { 0.0 }).abs();
            if error > f64::from(self.max_error) {
                return Err(Error::requirement(
                    path,
                    format!(
                        "activation error {error} exceeds maximum {}",
                        self.max_error
                    ),
                ));
            }
            let mut light = light.clone();
            light.activation = LightActivation::Enabled(draft.add_value(enabled));
            draft.set_feature(id, NodeFeature::Light(Light::AmbientDiffuse(light)))?;
            report.issues.push(Issue {
                path: path.to_string(),
                kind: if error == 0.0 { IssueKind::Normalized } else { IssueKind::Approximated },
                description: format!("converted visibility {visibility} to {enabled} using threshold {}; absolute activation error {error}, maximum {}", self.threshold, self.max_error),
            });
        }
        Ok(())
    }
}
