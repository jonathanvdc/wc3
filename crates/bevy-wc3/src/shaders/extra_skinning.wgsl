// Additional weighted joints shared by main, shadow, and normal prepasses.
// Existing license: MIT OR Apache-2.0.
#import bevy_pbr::{mesh_bindings::mesh, skinning}

#ifdef SKINNED
// Skin normals and tangent handedness use the complete weighted transform.
// Singular transforms have no defined normal; keep their output finite.
fn skin_finite_unit(value: vec3<f32>) -> vec3<f32> {
    let scale = max(max(abs(value.x), abs(value.y)), abs(value.z));
    if !(scale > 0.0 && scale <= 3.402823e38) {
        return vec3<f32>(0.0);
    }
    let scaled = value / scale;
    return scaled * inverseSqrt(dot(scaled, scaled));
}

fn skin_determinant(transform: mat4x4<f32>) -> f32 {
    return dot(transform[0].xyz, cross(transform[1].xyz, transform[2].xyz));
}

fn skin_normal_finite(transform: mat4x4<f32>, normal: vec3<f32>) -> vec3<f32> {
    let determinant = skin_determinant(transform);
    if !(abs(determinant) > 1.0e-12 && abs(determinant) <= 3.402823e38) {
        return vec3<f32>(0.0);
    }
    let cofactor_normal = cross(transform[1].xyz, transform[2].xyz) * normal.x
        + cross(transform[2].xyz, transform[0].xyz) * normal.y
        + cross(transform[0].xyz, transform[1].xyz) * normal.z;
    return skin_finite_unit(cofactor_normal / determinant);
}

fn skin_tangent_finite(transform: mat4x4<f32>, tangent: vec4<f32>) -> vec4<f32> {
    let direction = skin_finite_unit((transform * vec4<f32>(tangent.xyz, 0.0)).xyz);
    let handedness = select(1.0, -1.0, skin_determinant(transform) < 0.0);
    return vec4<f32>(direction, tangent.w * handedness);
}

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
