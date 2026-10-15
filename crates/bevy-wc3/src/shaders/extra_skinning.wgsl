// Additional weighted joints shared by main, shadow, and normal prepasses.
// Existing license: MIT OR Apache-2.0.
#import bevy_pbr::{mesh_bindings::mesh, skinning}

#ifdef SKINNED
fn extra_skin_model(indices: vec4<u32>, weights: vec4<f32>, extra_indices: vec4<u32>, extra_weights: vec4<f32>, instance_index: u32) -> mat4x4<f32> {
    var result = skinning::skin_model(indices, weights, instance_index);
#ifdef SKINS_USE_UNIFORM_BUFFERS
    result += extra_weights.x * skinning::joint_matrices.data[extra_indices.x]
        + extra_weights.y * skinning::joint_matrices.data[extra_indices.y]
        + extra_weights.z * skinning::joint_matrices.data[extra_indices.z]
        + extra_weights.w * skinning::joint_matrices.data[extra_indices.w];
#else
    let skin_index = mesh[instance_index].current_skin_index;
    result += extra_weights.x * skinning::joint_matrices[skin_index + extra_indices.x]
        + extra_weights.y * skinning::joint_matrices[skin_index + extra_indices.y]
        + extra_weights.z * skinning::joint_matrices[skin_index + extra_indices.z]
        + extra_weights.w * skinning::joint_matrices[skin_index + extra_indices.w];
#endif
    return result;
}

fn extra_skin_prev_model(indices: vec4<u32>, weights: vec4<f32>, extra_indices: vec4<u32>, extra_weights: vec4<f32>, instance_index: u32) -> mat4x4<f32> {
    var result = skinning::skin_prev_model(indices, weights, instance_index);
#ifdef SKINS_USE_UNIFORM_BUFFERS
    result += extra_weights.x * skinning::prev_joint_matrices.data[extra_indices.x]
        + extra_weights.y * skinning::prev_joint_matrices.data[extra_indices.y]
        + extra_weights.z * skinning::prev_joint_matrices.data[extra_indices.z]
        + extra_weights.w * skinning::prev_joint_matrices.data[extra_indices.w];
#else
    let skin_index = mesh[instance_index].current_skin_index;
    result += extra_weights.x * skinning::prev_joint_matrices[skin_index + extra_indices.x]
        + extra_weights.y * skinning::prev_joint_matrices[skin_index + extra_indices.y]
        + extra_weights.z * skinning::prev_joint_matrices[skin_index + extra_indices.z]
        + extra_weights.w * skinning::prev_joint_matrices[skin_index + extra_indices.w];
#endif
    return result;
}
#endif
