use wc3::model::formats::mdlx::geometry::BindPoseMatrix;
use wc3::model::formats::mdlx::visit_document;
use wc3::model::formats::mdlx::{
    CommonModelAccess, Document, DynamicDocument, TryModelAccess, ValueError, V1800, V800, V900,
};
use wc3::model::formats::mdlx::{Extended, NoExtensions};

fn replace_durations<'data>(model: &mut impl CommonModelAccess<'data>) {
    model.set_global_sequences(&[100, 250]);
    assert_eq!(model.global_sequences(), [100, 250]);
}

#[test]
fn shared_accessors_work_for_typed_and_runtime_models() {
    let mut typed = Document::<V800>::new();
    replace_durations(&mut typed);

    let mut runtime = DynamicDocument::<NoExtensions>::V1800(Document::new());
    replace_durations(&mut runtime);
    assert_eq!(runtime.global_sequences(), [100, 250]);
    assert_eq!(runtime.model_info(), None);
}

fn check_bind_poses<'data>(model: &mut impl TryModelAccess<'data>, supported: bool) {
    let poses = [BindPoseMatrix([0.0; 12])];
    if supported {
        assert!(model.try_bind_poses().unwrap().next().is_none());
        model.try_set_bind_poses(poses).unwrap();
        assert_eq!(
            model.try_bind_poses().unwrap().copied().collect::<Vec<_>>(),
            poses
        );
    } else {
        assert!(
            matches!(model.try_bind_poses(), Err(ValueError::UnsupportedVersion { tag, minimum: 900, actual: 800 }) if tag == *b"BPOS")
        );
        assert!(model.try_set_bind_poses(poses).is_err());
    }
}

#[test]
fn checked_accessors_distinguish_unsupported_and_empty() {
    let mut old = Document::<V800>::new();
    check_bind_poses(&mut old, false);
    assert_eq!(old.chunks.len(), 1);

    let mut modern = Document::<V1800>::new();
    check_bind_poses(&mut modern, true);
    assert_eq!(
        modern.bind_poses().cloned().collect::<Vec<_>>(),
        [BindPoseMatrix([0.0; 12])]
    );
    assert!(Document::<V900>::new().bind_poses().next().is_none());

    let mut runtime = DynamicDocument::<NoExtensions>::V800(Document::new());
    check_bind_poses(&mut runtime, false);
    assert!(runtime.try_bind_poses().is_err());
    assert!(
        matches!(runtime.try_face_fx(), Err(ValueError::UnsupportedVersion { tag, .. }) if tag == *b"FAFX")
    );
    assert!(
        matches!(runtime.try_popcorn_emitters(), Err(ValueError::UnsupportedVersion { tag, .. }) if tag == *b"CORN")
    );
    assert!(runtime.try_set_face_fx([]).is_err());
    assert!(runtime.try_set_popcorn_emitters([]).is_err());

    let mut runtime = DynamicDocument::<NoExtensions>::V1800(Document::new());
    assert!(runtime.try_face_fx().unwrap().next().is_none());
    assert!(runtime.try_popcorn_emitters().unwrap().next().is_none());
    runtime.try_set_face_fx([]).unwrap();
    runtime.try_set_popcorn_emitters([]).unwrap();
}

#[test]
fn named_setters_move_owned_record_buffers() {
    use wc3::model::formats::mdlx::animation::{Track, ValueKeyframe};
    use wc3::model::formats::mdlx::geometry::Geoset;
    use wc3::model::formats::mdlx::scene::{Bone, Node};

    let geoset = Geoset::<V800>::new([[1.0; 3]], [[0.0; 3]], []).unwrap();
    let vertices = geoset.vertices().as_bytes().as_ptr();
    let mut node = Node::new("Root", 0).unwrap();
    node.translation = Some(
        Track::linear(
            [ValueKeyframe {
                frame: 0,
                value: [2.0; 3],
            }],
            None,
        )
        .unwrap(),
    );
    let keys = node
        .translation
        .as_ref()
        .unwrap()
        .linear_keys()
        .unwrap()
        .as_bytes()
        .as_ptr();
    let bone = Bone::new(node, u32::MAX, u32::MAX);
    let mut model = Document::<V800>::new();
    model.set_geosets([geoset]);
    model.set_bones(vec![bone]);
    assert_eq!(
        model
            .geosets()
            .next()
            .unwrap()
            .vertices()
            .as_bytes()
            .as_ptr(),
        vertices
    );
    assert_eq!(
        model
            .bones()
            .next()
            .unwrap()
            .node
            .translation
            .as_ref()
            .unwrap()
            .linear_keys()
            .unwrap()
            .as_bytes()
            .as_ptr(),
        keys
    );
}

