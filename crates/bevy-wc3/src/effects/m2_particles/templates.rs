//! Immutable first-four-slot adopted-template catalogue, compiled from Universal.
//! Template slots remain authored slots, including omitted ones. Instances share
//! these definitions and images; no source graph, native document or clock survives.
use super::birth::{BirthFrame, BirthInputs};
use super::{signed, Birth, BirthAppearance, Emitter};
use crate::diagnostics::{RenderDiagnosticKind, RenderIssue};
use bevy::prelude::{Handle, Image, Vec3};
use std::collections::HashMap;
use std::sync::Arc;
use wc3::model::values::model::ParticleGenerator;
use wc3::model::{
    AnimationSample, FeatureId, Model, NodeFeature, ParticleEmitter, Time, ValueSample,
};

pub(super) const MAX_TEMPLATES: usize = 4;
#[derive(Clone, Default)]
pub(super) struct TemplateCatalogue {
    pub slots: Vec<TemplateSlot>,
}
#[derive(Clone)]
pub(super) struct TemplateSlot {
    pub feature: FeatureId,
    pub template: Result<Arc<FrozenTemplate>, RenderIssue>,
}
#[derive(Clone)]
pub(super) struct FrozenTemplate {
    // Deliberately private: no caller can use the source node/animation slots.
    // Only immutable motion/appearance/style data are used after preparation.
    kernel: Arc<Emitter>,
    inputs: BirthInputs,
    pub rate: f64,
    pub rate_variation: f32,
    pub issues: Vec<RenderIssue>,
}
impl TemplateCatalogue {
    pub(super) fn compile(
        model: &Model<'_>,
        emitters: &[Arc<Emitter>],
        issues: &HashMap<FeatureId, Vec<RenderIssue>>,
    ) -> Self {
        let selection = AnimationSample {
            clip: model.clips().iter().next().map(|(id, _)| id),
            clip_time: Time::seconds(0.).unwrap(),
            shared_time: Time::seconds(0.).unwrap(),
        };
        let sample = model.animation_sample(selection);
        let slots = model
            .features()
            .iter()
            .filter_map(|(feature, value)| {
                let NodeFeature::Particle(ParticleEmitter::M2(definition)) = &value.payload else {
                    return None;
                };
                let details = &definition.details;
                let reason = if details.rendering.child_model.is_some() {
                    Some("Nested M2 adopted emitter resources are not adopted")
                } else if details.rendering.model.is_some() {
                    Some("Geometry-model outputs are not adopted as M2 sprite templates")
                } else if details.shape.generator == ParticleGenerator::Bone {
                    Some("Bone generators require a posed model and are not adopted")
                } else {
                    None
                };
                let template = if let Some(message) = reason {
                    Err(RenderIssue::new(RenderDiagnosticKind::Unsupported, message))
                } else if let Ok(sample) = &sample {
                    emitters
                        .iter()
                        .find(|emitter| emitter.feature == feature)
                        .ok_or_else(|| {
                            RenderIssue::new(
                                RenderDiagnosticKind::Unsupported,
                                "The M2 template has no supported sprite interpreter",
                            )
                        })
                        .map(|emitter| {
                            let mut template = FrozenTemplate::new(emitter, sample);
                            template.issues = issues.get(&feature).cloned().unwrap_or_default();
                            if details.flags.attached() {
                                template.issues.push(RenderIssue::new(
                                    RenderDiagnosticKind::Unverified,
                                    "Attached adopted templates use the owner-local particle frame and inverse-owner-frame inherited velocity; exact native inheritance, follow and frame behavior remain unverified",
                                ));
                            }
                            Arc::new(template)
                        })
                } else {
                    Err(RenderIssue::new(
                        RenderDiagnosticKind::Resource,
                        "The template's first playback clip could not be sampled",
                    ))
                };
                Some(TemplateSlot { feature, template })
            })
            .take(MAX_TEMPLATES)
            .collect();
        Self { slots }
    }
    pub(super) fn remap_images(
        &mut self,
        kernels: &HashMap<*const Emitter, Arc<Emitter>>,
        map: &mut impl FnMut(&Handle<Image>) -> Option<Handle<Image>>,
    ) {
        for slot in &mut self.slots {
            if let Ok(template) = &mut slot.template {
                let template = Arc::make_mut(template);
                if let Some(kernel) = kernels.get(&Arc::as_ptr(&template.kernel)) {
                    template.kernel = kernel.clone();
                    continue;
                }
                for image in Arc::make_mut(&mut template.kernel)
                    .gpu_style
                    .textures
                    .iter_mut()
                    .flatten()
                {
                    if let Some(replacement) = map(image) {
                        *image = replacement;
                    }
                }
            }
        }
    }
}
impl FrozenTemplate {
    pub(super) fn new(emitter: &Arc<Emitter>, sample: &impl ValueSample) -> Self {
        let mut inputs = BirthInputs::sample(emitter, sample);
        inputs.enabled = true;
        Self {
            kernel: emitter.clone(),
            inputs,
            rate: f64::from(sample.animated(&emitter.rate).max(0.)),
            rate_variation: emitter.rate_variation,
            issues: Vec::new(),
        }
    }
    pub(super) fn birth(&self, mut frame: BirthFrame) -> Option<(Birth, Option<BirthAppearance>)> {
        // This path varies inherited velocity around one, independently of the
        // ordinary node-displacement multiplier. Existing birth packets retain
        // their draw layout; exact native random history remains unverified.
        let mut seed = self.kernel.seed ^ frame.ordinal.wrapping_mul(0x3c79ac492ba7b653);
        frame.inherited = if self.kernel.flags.inherit_velocity() {
            frame.inherited * (1. + signed(&mut seed) * self.kernel.inherit_velocity)
        } else {
            Vec3::ZERO
        };
        self.kernel.birth_sampled(self.inputs, frame, |_| None)
    }
    pub(super) fn kernel(&self) -> &Arc<Emitter> {
        &self.kernel
    }
}
