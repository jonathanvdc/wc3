use super::material::Wc3LayerMaterial;
use bevy::prelude::*;
use wc3::model::animation::{Animatable, Sequence, Track};
use wc3::model::materials::LayerFilterMode;

#[derive(Component)]
pub struct Wc3Animation {
    pub sequence: usize,
    pub elapsed_ms: f64,
    pub speed: f64,
    pub playing: bool,
    pub(crate) sequences: Vec<Sequence>,
    pub(crate) global_sequences: Vec<u32>,
}

impl Wc3Animation {
    pub fn sequences(&self) -> &[Sequence] {
        &self.sequences
    }

    pub fn play(&mut self, sequence: usize) {
        if sequence < self.sequences.len() {
            self.sequence = sequence;
            self.elapsed_ms = 0.0;
            self.playing = true;
        }
    }
}

#[derive(Component)]
pub(crate) struct AnimatedNode {
    pub(crate) root: Entity,
    pub(crate) pivot: Vec3,
    pub(crate) parent_pivot: Vec3,
    pub(crate) translation: Option<Track<[f32; 3]>>,
    pub(crate) rotation: Option<Track<[f32; 4]>>,
    pub(crate) scaling: Option<Track<[f32; 3]>>,
}

#[derive(Component)]
pub(crate) struct AnimatedLayer {
    pub(crate) root: Entity,
    pub(crate) alpha: Animatable<f32>,
    pub(crate) geoset_alpha: Option<Animatable<f32>>,
    pub(crate) texture_id: Animatable<u32>,
    pub(crate) textures: Vec<Option<Handle<Image>>>,
}