fn edit_repeated_sequences<'data>(model: &mut impl CommonModelAccess<'data>) {
    for (index, sequence) in model.sequences_mut().enumerate() {
        sequence.move_speed = index as f32 + 1.0;
    }
    assert_eq!(
        model
            .sequences()
            .map(|sequence| sequence.move_speed)
            .collect::<Vec<_>>(),
        [1.0, 2.0]
    );
}

#[test]
fn named_mutable_iterators_preserve_repeated_chunks_for_typed_and_dynamic_models() {
    use wc3::model::formats::mdlx::animation::Sequence;
    use wc3::model::formats::mdlx::chunks::{ModelChunk, RawChunk, SequencesChunk, UnknownChunk};

    let mut model = Document::<Extended<V800, NoExtensions>>::new();
    model
        .chunks
        .push(SequencesChunk::new(vec![Sequence::new("First", [0, 10]).unwrap()]).into());
    model.chunks.push(ModelChunk::Unknown(
        UnknownChunk::new(RawChunk::new(*b"TEST", vec![1, 2, 3])).unwrap(),
    ));
    model
        .chunks
        .push(SequencesChunk::new(vec![Sequence::new("Second", [11, 20]).unwrap()]).into());
    let tags: Vec<_> = model.chunks.iter().map(|chunk| chunk.tag()).collect();
    edit_repeated_sequences(&mut model);
    assert_eq!(
        model
            .chunks
            .iter()
            .map(|chunk| chunk.tag())
            .collect::<Vec<_>>(),
        tags
    );
    let mut dynamic = DynamicDocument::<NoExtensions>::V800(model);
    edit_repeated_sequences(&mut dynamic);
    visit_document!(&dynamic, |model| assert_eq!(
        model
            .chunks
            .iter()
            .map(|chunk| chunk.tag())
            .collect::<Vec<_>>(),
        tags
    ));
    visit_document!(&mut dynamic, |typed| typed.set_global_sequences(&[42]));
    assert_eq!(dynamic.global_sequences(), [42]);
    assert!(dynamic.try_bind_poses_mut().is_err());
    assert!(dynamic.try_face_fx_mut().is_err());
    assert!(dynamic.try_popcorn_emitters_mut().is_err());
}

#[test]
fn checked_mutable_accessors_edit_supported_records_and_reject_before_iteration() {
    use std::iter::once_with;
    use wc3::model::formats::mdlx::emitters::PopcornEmitter;
    use wc3::model::formats::mdlx::scene::{FaceFx, Node};

    let mut model = DynamicDocument::<NoExtensions>::V1800(Document::new());
    model
        .try_set_bind_poses([BindPoseMatrix([0.0; 12])])
        .unwrap();
    model.try_bind_poses_mut().unwrap().next().unwrap().0[0] = 2.0;
    assert_eq!(model.try_bind_poses().unwrap().next().unwrap().0[0], 2.0);
    model
        .try_set_face_fx([FaceFx::new("Face", "face.facefx").unwrap()])
        .unwrap();
    model
        .try_face_fx_mut()
        .unwrap()
        .next()
        .unwrap()
        .name
        .set_text("Changed")
        .unwrap();
    assert_eq!(
        model.try_face_fx().unwrap().next().unwrap().name.text(),
        "Changed"
    );
    model
        .try_set_popcorn_emitters([
            PopcornEmitter::new(Node::new("Emitter", 0).unwrap(), "", "").unwrap(),
        ])
        .unwrap();
    model
        .try_popcorn_emitters_mut()
        .unwrap()
        .next()
        .unwrap()
        .node
        .object_id = 1;
    assert_eq!(
        model
            .try_popcorn_emitters()
            .unwrap()
            .next()
            .unwrap()
            .node
            .object_id,
        1
    );

    let mut old = Document::<V800>::new();
    assert!(old
        .try_set_bind_poses(once_with(|| panic!("unsupported input must not be read")))
        .is_err());
    assert!(old.try_bind_poses_mut().is_err());
    assert!(old.try_face_fx_mut().is_err());
    assert!(old.try_popcorn_emitters_mut().is_err());
    assert_eq!(old.chunks.len(), 1);
}
