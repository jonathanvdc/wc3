use super::*;
use crate::assembly::{Atlas, PixelRect, TextureLayer};
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
    assert_eq!(
        recipe.validate(),
        Err(RecipeError::DuplicateCustomization(3))
    );
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
    assert_eq!(
        recipe.validate(),
        Err(RecipeError::DuplicateEquipment(EquipmentSlot::Chest))
    );
}
#[test]
fn checked_rectangles_reject_overflow_before_asset_work() {
    let definition = CharacterDefinition {
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
    assert_eq!(definition.validate(), Err(RecipeError::InvalidAtlas));
}
