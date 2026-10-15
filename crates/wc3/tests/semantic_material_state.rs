use wc3::model::materials::diablo3::RenderParams;
use wc3::model::materials::state::{BlendEquation, DepthState, PassState, Rasterization};
use wc3::model::{Link, ValidateLinks};

#[derive(Clone, Copy, Debug)]
struct Component(u8);
impl ValidateLinks for Component {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        for _ in 0..self.0 {
            visitor(Link::InvalidValue);
        }
    }
}

#[test]
fn custom_draw_state_traverses_all_selected_domains() {
    let state = PassState {
        blend: BlendEquation {
            enabled: Component(1),
            operation: Component(2),
            source: Component(3),
            destination: Component(4),
            constant: Component(5),
        },
        rasterization: Rasterization {
            depth: DepthState {
                test: Component(6),
                write: Component(7),
                bias: Component(8),
            },
            faces: Component(9),
            lighting: Component(10),
            controls: Component(11),
        },
        ordering: Component(12),
    };
    let mut count = 0;
    state.visit_links(&mut |link| count += usize::from(matches!(link, Link::InvalidValue)));
    assert_eq!(count, 78);
}

#[test]
fn explicit_equation_state_checks_depth_bias_without_interpreting_unknown_codes() {
    let mut state = RenderParams::default();
    state.blend.operation.0 = -192;
    state.rasterization.faces.0 = -83;
    let mut invalid = false;
    state.visit_links(&mut |link| invalid |= matches!(link, Link::InvalidValue));
    assert!(!invalid);
    state.rasterization.depth.bias = f32::NAN;
    state.visit_links(&mut |link| invalid |= matches!(link, Link::InvalidValue));
    assert!(invalid);
}
