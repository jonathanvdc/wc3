//! Native world vertex-light preparation, leaving semantic source arrays intact.
use bevy::math::Vec3;
use bevy::math::Vec4;
use std::sync::Arc;
use wc3::model::controls::{world_draw::Lighting, DrawControls, InstanceControls};
use wc3::model::systems::{ModelMetadata, System};
use wc3::model::{Billboard, MeshInstance, Model, PrimitiveRange, Topology, Transform};

// Regional illumination is a material contribution. Bevy scene lights and ambient
// resources retain their ordinary ownership and affect every compatible asset.
pub(crate) fn illumination(
    model: &Model<'_>,
    instance: &MeshInstance<'_>,
    controls: Option<&DrawControls>,
    exterior_material: bool,
) -> (Vec3, u32) {
    let region = match &instance.controls {
        Some(InstanceControls::Wmo(region)) => Some(region.as_ref()),
        _ => None,
    };
    let root_ambient = match model.metadata() {
        ModelMetadata::Wmo(metadata) => metadata.ambient,
        _ => [0; 4],
    };
    let ambient = region
        .and_then(|region| region.ambient_override)
        .unwrap_or(root_ambient);
    let ambient = Vec3::new(ambient[0] as f32, ambient[1] as f32, ambient[2] as f32) / 255.;
    if exterior_material || region.is_some_and(|region| region.options.exterior_lit) {
        return (ambient, 2);
    }
    let class = match controls {
        Some(DrawControls::Wmo(controls)) => Some(controls.lighting),
        _ => None,
    };
    let lighting = match class {
        Some(Lighting::Transition) => 0,
        Some(Lighting::Interior) => 1,
        Some(Lighting::Exterior) => 2,
        _ if region.is_some_and(|region| region.options.interior && !region.options.exterior) => 1,
        _ => 2,
    };
    (ambient, lighting)
}

/// One immutable policy per original world geometry placement. Projection must
/// pass source vertex indices, before seams or draw ranges duplicate vertices.
#[derive(Clone, Debug)]
pub(crate) struct ColorAdjustment {
    base: [u8; 3],
    alpha_only: bool,
    exterior: bool,
    force_exterior: bool,
    transition_end: usize,
    portal_alpha: Arc<[Option<u8>]>,
    diagnostic: Option<&'static str>,
}
impl ColorAdjustment {
    pub(crate) fn new(model: &Model<'_>, instance: &MeshInstance<'_>) -> Option<Self> {
        let ModelMetadata::Wmo(metadata) = model.metadata() else {
            return None;
        };
        let Some(InstanceControls::Wmo(region)) = &instance.controls else {
            return None;
        };
        let mesh = model.mesh(instance.mesh)?;
        let mut transition_end = 0;
        for draw in &instance.draws {
            if !matches!(&draw.controls, Some(DrawControls::Wmo(controls)) if controls.lighting == Lighting::Transition)
            {
                continue;
            }
            let Some(subset) = mesh.subset(draw.subset) else {
                continue;
            };
            let last = subset
                .vertex_window
                .map(|window| window[1] as usize)
                .or_else(|| {
                    let Topology::Triangles(triangles) =
                        mesh.connectivity(subset.selection.connectivity)?
                    else {
                        return None;
                    };
                    let PrimitiveRange::Primitives { first, count } = subset.selection.range else {
                        return None;
                    };
                    triangles
                        .iter()
                        .skip(first as usize)
                        .take(count as usize)
                        .flatten()
                        .max()
                        .map(|index| index as usize)
                });
            if let Some(last) = last {
                transition_end = transition_end.max(last.saturating_add(1));
            }
        }
        transition_end = transition_end.min(mesh.positions().len());
        let mut portals = Vec::new();
        let mut uncertain = false;
        let mut current = Some(instance.node);
        while let Some(id) = current {
            let Some(node) = model.node(id) else {
                uncertain = true;
                break;
            };
            if node.transform.defaults(model.values()) != Transform::default()
                || node.billboard != Billboard::None
                || node.camera_anchored
            {
                uncertain = true;
                break;
            }
            current = node.parent;
        }
        if !metadata.options.no_vertex_attenuation && transition_end > 0 {
            for connection in &region.portals {
                if connection.adjusted_plane || !matches!(connection.side, -1 | 1) {
                    uncertain = true;
                    break;
                }
                let Some(System::Wmo(world)) = model.system(connection.portal.system) else {
                    uncertain = true;
                    break;
                };
                let Some(portal) = world.portals.get(connection.portal.index as usize) else {
                    uncertain = true;
                    break;
                };
                let Some(System::Wmo(neighbors)) = model.system(connection.neighbor.system) else {
                    uncertain = true;
                    break;
                };
                let Some(neighbor) = neighbors.groups.get(connection.neighbor.index as usize)
                else {
                    uncertain = true;
                    break;
                };
                let normal = Vec3::new(portal.plane[0], portal.plane[1], portal.plane[2]);
                let length = normal.length();
                let vertices: Vec<_> = portal.vertices.iter().collect();
                let area = vertices
                    .iter()
                    .zip(vertices.iter().cycle().skip(1))
                    .take(vertices.len())
                    .fold(Vec3::ZERO, |sum, (&a, &b)| sum + a.cross(b));
                if length <= f32::EPSILON
                    || !length.is_finite()
                    || !portal.plane[3].is_finite()
                    || vertices.len() < 3
                    || vertices.iter().any(|v| !v.is_finite())
                    || area.length_squared() <= f32::EPSILON
                    || portal.adjustment.is_some()
                {
                    uncertain = true;
                    break;
                }
                portals.push(PortalBoundary {
                    vertices,
                    normal: normal / length,
                    distance: portal.plane[3] / length,
                    side: connection.side,
                    exterior: neighbor.options.exterior || neighbor.options.exterior_lit,
                });
            }
        }
        let portal_alpha =
            if metadata.options.no_vertex_attenuation || transition_end == 0 || uncertain {
                Arc::from([])
            } else {
                mesh.positions()
                    .iter()
                    .take(transition_end)
                    .map(|position| Some(byte(portal_blend(position, &portals) * 255.)))
                    .collect::<Vec<_>>()
                    .into()
            };
        Some(Self {
            base: if metadata.options.skip_ambient_in_vertex_color {
                [0; 3]
            } else {
                metadata.ambient[..3].try_into().expect("RGB")
            },
            alpha_only: metadata.options.vertex_color_alpha_only,
            exterior: region.options.exterior,
            force_exterior: region.options.interior && region.options.exterior_lit,
            transition_end,
            portal_alpha,
            diagnostic: (uncertain && !metadata.options.no_vertex_attenuation && transition_end > 0).then_some("WMO portal vertex attenuation retains authored alpha because adjusted, ambiguous, or degenerate portal geometry or a transformed group node has no verified interpretation"),
        })
    }
    pub(crate) fn diagnostic(&self) -> Option<&'static str> {
        self.diagnostic
    }
    pub(crate) fn color(&self, vertex: usize, raw: Vec4) -> Vec4 {
        let mut channels = raw.to_array().map(|value| byte(value * 255.));
        let transition = vertex < self.transition_end;
        if let Some(Some(alpha)) = self.portal_alpha.get(vertex) {
            for channel in &mut channels[..3] {
                *channel = byte(*channel as f32 * (1. - *alpha as f32 / 255.));
            }
            channels[3] = *alpha;
        }
        if !self.alpha_only {
            let alpha = channels[3];
            for (channel, ambient) in channels[..3].iter_mut().zip(self.base) {
                *channel = if transition {
                    let lifted = channel.saturating_sub(ambient);
                    byte(lifted as f32 * (1. - alpha as f32 / 255.)) >> 1
                } else {
                    let lit = i32::from(*channel)
                        + ((u32::from(*channel) * u32::from(alpha)) >> 6) as i32
                        - i32::from(ambient);
                    (lit.max(0) >> 1).min(255) as u8
                };
            }
        }
        if !transition {
            channels[3] = if self.exterior { 255 } else { 0 };
        }
        if self.force_exterior {
            channels[3] = 255;
        }
        Vec4::from_array(channels.map(|channel| f32::from(channel) / 255.))
    }
}

