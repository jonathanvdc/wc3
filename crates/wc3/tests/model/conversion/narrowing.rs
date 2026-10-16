use std::borrow::Cow;
use wc3::model::conversion::ImportOptions;
use wc3::model::formats::mdlx::{self, mdl::Read as _, mdx::Read as _, mdx::Write as _};
use wc3::model::io;
use wc3::model::profiles::{Extends, Mdlx, Narrows, Universal, Wmo, M2, M3};
use wc3::model::{
    AppearanceSchema, DrawAppearanceSchema, DrawLooks, Look, MdlxImageResource, Model, SingleLook,
    Texture, UniversalAppearance,
};

#[test]
fn native_widening_and_narrowing_preserves_identities_and_borrowed_geometry() {
    let source =
        mdlx::Document::<mdlx::V800>::decode_mdl(include_str!("../../fixtures/mdl/quad_model.mdl"))
            .unwrap();
    let bytes = source.encode_mdx().unwrap();
    let source = mdlx::Document::<mdlx::V800>::decode_mdx(&bytes).unwrap();
    let asset = io::mdlx::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let model_id = asset.id();
    let mesh_id = asset.meshes().iter().next().unwrap().0;
    let positions = asset.mesh(mesh_id).unwrap().positions().clone();
    assert!(!positions.is_owned());

    let asset = asset.widen::<Universal>().try_narrow::<Mdlx>().unwrap();
    assert_eq!(asset.id(), model_id);
    let restored = asset.mesh(mesh_id).unwrap().positions();
    assert!(!restored.is_owned());
    assert_eq!(
        restored.iter().collect::<Vec<_>>(),
        positions.iter().collect::<Vec<_>>()
    );
}

#[test]
fn resource_narrowing_moves_owned_paths_without_reallocating() {
    let mut builder = Model::<Mdlx>::builder("owned resource");
    let path: Cow<'_, [u8]> = Cow::Owned(b"owned.blp".to_vec());
    let pointer = path.as_ptr();
    let resource = builder.add_image_resource(MdlxImageResource::Path(path));
    let id = builder.add_texture(Texture::new(resource)).unwrap();
    let asset = builder
        .finish()
        .unwrap()
        .widen::<Universal>()
        .try_narrow::<Mdlx>()
        .unwrap();
    let MdlxImageResource::Path(path) = asset
        .image_resource(asset.texture(id).unwrap().resource)
        .unwrap()
    else {
        unreachable!()
    };
    assert_eq!(path.as_ptr(), pointer);
    assert!(matches!(path, Cow::Owned(_)));
}

#[test]
fn narrowing_rejects_foreign_payloads_and_missing_required_controls() {
    let asset = Model::<Mdlx>::builder("Warcraft III")
        .finish()
        .unwrap()
        .widen::<Universal>();
    assert!(asset.try_narrow::<Wmo>().is_err());
    let asset = Model::<Universal>::builder("no profile controls")
        .finish()
        .unwrap();
    assert!(asset.try_narrow::<Mdlx>().is_err());
}

#[test]
fn shared_single_appearances_narrow_by_capability_and_preserve_values() {
    let mut builder = Model::<Mdlx>::builder("single appearance");
    let id = builder.model().default_look().unwrap();
    builder
        .set_look(
            id,
            Look {
                name: Cow::Borrowed("shared"),
                bindings: Vec::new(),
            },
        )
        .unwrap();
    builder.set_default_look(Some(id)).unwrap();
    let model = builder.finish().unwrap();
    let source = model.appearance().clone();
    let widened = [
        <Universal as Extends<Mdlx>>::appearance(source.clone()),
        <Universal as Extends<M3>>::appearance(source.clone()),
        <Universal as Extends<Wmo>>::appearance(source),
    ];
    for appearance in widened {
        assert!(matches!(appearance, UniversalAppearance::Single(_)));
        let projections = [
            <Mdlx as Narrows<Universal>>::appearance(appearance.clone()).unwrap(),
            <M3 as Narrows<Universal>>::appearance(appearance.clone()).unwrap(),
            <Wmo as Narrows<Universal>>::appearance(appearance.clone()).unwrap(),
        ];
        for value in projections {
            assert_eq!(value.default_look(), Some(id));
            assert_eq!(value.look(id).unwrap().name, "shared");
            assert!(matches!(value.look(id).unwrap().name, Cow::Borrowed(_)));
        }
        assert!(<M2 as Narrows<Universal>>::appearance(appearance).is_err());
    }
}

#[test]
fn universal_draw_appearances_support_m2_without_a_source_tag() {
    let appearance = UniversalAppearance::default();
    assert!(matches!(appearance, UniversalAppearance::Draw(_)));
    assert!(<M2 as Narrows<Universal>>::appearance(appearance.clone()).is_ok());
    assert!(<Mdlx as Narrows<Universal>>::appearance(appearance.clone()).is_err());
    assert!(<M3 as Narrows<Universal>>::appearance(appearance.clone()).is_err());
    assert!(<Wmo as Narrows<Universal>>::appearance(appearance).is_err());
    assert!(matches!(
        <Universal as Extends<M2>>::appearance(DrawLooks::default()),
        UniversalAppearance::Draw(_)
    ));
}

#[test]
fn single_appearance_limit_survives_widening() {
    let mut builder = Model::<Universal>::builder("look identities");
    let foreign = builder.add_look(Look::default()).unwrap();
    let mut appearance = <Universal as Extends<M3>>::appearance(SingleLook::default());
    let sole = appearance.default_look().unwrap();
    assert!(appearance.add_look(foreign, Look::default()).is_err());
    assert_eq!(appearance.looks().len(), 1);
    appearance.set_default_look(Some(sole)).unwrap();
    assert!(appearance.remove_look(sole).is_none());
    assert!(appearance.set_default_look(None).is_err());
    assert!(appearance.set_default_look(Some(foreign)).is_err());
    assert_eq!(appearance.default_look(), Some(sole));
    appearance.validate().unwrap();
}
