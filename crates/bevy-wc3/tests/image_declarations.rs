use bevy::asset::AssetPath;
use bevy_wc3::{texture_requests, ResourceResolver};
use wc3::model::materials::mdlx::MaterialStep;
use wc3::model::profiles::mdlx::Material as MdlxMaterial;
use wc3::model::{
    Construct as _, ImageResource, Material, MaterialSurface, Model, ModelMetadata,
    ResourceLocator, Texture,
};

struct FileIds;
impl ResourceResolver for FileIds {
    fn candidates(
        &self,
        resource: &ResourceLocator<'_>,
        _: &AssetPath<'_>,
    ) -> Vec<AssetPath<'static>> {
        let ResourceLocator::FileDataId(id) = resource else {
            panic!("expected a file-data identity");
        };
        vec![AssetPath::from(format!("filedata/{id}.blp"))]
    }
}

#[test]
fn file_data_resolution_uses_the_id_while_requests_retain_the_filename() {
    let mut draft = Model::builder(ModelMetadata::Mdlx(Default::default()));
    let resource = draft.add_image_resource(ImageResource::FileDataId {
        id: 42,
        filename: b"authored.blp".as_slice().into(),
    });
    let step = MaterialStep::construct(draft.values_mut());
    draft
        .values_mut()
        .set(
            step.shader.inputs().binding.image,
            Some(Texture::new(resource)),
        )
        .unwrap();
    draft
        .add_material(Material {
            name: "image request".into(),
            surface: MaterialSurface::from(MdlxMaterial { steps: vec![step] }),
        })
        .unwrap();
    let model = draft.finish().unwrap();
    let requests = texture_requests(&model);
    let request = &requests[0];
    assert!(
        matches!(request.resource, ImageResource::FileDataId { id: 42, filename } if filename.as_ref() == b"authored.blp")
    );
    let paths = FileIds.texture_candidates(0, request, &AssetPath::parse("model.m2"));
    assert_eq!(paths[0].path().to_str(), Some("filedata/42.blp"));
}
