struct ParticleComposition { mode: vec4<u32>, };
@group(#{MATERIAL_BIND_GROUP}) @binding(113) var<uniform> particle: ParticleComposition;
fn compose_particle(a:vec4<f32>,b:vec4<f32>,c:vec4<f32>,tint:vec4<f32>) -> vec4<f32> {
    var rgb=a.rgb*b.rgb;
    if particle.mode.x==2u { rgb*=c.rgb; }
    return vec4(tint.rgb*rgb,tint.a*a.a*b.a*c.a);
}
