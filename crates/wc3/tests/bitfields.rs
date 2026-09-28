use wc3::model::animation::{GeosetAnimationFlags, SequenceFlags};
use wc3::model::materials::{LayerShadingFlags, MaterialRenderFlags, TextureFlags};
use wc3::model::scene::NodeFlags;

macro_rules! check_fields {
    ($name:ident, $(($get:ident, $set:ident, $bit:literal)),+ $(,)?) => {{
        let mut flags = $name(1 << 31);
        $(
            assert!(!flags.$get());
            flags.$set(true);
            assert!(flags.$get());
            assert_eq!(flags.bits(), (1 << 31) | (1 << $bit));
            flags.$set(false);
            assert_eq!(flags.bits(), 1 << 31);
        )+
    }};
}

#[test]
fn named_fields_preserve_unknown_bits() {
    check_fields!(
        MaterialRenderFlags,
        (constant_color, set_constant_color, 0),
        (sort_primitives_far_z, set_sort_primitives_far_z, 4),
        (full_resolution, set_full_resolution, 5),
    );
    check_fields!(
        TextureFlags,
        (wrap_width, set_wrap_width, 0),
        (wrap_height, set_wrap_height, 1),
    );
    check_fields!(
        LayerShadingFlags,
        (unshaded, set_unshaded, 0),
        (sphere_env_map, set_sphere_env_map, 1),
        (two_sided, set_two_sided, 4),
        (unfogged, set_unfogged, 5),
        (no_depth_test, set_no_depth_test, 6),
        (no_depth_set, set_no_depth_set, 7),
        (unlit, set_unlit, 8),
    );
    check_fields!(SequenceFlags, (non_looping, set_non_looping, 0),);
    check_fields!(
        GeosetAnimationFlags,
        (drop_shadow, set_drop_shadow, 0),
        (color, set_color, 1),
    );
    check_fields!(
        NodeFlags,
        (dont_inherit_translation, set_dont_inherit_translation, 0),
        (dont_inherit_rotation, set_dont_inherit_rotation, 1),
        (dont_inherit_scaling, set_dont_inherit_scaling, 2),
        (billboarded, set_billboarded, 3),
        (billboard_lock_x, set_billboard_lock_x, 4),
        (billboard_lock_y, set_billboard_lock_y, 5),
        (billboard_lock_z, set_billboard_lock_z, 6),
        (camera_anchored, set_camera_anchored, 7),
        (bone, set_bone, 8),
        (light, set_light, 9),
        (event_object, set_event_object, 10),
        (attachment, set_attachment, 11),
        (particle_emitter, set_particle_emitter, 12),
        (collision_shape, set_collision_shape, 13),
        (ribbon_emitter, set_ribbon_emitter, 14),
        (
            emitter_uses_mdl_or_unshaded,
            set_emitter_uses_mdl_or_unshaded,
            15
        ),
        (
            emitter_uses_tga_or_sort_far_z,
            set_emitter_uses_tga_or_sort_far_z,
            16
        ),
        (line_emitter, set_line_emitter, 17),
        (unfogged, set_unfogged, 18),
        (model_space, set_model_space, 19),
        (xy_quad, set_xy_quad, 20),
    );
}
