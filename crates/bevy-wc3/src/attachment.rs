//! Attachment points, mounted models, and animated attachment visibility.
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use wc3::model::animation::Track;

use crate::animation::{sample, Wc3Animation};
use crate::assets::loader::Wc3ModelAsset;
use crate::instance::{Wc3ModelInstance, Wc3ModelOwner};
use crate::preparation::PreparedModel;

/// One attachment point in a model instance. `node` is the animated MDX node;
/// `mount` follows it and gates only attached content with the visibility track.
#[derive(Clone, Debug)]
pub struct Wc3AttachmentPoint {
    /// Parent model animation root and lifetime owner.
    pub root: Entity,
    /// Animated source node, before attachment visibility gating.
    pub node: Entity,
    /// Child mount that inherits the node transform and attachment visibility.
    pub mount: Entity,
    /// Authored attachment ID, independent of the node object ID.
    pub id: u32,
    /// Full authored attachment name.
    pub name: String,
    /// Automatically spawned path model, if the resource was resolved.
    pub model: Option<Entity>,
}

impl Wc3AttachmentPoint {
    /// Spawn an independently animated model on this point. It follows the
    /// mount, shares its visibility, and is cleaned up with the parent instance.
    /// Attachment visibility transitions restart sequence zero; sequences loop.
    /// Uses `Wc3BevyPlugin` to load and animate the child model.
    pub fn spawn_model(&self, commands: &mut Commands, model: Handle<Wc3ModelAsset>) -> Entity {
        commands
            .spawn((
                Wc3ModelInstance::new(model),
                ChildOf(self.mount),
                Wc3ModelOwner(self.root),
                AttachmentModel {
                    mount: self.mount,
                    initialized: false,
                    visible: false,
                    restart: 0,
                },
            ))
            .id()
    }
}

/// Attachment points on an instance root, in source record order. IDs and names
/// are independent of both the record index and MDX node object ID.
#[derive(Component)]
pub struct Wc3Attachments(Vec<Wc3AttachmentPoint>);

impl Wc3Attachments {
    /// Look up a point by attachment record index.
    pub fn get(&self, index: usize) -> Option<&Wc3AttachmentPoint> {
        self.0.get(index)
    }

    /// Return the first point with this authored attachment ID.
    pub fn by_id(&self, id: u32) -> Option<&Wc3AttachmentPoint> {
        self.0.iter().find(|point| point.id == id)
    }

    /// ASCII case-insensitive lookup of the full name, e.g. `Weapon Ref`.
    pub fn by_name(&self, name: &str) -> Option<&Wc3AttachmentPoint> {
        self.0
            .iter()
            .find(|point| point.name.eq_ignore_ascii_case(name))
    }

    /// Iterate points in source record order.
    pub fn iter(&self) -> impl Iterator<Item = &Wc3AttachmentPoint> {
        self.0.iter()
    }
}

#[derive(Component)]
pub(crate) struct AnimatedAttachment {
    root: Entity,
    visibility: Option<Track<f32>>,
    last_sequence: usize,
    last_elapsed_ms: f64,
    restart: u64,
}

#[derive(Component)]
pub(crate) struct AttachmentModel {
    mount: Entity,
    initialized: bool,
    visible: bool,
    restart: u64,
}

pub(crate) fn spawn_attachments(
    commands: &mut Commands,
    prepared: &PreparedModel,
    root: Entity,
    nodes: &HashMap<u32, Entity>,
) {
    let mut points = Vec::new();
    for (index, attachment) in prepared.model.attachments().into_iter().enumerate() {
        let Some(&node) = nodes.get(&attachment.node.object_id) else {
            continue;
        };
        let mount = commands
            .spawn((
                Transform::default(),
                Visibility::Hidden,
                ChildOf(node),
                AnimatedAttachment {
                    root,
                    visibility: attachment.visibility,
                    last_sequence: usize::MAX,
                    last_elapsed_ms: 0.0,
                    restart: 0,
                },
            ))
            .id();
        let mut point = Wc3AttachmentPoint {
            root,
            node,
            mount,
            id: attachment.id,
            name: attachment.node.name.text().into_owned(),
            model: None,
        };
        if let Some(model) = prepared.model_resources().attachment(index) {
            point.model = Some(point.spawn_model(commands, model));
        }
        points.push(point);
    }
    commands.entity(root).insert(Wc3Attachments(points));
}

pub(crate) fn animate_attachments(
    mut animations: Query<&mut Wc3Animation>,
    mut attachments: Query<(Entity, &mut AnimatedAttachment, &mut Visibility)>,
    mut models: Query<(Entity, &mut AttachmentModel)>,
    owners: Query<&Wc3ModelOwner>,
    root_visibility: Query<&Visibility, Without<AnimatedAttachment>>,
) {
    // Process outer models first so a child's restart is visible to its own
    // attachment tracks in this same frame, including nested child models.
    let mut depths = HashMap::new();
    let mut ordered = Vec::new();
    for (entity, attachment, _) in &attachments {
        let depth = *depths.entry(attachment.root).or_insert_with(|| {
            let mut root = attachment.root;
            let mut ancestors = HashSet::new();
            while ancestors.insert(root) {
                let Ok(owner) = owners.get(root) else {
                    break;
                };
                root = owner.0;
            }
            ancestors.len()
        });
        ordered.push((entity, depth));
    }
    ordered.sort_by_key(|(_, depth)| *depth);
    let mut children: HashMap<Entity, Vec<Entity>> = HashMap::new();
    for (entity, model) in &models {
        children.entry(model.mount).or_default().push(entity);
    }
    for (entity, _) in ordered {
        let Ok((_, mut attachment, mut visibility)) = attachments.get_mut(entity) else {
            continue;
        };
        let Ok(animation) = animations.get(attachment.root) else {
            continue;
        };
        let value = attachment
            .visibility
            .as_ref()
            .and_then(|track| sample(track, animation))
            .unwrap_or(1.0);
        if animation.sequence != attachment.last_sequence
            || animation.elapsed_ms < attachment.last_elapsed_ms
        {
            attachment.restart = attachment.restart.wrapping_add(1);
        }
        attachment.last_sequence = animation.sequence;
        attachment.last_elapsed_ms = animation.elapsed_ms;
        let parent_visible = root_visibility
            .get(attachment.root)
            .is_ok_and(|visibility| *visibility != Visibility::Hidden)
            && models
                .get(attachment.root)
                .map_or(true, |(_, model)| model.visible);
        let visible = value > 0.1 && parent_visible;
        *visibility = if visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        for child in children.get(&entity).into_iter().flatten() {
            let Ok(mut animation) = animations.get_mut(*child) else {
                continue;
            };
            let Ok((_, mut model)) = models.get_mut(*child) else {
                continue;
            };
            if !model.initialized {
                // Override the instance's flags without changing its shared asset.
                for sequence in &mut animation.sequences {
                    sequence.flags.set_non_looping(false);
                }
            }
            if !model.initialized || visible != model.visible || model.restart != attachment.restart
            {
                animation.playing = visible;
                if visible {
                    if animation.sequences().is_empty() {
                        animation.restart();
                    } else {
                        animation.play(0);
                    }
                }
            }
            model.initialized = true;
            model.visible = visible;
            model.restart = attachment.restart;
        }
    }
}

#[cfg(test)]
#[path = "attachment_tests.rs"]
mod tests;
