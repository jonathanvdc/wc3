use bevy::prelude::*;
use wc3::model::animation::Sequence;

#[derive(Component, Clone)]
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

pub(crate) fn advance_animation(time: Res<Time>, mut instances: Query<&mut Wc3Animation>) {
    for mut instance in &mut instances {
        if instance.playing {
            instance.elapsed_ms += time.delta_secs_f64() * 1000.0 * instance.speed;
        }
    }
}
