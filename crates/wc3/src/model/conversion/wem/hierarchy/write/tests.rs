use super::*;
use crate::model::conversion::wem::hierarchy::{payloads::ReadContext, read};
use crate::model::semantic::profiles::wem::actors::common::{Camera, Feature as CommonFeature};
use crate::model::semantic::profiles::wem::poses::SkinSetup;
use crate::model::semantic::profiles::wem::{Feature, NodeControls};
use crate::model::semantic::{Animated, Billboard, Node};

fn camera() -> Feature<'static> {
    Feature::Common(CommonFeature::Camera(Camera {
        field_of_view: Animated::constant(0.75),
        clipping: [Animated::constant(0.1), Animated::constant(100.)],
        target: None,
        depth_of_field: None,
    }))
}
#[test]
fn legacy_camera_declarations_choose_a_supported_vocabulary_without_inventing_fields() {
    let mut draft = Model::<Wem>::builder("legacy camera");
    let mut parent = Node::named("parent");
    parent.billboard = Billboard::Locked {
        enabled: false,
        x: true,
        y: false,
        z: false,
    };
    let parent = draft.add_node(parent).unwrap();
    let mut child = Node::named("camera");
    child.parent = Some(parent);
    child.transform.translation = [2., 3., 4.];
    let child = draft.add_node(child).unwrap();
    let feature = draft.add_feature(child, camera()).unwrap();
    let model = draft.finish().unwrap();
    let nodes = BTreeMap::from([(parent, 0), (child, 1)]);
    let features = BTreeMap::from([(feature, 1)]);
    let mut document = RawDocument::new();
    let reference = write(
        &mut document,
        &model,
        &Context {
            nodes: &nodes,
            meshes: &BTreeMap::new(),
            models: &BTreeMap::new(),
            features: &features,
            textures: &BTreeMap::new(),
            slots: &BTreeMap::new(),
            sections: &BTreeMap::new(),
        },
    )
    .unwrap();
    assert_eq!(
        document
            .referenced_section(reference)
            .unwrap()
            .unwrap()
            .revision(),
        3
    );
    let source = read(&document, reference).unwrap();
    let rebuilt = source[1]
        .read_base(&document, &[parent, child], &model.controls().poses)
        .unwrap();
    assert_eq!(rebuilt.parent, Some(parent));
    assert_eq!(rebuilt.transform.translation, [2., 3., 4.]);
    assert!(rebuilt.controls.setup.skin.is_none());
    assert!(rebuilt.controls.setup.rig.is_none());
    let decoded = source[1]
        .read_payload(&ReadContext {
            document: &document,
            nodes: &[parent, child],
            meshes: &[],
            models: &[],
            features: &features.iter().map(|(id, index)| (*index, *id)).collect(),
            slots: &[],
            textures: &[],
            sections: &[],
        })
        .unwrap();
    let Some(Feature::Common(CommonFeature::Camera(decoded))) = decoded.feature else {
        panic!("camera expected")
    };
    assert!(decoded.target.is_none());
    assert!(decoded.depth_of_field.is_none());
    assert_eq!(decoded.field_of_view.default, 0.75);
    let rebuilt = source[0]
        .read_base(&document, &[parent, child], &model.controls().poses)
        .unwrap();
    assert_eq!(
        rebuilt.billboard,
        Billboard::Locked {
            enabled: false,
            x: true,
            y: false,
            z: false
        }
    );
}
#[test]
fn incompatible_declaration_presence_is_rejected_before_publication() {
    let mut draft = Model::<Wem>::builder("incompatible fields");
    let mut node = Node::<Wem>::named("camera");
    node.controls = NodeControls::default();
    node.controls.setup.skin = Some(SkinSetup::default());
    let id = draft.add_node(node).unwrap();
    draft.add_feature(id, camera()).unwrap();
    assert!(draft
        .finish()
        .unwrap_err()
        .to_string()
        .contains("nodes.capabilities"));
}
