use super::*;
use crate::assembly::{Atlas, AtlasError, PixelRect, TextureLayer};
use wc3::model::{ImageResource, ResourceLocator};
#[test]
fn conflicting_customization_and_equipment_are_rejected_before_resolution() {
    let mut recipe = CharacterRecipe {
        race: 4,
        sex: Sex::Female,
        customization: vec![
            Customization {
                option: 3,
                choice: 5,
            },
            Customization {
                option: 3,
                choice: 6,
            },
        ],
        equipment: vec![],
    };
    assert!(matches!(
        recipe.validate(),
        Err(RecipeError::DuplicateCustomization(3))
    ));
    recipe.customization.pop();
    recipe.equipment = vec![
        Equipment {
            slot: EquipmentSlot::Chest,
            display: 7,
        },
        Equipment {
            slot: EquipmentSlot::Chest,
            display: 8,
        },
    ];
    assert!(matches!(
        recipe.validate(),
        Err(RecipeError::DuplicateEquipment(EquipmentSlot::Chest))
    ));
}
#[test]
fn checked_rectangles_reject_overflow_before_asset_work() {
    let definition = CharacterDefinition {
        bone_sets: Default::default(),
        model: ResourceLocator::FileDataId(1),
        geosets: GeosetSelection::default(),
        parts: vec![],
        textures: BTreeMap::from([(
            1,
            TextureReplacement::Atlas {
                atlas: Atlas {
                    width: 10,
                    height: 10,
                    clear: [0; 4],
                },
                layers: vec![TextureLayer {
                    resource: ImageResource::file_data_id(2),
                    source: None,
                    destination: PixelRect {
                        x: u32::MAX,
                        y: 0,
                        width: 2,
                        height: 1,
                    },
                    opacity: 255,
                    blend: Default::default(),
                }],
            },
        )]),
    };
    let error = definition.validate().unwrap_err();
    assert!(matches!(
        &error,
        RecipeError::InvalidAtlas {
            part: None,
            category: 1,
            ..
        }
    ));
    let source = error.source().unwrap().source().unwrap();
    assert_eq!(
        source.downcast_ref::<AtlasError>(),
        Some(&AtlasError::Rectangle)
    );
}

#[test]
fn database_errors_remain_typed_without_thread_safety_bounds() {
    use std::io::{Error as IoError, ErrorKind};
    use std::rc::Rc;
    #[derive(Debug)]
    struct ProviderError(Rc<IoError>);
    impl Display for ProviderError {
        fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
            Display::fmt(&self.0, f)
        }
    }
    impl Error for ProviderError {}
    struct Database(Rc<IoError>);
    impl CharacterDatabase for Database {
        type Error = ProviderError;
        fn resolve(&self, _: &CharacterRecipe) -> Result<CharacterDefinition, Self::Error> {
            Err(ProviderError(self.0.clone()))
        }
    }
    let original = Rc::new(IoError::new(
        ErrorKind::PermissionDenied,
        "catalog unavailable",
    ));
    let recipe = CharacterRecipe {
        race: 1,
        sex: Sex::Female,
        customization: vec![],
        equipment: vec![],
    };
    let error = recipe.resolve(&Database(original.clone())).unwrap_err();
    let provider = error
        .source()
        .unwrap()
        .downcast_ref::<ProviderError>()
        .unwrap();
    assert!(Rc::ptr_eq(&original, &provider.0));
}