pub(crate) fn animate_layers(
    instances: Query<&Wc3Animation>,
    mut layers: Query<(
        &AnimatedLayer,
        &MeshMaterial3d<Wc3LayerMaterial>,
        &mut Visibility,
    )>,
    mut materials: ResMut<Assets<Wc3LayerMaterial>>,
) {
    for (layer, material_handle, mut visibility) in &mut layers {
        let Ok(animation) = instances.get(layer.root) else {
            continue;
        };
        let Some(mut material) = materials.get_mut(&material_handle.0) else {
            continue;
        };
        let alpha = layer
            .alpha
            .track()
            .and_then(|track| sample(track, animation))
            .or_else(|| layer.alpha.value().copied())
            .unwrap_or(1.0);
        let geoset_alpha = layer
            .geoset_alpha
            .as_ref()
            .map(|alpha| {
                alpha
                    .track()
                    .and_then(|track| sample(track, animation))
                    .or_else(|| alpha.value().copied())
                    .unwrap_or(1.0)
            })
            .unwrap_or(1.0);
        *visibility = if alpha * geoset_alpha <= 0.0 {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
        if layer.geoset_alpha.is_some() {
            let partial = (0.0..1.0).contains(&geoset_alpha);
            match material.extension.filter {
                LayerFilterMode::None => {
                    material.base.alpha_mode = if partial {
                        AlphaMode::AlphaToCoverage
                    } else {
                        AlphaMode::Opaque
                    };
                }
                LayerFilterMode::Transparent => {
                    material.base.alpha_mode = if partial {
                        AlphaMode::AlphaToCoverage
                    } else {
                        AlphaMode::Mask(0.5)
                    };
                }
                _ => {}
            }
        }
        let texture_id = layer
            .texture_id
            .track()
            .and_then(|track| sample(track, animation))
            .or_else(|| layer.texture_id.value().copied())
            .unwrap_or(0);
        material.base.base_color = Color::srgba(1.0, 1.0, 1.0, alpha * geoset_alpha);
        material.base.base_color_texture =
            layer.textures.get(texture_id as usize).cloned().flatten();
    }
}

pub(crate) fn advance_animation(time: Res<Time>, mut instances: Query<&mut Wc3Animation>) {
    for mut instance in &mut instances {
        if instance.playing {
            instance.elapsed_ms += time.delta_secs_f64() * 1000.0 * instance.speed;
        }
    }
}

fn sample<T: wc3::model::animation::Interpolate>(
    track: &Track<T>,
    animation: &Wc3Animation,
) -> Option<T> {
    if let Some(global_id) = track.global_sequence_id() {
        let length = *animation.global_sequences.get(global_id as usize)?;
        if length == 0 {
            return track.evaluate(0.0);
        }
        return track.evaluate(animation.elapsed_ms.rem_euclid(length as f64));
    }
    let sequence = animation.sequences.get(animation.sequence)?;
    let start = sequence.interval[0] as f64;
    let end = sequence.interval[1] as f64;
    let length = (end - start).max(0.0);
    let elapsed = if sequence.flags.non_looping() {
        animation.elapsed_ms.clamp(0.0, length)
    } else if length > 0.0 {
        animation.elapsed_ms.rem_euclid(length)
    } else {
        0.0
    };
    track.evaluate_in(
        start + elapsed,
        sequence.interval[0] as i32..=sequence.interval[1] as i32,
    )
}

pub(crate) fn animate_nodes(
    instances: Query<&Wc3Animation>,
    mut nodes: Query<(&AnimatedNode, &mut Transform)>,
) {
    for (node, mut transform) in &mut nodes {
        let Ok(animation) = instances.get(node.root) else {
            continue;
        };
        let translation = node
            .translation
            .as_ref()
            .and_then(|track| sample(track, animation))
            .unwrap_or([0.0; 3]);
        let rotation = node
            .rotation
            .as_ref()
            .and_then(|track| sample(track, animation))
            .unwrap_or([0.0, 0.0, 0.0, 1.0]);
        let scaling = node
            .scaling
            .as_ref()
            .and_then(|track| sample(track, animation))
            .unwrap_or([1.0; 3]);
        transform.translation = node.pivot - node.parent_pivot + Vec3::from_array(translation);
        transform.rotation = Quat::from_xyzw(rotation[0], rotation[1], rotation[2], rotation[3]);
        transform.scale = Vec3::from_array(scaling);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::Wc3LayerState;
    use wc3::model::animation::ValueKeyframe;
    use wc3::model::materials::LayerFilterMode;

    #[test]
    fn geoset_alpha_hides_decay_geometry_and_combines_with_layer_alpha() {
        let mut app = App::new();
        app.insert_resource(Assets::<Wc3LayerMaterial>::default());
        app.add_systems(Update, animate_layers);
        let root = app
            .world_mut()
            .spawn(Wc3Animation {
                sequence: 0,
                elapsed_ms: 0.0,
                speed: 1.0,
                playing: false,
                sequences: vec![Sequence::new("Stand", [0, 101]).unwrap()],
                global_sequences: vec![],
            })
            .id();
        let material = app
            .world_mut()
            .resource_mut::<Assets<Wc3LayerMaterial>>()
            .add(Wc3LayerMaterial {
                base: StandardMaterial::default(),
                extension: Wc3LayerState {
                    filter: LayerFilterMode::None,
                    no_depth_test: false,
                    no_depth_set: false,
                },
            });
        let entity = app
            .world_mut()
            .spawn((
                AnimatedLayer {
                    root,
                    alpha: Animatable::Static(0.5),
                    geoset_alpha: Some(Animatable::Animated(
                        Track::linear(
                            vec![
                                ValueKeyframe {
                                    frame: 0,
                                    value: 0.0,
                                },
                                ValueKeyframe {
                                    frame: 100,
                                    value: 1.0,
                                },
                            ],
                            None,
                        )
                        .unwrap(),
                    )),
                    texture_id: Animatable::Static(0),
                    textures: vec![],
                },
                MeshMaterial3d(material.clone()),
                Visibility::Visible,
            ))
            .id();
        app.update();
        assert_eq!(
            *app.world().entity(entity).get::<Visibility>().unwrap(),
            Visibility::Hidden
        );
        app.world_mut()
            .entity_mut(root)
            .get_mut::<Wc3Animation>()
            .unwrap()
            .elapsed_ms = 50.0;
        app.update();
        assert_eq!(
            *app.world().entity(entity).get::<Visibility>().unwrap(),
            Visibility::Visible
        );
        assert_eq!(
            app.world()
                .resource::<Assets<Wc3LayerMaterial>>()
                .get(&material)
                .unwrap()
                .base
                .base_color
                .to_srgba()
                .alpha,
            0.25,
        );
        assert_eq!(
            app.world()
                .resource::<Assets<Wc3LayerMaterial>>()
                .get(&material)
                .unwrap()
                .base
                .alpha_mode,
            AlphaMode::AlphaToCoverage,
        );
        app.world_mut()
            .entity_mut(root)
            .get_mut::<Wc3Animation>()
            .unwrap()
            .elapsed_ms = 100.0;
        app.update();
        let material = app
            .world()
            .resource::<Assets<Wc3LayerMaterial>>()
            .get(&material)
            .unwrap();
        assert_eq!(material.base.alpha_mode, AlphaMode::Opaque);
        app.world_mut()
            .entity_mut(entity)
            .get_mut::<AnimatedLayer>()
            .unwrap()
            .alpha = Animatable::Static(0.0);
        app.update();
        assert_eq!(
            *app.world().entity(entity).get::<Visibility>().unwrap(),
            Visibility::Hidden
        );
    }

    #[test]
    fn sequence_clock_loops_and_global_clock_runs_independently() {
        let animation = Wc3Animation {
            sequence: 0,
            elapsed_ms: 150.0,
            speed: 1.0,
            playing: true,
            sequences: vec![Sequence::new("Stand", [1000, 1100]).unwrap()],
            global_sequences: vec![200],
        };
        let sequence_track = Track::linear(
            vec![
                ValueKeyframe {
                    frame: 1000,
                    value: 0.0f32,
                },
                ValueKeyframe {
                    frame: 1100,
                    value: 1.0f32,
                },
            ],
            None,
        )
        .unwrap();
        let global_track = Track::linear(
            vec![
                ValueKeyframe {
                    frame: 0,
                    value: 0.0f32,
                },
                ValueKeyframe {
                    frame: 200,
                    value: 1.0f32,
                },
            ],
            Some(0),
        )
        .unwrap();
        assert_eq!(sample(&sequence_track, &animation), Some(0.5));
        assert_eq!(sample(&global_track, &animation), Some(0.75));
    }
}
