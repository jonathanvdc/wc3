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
