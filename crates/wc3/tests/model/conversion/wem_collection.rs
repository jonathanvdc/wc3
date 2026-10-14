use wc3::model::conversion::{self, ExportOptions, ImportOptions};
use wc3::model::semantic::profiles::wem::{
    actors::warcraft::Feature as WarcraftFeature,
    animation::{AnyDeclaration, Target as AnimationTarget},
    Feature,
};
use wc3::model::wem::{self, records, RawDocument, Record, Ref, Target, Write};

fn rows<T: Record + Write, U: Target>(document: &mut RawDocument<'_>, values: &[T]) -> Ref<U> {
    let section = document.append(values).unwrap();
    Ref::from_section(values.len() as u32, section.index())
}
#[test]
fn global_clip_bank_keeps_one_owner_and_shared_texture_across_models() {
    let mut source = RawDocument::new();
    let path = source
        .append_raw(*b"CHAR", 0, 10, b"shared.blp".to_vec())
        .unwrap();
    let path = Ref::from_section(10, path.index());
    let textures = rows(
        &mut source,
        &[records::TextureRecord {
            key: records::TextureKey::Path(records::TexturePath { value: path }),
            ..Default::default()
        }],
    );
    let nodes = rows(
        &mut source,
        &[records::Node::<records::V1> {
            parent: u32::MAX,
            kind: records::NodeKind::Wc3ParticleEmitter2,
            local: records::Transform {
                translation: [0.; 3],
                rotation: [0., 0., 0., 1.],
                scale: [1.; 3],
            },
            payload: records::NodePayload::Wc3ParticleEmitter2(Box::new(
                records::Wc3ParticleEmitter2Payload {
                    texture: 0,
                    lifespan: 1.,
                    speed: 1.,
                    emission_rate: 1.,
                    width: 1.,
                    length: 1.,
                    rows: 1,
                    columns: 1,
                    time: 0.5,
                    start: records::Wc3ParticleSegment {
                        color: [1.; 3],
                        alpha: 255,
                        scaling: 1.,
                    },
                    middle: records::Wc3ParticleSegment {
                        color: [1.; 3],
                        alpha: 255,
                        scaling: 1.,
                    },
                    end: records::Wc3ParticleSegment {
                        color: [1.; 3],
                        alpha: 0,
                        scaling: 1.,
                    },
                    ..Default::default()
                },
            )),
            ..Default::default()
        }],
    );
    let channels = rows(
        &mut source,
        &[records::AnimChannel {
            id: 0,
            target: records::TrackTarget {
                kind: records::TrackTargetKind::Node,
                node: 0,
                channel: records::Channel::Translation,
                ..Default::default()
            },
            value_type: records::AttrType::F32x3,
            ..Default::default()
        }],
    );
    let times = rows(&mut source, &[records::Real { value: 0. }]);
    let bytes: [u8; 12] = [1f32, 2., 3.]
        .into_iter()
        .flat_map(f32::to_le_bytes)
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    let data = source.append_raw(*b"U8__", 0, 12, bytes.to_vec()).unwrap();
    let tracks = rows(
        &mut source,
        &[records::SubTrack::<records::V2> {
            channel: 0,
            interp: records::Interpolation::Linear,
            times,
            values: Ref::from_section(12, data.index()),
            ..Default::default()
        }],
    );
    let containers = rows(
        &mut source,
        &[records::SubTrackContainer {
            sub_tracks: tracks,
            ..Default::default()
        }],
    );
    let clips = rows(
        &mut source,
        &[records::Clip::<records::V1> {
            model: 0,
            duration: 1.,
            looping: true.into(),
            containers,
            ..Default::default()
        }],
    );
    let tags = rows(
        &mut source,
        &[records::AnimTag {
            tag_id: 42,
            clip: 0,
        }],
    );
    let sets = rows(
        &mut source,
        &[records::AnimSet {
            by_tag: tags,
            base_anim_set: u32::MAX,
            ..Default::default()
        }],
    );
    let model = records::ModelRecord::<records::V1> {
        nodes: records::NodeTree {
            nodes,
            authoritative_pose: u32::MAX,
            ..Default::default()
        },
        anim_channels: records::AnimChannelTable { channels },
        anim_set: 0,
        ..Default::default()
    };
    let models = rows(&mut source, &[model.clone(), model]);
    let root: Ref<records::DocumentRecord<records::V1>> = rows(
        &mut source,
        &[records::DocumentRecord::<records::V1> {
            models,
            textures,
            clips,
            anim_sets: sets,
            unit_scale: 1.,
            profiles: wem::InlineList(vec![records::ProfileId::Generic]),
            ..Default::default()
        }],
    );
    source.set_root(root);
    let first = conversion::wem::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    check(&first);
    let output = conversion::wem::plan(&first, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let bytes = output.write().unwrap();
    let output = RawDocument::read_with_options(&bytes, output.options()).unwrap();
    let second = conversion::wem::import(&output, &ImportOptions::default())
        .unwrap()
        .value;
    check(&second);
}
fn check(asset: &conversion::Asset<'_, wc3::model::semantic::profiles::Wem>) {
    assert_eq!(asset.models.len(), 2);
    assert_eq!(asset.resources().textures().len(), 1);
    let first = &asset.models[0];
    let second = &asset.models[1];
    assert_eq!(first.clips().len(), 1);
    assert_eq!(second.clips().len(), 0);
    assert_eq!(first.controllers().len(), 1);
    assert_eq!(second.controllers().len(), 0);
    assert_eq!(first.controls().channels.declarations().len(), 1);
    assert_eq!(second.controls().channels.declarations().len(), 1);
    let first_node = first.nodes().next().unwrap().0;
    let second_node = second.nodes().next().unwrap().0;
    assert_ne!(first_node, second_node);
    for (model, node) in [(first, first_node), (second, second_node)] {
        let declaration = model.controls().channels.declarations().next().unwrap();
        let AnyDeclaration::Vector3(_, declaration) = declaration else {
            panic!("channel domain changed")
        };
        assert!(matches!(declaration.target, AnimationTarget::Node{node:owner,..} if owner==node));
    }
    let clip = first.clips().next().unwrap().0;
    let set = asset.controls().animation_sets().next().unwrap().0;
    assert_eq!(first.controls().animation_set, Some(set));
    assert_eq!(second.controls().animation_set, Some(set));
    assert_eq!(
        asset.controls().animation_set(set).unwrap().tags,
        vec![(42, clip)]
    );
    let texture = asset.resources().textures().next().unwrap().0;
    for model in asset.models.iter() {
        let Feature::Warcraft(actor) = &model.features().next().unwrap().1.payload else {
            panic!("actor family changed")
        };
        let WarcraftFeature::SpriteParticles(actor) = actor.as_ref() else {
            panic!("actor kind changed")
        };
        assert_eq!(actor.texture, Some(texture));
    }
}
