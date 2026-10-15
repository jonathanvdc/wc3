use wc3::model::formats::m3;
use wc3::model::formats::m3::records::{ModelRecord, U32, V23};
use wc3::model::formats::m3::{
    AnimationFile, Bundle, Container, Document, Envelope, Md33, Md34, Ref,
};
fn model<E: Envelope>(primary: u32, additional: &[u32]) -> Document<'static, E> {
    let mut document = Container::new();
    let hashes: Vec<_> = additional
        .iter()
        .map(|value| U32 { value: *value })
        .collect();
    let hashes_id = document.append(&hashes).unwrap();
    let root = ModelRecord::<V23> {
        m3a_anim_hash: primary,
        m3a_anim_hashes: Ref::from_section(hashes.len() as u32, hashes_id.index()),
        ..ModelRecord::<V23>::default()
    };
    let id = document.append(&[root]).unwrap();
    document.set_root(Ref::<ModelRecord<V23>>::from_section(1, id.index()));
    Document::from_container(document).unwrap()
}
#[test]
fn external_animation_closure_retains_native_envelopes_and_hashes() {
    let base = model::<Md34>(100, &[200]).write().unwrap();
    let first = model::<Md33>(100, &[200]).write().unwrap();
    let second = model::<Md34>(200, &[100]).write().unwrap();
    let mut requested = Vec::new();
    let bundle = Bundle::<Md34>::read(&base, |hash| {
        requested.push(hash);
        Ok(match hash {
            100 => Some(first.as_slice()),
            200 => Some(second.as_slice()),
            _ => None,
        })
    })
    .unwrap();
    assert_eq!(requested, [100, 200]);
    assert!(matches!(
        bundle.animation(100),
        Some(AnimationFile::Md33(_))
    ));
    let encoded = bundle.write().unwrap();
    assert_eq!(&encoded.animations[&100][..4], b"33DM");
    assert_eq!(&encoded.animations[&200][..4], b"43DM");
    let decoded = Bundle::<Md34>::read(&encoded.model, |hash| {
        Ok(encoded.animations.get(&hash).map(Vec::as_slice))
    })
    .unwrap();
    decoded.validate_complete().unwrap();
}
#[test]
fn missing_or_wrong_animation_bindings_are_rejected() {
    let base = model::<Md34>(100, &[]).write().unwrap();
    assert!(Bundle::<Md34>::read(&base, |_| Ok(None)).is_err());
    let wrong = model::<Md34>(999, &[]).write().unwrap();
    assert!(Bundle::<Md34>::read(&base, |_| Ok(Some(wrong.as_slice()))).is_err());
}
#[test]
fn explicit_zero_authored_hash_association_is_supported() {
    let mut bundle = Bundle::from_model(model::<Md34>(100, &[]));
    bundle
        .insert_animation(100, model::<Md34>(0, &[]).into())
        .unwrap();
    bundle.validate_complete().unwrap();
    assert_eq!(bundle.write().unwrap().animations.len(), 1);
}
#[test]
fn explicitly_registered_cycles_are_valid_independent_associations() {
    let mut bundle = Bundle::from_model(model::<Md34>(0, &[]));
    bundle
        .insert_animation(100, model::<Md34>(100, &[200]).into())
        .unwrap();
    bundle
        .insert_animation(200, model::<Md34>(200, &[100]).into())
        .unwrap();
    assert_eq!(bundle.dependencies().unwrap(), [100, 200]);
    bundle.validate_complete().unwrap();
}

#[test]
fn public_root_dispatch_is_accessible_from_an_application() {
    let model = model::<Md34>(100, &[]);
    let root = model.root_record().unwrap();
    let flags = wc3::visit_m3_root!(&root, |native| native.vertex_flags);
    assert_eq!(flags, root.vertex_flags());
}