fn byte(value: f32) -> u8 {
    value.round_ties_even().clamp(0., 255.) as u8
}

struct PortalBoundary {
    vertices: Vec<Vec3>,
    normal: Vec3,
    distance: f32,
    side: i16,
    exterior: bool,
}
fn portal_blend(position: Vec3, portals: &[PortalBoundary]) -> f32 {
    let mut accumulated = 0.;
    for portal in portals {
        let plane_distance = portal.normal.dot(position) + portal.distance;
        let projected = position - portal.normal * plane_distance;
        let inside = inside_polygon(projected, &portal.vertices, portal.normal);
        let distance = if inside {
            if portal.side == 1 {
                plane_distance
            } else {
                -plane_distance
            }
        } else {
            portal
                .vertices
                .iter()
                .zip(portal.vertices.iter().cycle().skip(1))
                .take(portal.vertices.len())
                .map(|(&a, &b)| segment_distance(position, a, b))
                .fold(f32::INFINITY, f32::min)
        };
        if portal.exterior {
            let fade = 1. - distance.max(0.) * 0.15;
            if fade > 0.001 {
                accumulated += fade;
            }
        } else if distance > -1. && distance < 1. {
            return 0.;
        }
    }
    if accumulated <= 0.001 {
        0.
    } else {
        accumulated.min(1.)
    }
}
fn segment_distance(point: Vec3, a: Vec3, b: Vec3) -> f32 {
    let edge = b - a;
    let t = if edge.length_squared() > f32::EPSILON {
        ((point - a).dot(edge) / edge.length_squared()).clamp(0., 1.)
    } else {
        0.
    };
    point.distance(a + edge * t)
}
fn inside_polygon(point: Vec3, vertices: &[Vec3], normal: Vec3) -> bool {
    let dominant = normal.abs().max_element();
    let axes = if normal.x.abs() == dominant {
        [1, 2]
    } else if normal.y.abs() == dominant {
        [2, 0]
    } else {
        [0, 1]
    };
    let [x, y] = axes;
    let mut inside = false;
    for (&a, &b) in vertices
        .iter()
        .zip(vertices.iter().cycle().skip(1))
        .take(vertices.len())
    {
        if segment_distance(point, a, b) <= 1e-5 {
            return true;
        }
        if (a[y] > point[y]) != (b[y] > point[y]) {
            let crossing = a[x] + (point[y] - a[y]) * (b[x] - a[x]) / (b[y] - a[y]);
            if point[x] < crossing {
                inside = !inside;
            }
        }
    }
    inside
}

#[cfg(test)]
#[path = "world_lighting_tests.rs"]
mod tests;
