struct Combiner {
    uv_transform: mat3x3<f32>, third_transform: mat3x3<f32>,
    fourth_transform: mat3x3<f32>, additional_coordinates: vec4<u32>,
    coordinates: vec4<u32>, weights: vec4<f32>,
    guild_background: vec4<f32>, guild_emblem: vec4<f32>, guild_border: vec4<f32>,
    mode: vec4<u32>, clip: vec4<f32>,
};
@group(#{MATERIAL_BIND_GROUP}) @binding(100) var<uniform> combiner: Combiner;
@group(#{MATERIAL_BIND_GROUP}) @binding(101) var secondary: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(102) var secondary_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(111) var tertiary: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(112) var tertiary_sampler: sampler;

@group(#{MATERIAL_BIND_GROUP}) @binding(114) var fourth: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(115) var fourth_sampler: sampler;

struct Composed { color: vec4<f32>, emission: vec3<f32>, };
// Surface tint and opacity stay independent of texture masks and additive color.
fn compose(a:vec4<f32>,b:vec4<f32>,c:vec4<f32>,d:vec4<f32>,tint:vec4<f32>) -> Composed {
    let mode=combiner.mode.x;
    let weights=combiner.weights;
    var rgb=a.rgb;
    var alpha=1.0;
    var emission=vec3(0.0);
    switch mode {
        case 1u: {
            let gain=1.0+f32(combiner.mode.w);
            rgb=a.rgb*b.rgb*gain;
            if combiner.mode.y!=0u { alpha*=a.a; }
            if combiner.mode.z!=0u { alpha*=b.a*gain; }
        }
        case 2u,5u,14u: {
            rgb=a.rgb*(2.0*b.rgb*(1.0-a.a)+vec3(a.a));
            if mode==5u {emission=c.rgb*c.a*weights.z;}
            if mode==14u {
                let glow=clamp(c.a*weights.z,0.0,1.0);
                rgb*=1.0-glow;
                emission=c.rgb*glow;
            }
        }
        case 3u,4u,6u,7u,10u,12u: {
            emission=b.rgb*b.a;
            if mode==4u || mode==7u {emission*=1.0-a.a;}
            if mode==10u || mode==12u {emission*=weights.y;}
            if mode==6u || mode==12u {alpha=a.a;}
            if mode==7u {alpha=a.a+b.a*dot(b.rgb,vec3(0.3,0.59,0.11));}
        }
        case 8u: {rgb=mix(mix(a.rgb,b.rgb,b.a),a.rgb,a.a);}
        case 9u: {rgb=mix(2.0*a.rgb*b.rgb,c.rgb,c.a);}
        case 11u: {rgb=a.rgb*(b.rgb*(1.0-a.a)+vec3(a.a));}
        case 13u: {rgb=mix(a.rgb,b.rgb,b.a); emission=a.rgb*a.a*weights.x;}
        case 15u: {rgb=mix(a.rgb,b.rgb,b.a);}
        case 16u,17u: {
            if combiner.mode.y!=0u {
                emission=b.rgb;
                alpha=a.a;
                if mode==16u {alpha+=b.a;}
            } else { emission=b.rgb*b.a; }
        }
        case 18u: {
            let color=mix(mix(a,b,clamp(weights.y,0.0,1.0)),c,clamp(weights.z,0.0,1.0));
            rgb=color.rgb; alpha=color.a;
        }
        case 23u: {
            let color=mix(mix(a,b,clamp(weights.y,0.0,1.0)),c,clamp(weights.z,0.0,1.0));
            rgb=color.rgb; alpha=color.a*d.a;
        }
        case 24u,25u,26u: {
            rgb=a.rgb*mix(combiner.guild_background.rgb,b.rgb*combiner.guild_emblem.rgb,b.a);
            if mode!=25u {rgb=mix(rgb,c.rgb*combiner.guild_border.rgb,c.a);}
            if mode!=26u {alpha=a.a;}
        }
        case 19u: {rgb=mix(mix(2.0*a.rgb*b.rgb,c.rgb,c.a),a.rgb,a.a);}
        case 20u: {emission=b.rgb*(1.0-a.a); alpha=a.a+b.a;}
        case 21u: {alpha=1.0;}
        case 22u: {alpha=a.a;}
        default: {alpha=a.a;}
    }
    return Composed(vec4(tint.rgb*rgb,tint.a*alpha),emission);
}