#[test]
fn native_roots_share_one_name_section_and_record_revisions_remain_independent() {
    use wc3::model::formats::m3::records::{AnimationGroup, Character, V21};
    let mut document = Container::<Md34>::new();
    let characters: Vec<_> = b"shared\0"
        .iter()
        .map(|byte| Character { value: *byte })
        .collect();
    let name = document.append(&characters).unwrap();
    let group = AnimationGroup {
        name: Ref::new(
            characters.len() as u32,
            name.index(),
            m3::records::ReferenceFlags(7),
        ),
        ..AnimationGroup::default()
    };
    let group_id = document.append(&[group]).unwrap();
    let root = ModelRecord::<V21> {
        name: group.name,
        animation_groups: Ref::from_section(1, group_id.index()),
        ..ModelRecord::<V21>::default()
    };
    let root_id = document.append(&[root]).unwrap();
    document.set_root(Ref::<ModelRecord<V21>>::from_section(1, root_id.index()));
    let model = Document::from_container(document).unwrap();
    let a = model
        .container()
        .resolve::<Character>(model.root::<ModelRecord<V21>>().unwrap().name.cast())
        .unwrap();
    let b = model
        .container()
        .resolve::<Character>(
            model
                .container()
                .array::<AnimationGroup>(group_id)
                .unwrap()
                .get(0)
                .unwrap()
                .unwrap()
                .name
                .cast(),
        )
        .unwrap();
    assert_eq!(a.as_bytes().as_ptr(), b.as_bytes().as_ptr());
    let bytes = model.write().unwrap();
    let decoded = Document::<Md34>::read(&bytes).unwrap();
    assert_eq!(decoded.root_record().unwrap().revision(), 21);
    assert_eq!(
        decoded
            .container()
            .section(decoded.container().section_id(group_id.index()).unwrap())
            .unwrap()
            .revision(),
        0
    );
    assert_eq!(
        decoded
            .root::<ModelRecord<V21>>()
            .unwrap()
            .name
            .flags
            .bits(),
        7
    );
}

#[test]
fn creation_profiles_select_defaults_without_converting_existing_roots() {
    use wc3::model::formats::m3::{Beta, HeroesOfTheStorm, StarCraft2};
    let beta = Document::<Md33>::new::<Beta>().unwrap();
    let sc2 = Document::<Md34>::new::<StarCraft2>().unwrap();
    let heroes = Document::<Md34>::new::<HeroesOfTheStorm>().unwrap();
    assert_eq!(beta.root_record().unwrap().revision(), 23);
    assert_eq!(sc2.root_record().unwrap().revision(), 29);
    assert_eq!(heroes.root_record().unwrap().revision(), 30);
    let existing = model::<Md34>(0, &[]).write().unwrap();
    assert_eq!(
        Document::<Md34>::read(&existing)
            .unwrap()
            .root_record()
            .unwrap()
            .revision(),
        23
    );
    let mut bundle = Bundle::from_model(sc2);
    bundle.insert_animation(17, beta.into()).unwrap();
    bundle.insert_animation(19, heroes.into()).unwrap();
    let output = bundle.write().unwrap();
    assert_eq!(&output.animations[&17][..4], b"33DM");
    assert_eq!(
        AnimationFile::read(&output.animations[&19])
            .unwrap()
            .binding_hash()
            .unwrap(),
        0
    );
}

#[test]
fn newly_established_root_revisions_have_exact_common_metadata() {
    use wc3::model::formats::m3::records::{V22, V27};
    let mut document = Container::<Md34>::new();
    let id = document
        .append(&[ModelRecord::<V22> {
            m3a_anim_hash: 22,
            ..ModelRecord::<V22>::default()
        }])
        .unwrap();
    document.set_root(Ref::<ModelRecord<V22>>::from_section(1, id.index()));
    let model = Document::from_container(document).unwrap();
    assert_eq!(model.root_record().unwrap().revision(), 22);
    assert_eq!(model.animation_hashes().unwrap(), [22]);
    let mut document = Container::<Md33>::new();
    let hashes = document.append(&[U32 { value: 28 }]).unwrap();
    let id = document
        .append(&[ModelRecord::<V27> {
            m3a_anim_hash: 27,
            m3a_anim_hashes: Ref::from_section(1, hashes.index()),
            ..ModelRecord::<V27>::default()
        }])
        .unwrap();
    document.set_root(Ref::<ModelRecord<V27>>::from_section(1, id.index()));
    let model = Document::from_container(document).unwrap();
    assert_eq!(model.root_record().unwrap().revision(), 27);
    assert_eq!(model.animation_hashes().unwrap(), [27, 28]);
}
