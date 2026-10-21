//! Established native shader coordinate routes; unknown programmes retain their tables.
#[derive(Clone, Copy)]
pub(super) struct Route {
    pub source: i16,
    pub transform: Option<usize>,
}
const T1: Route = Route {
    source: 0,
    transform: Some(0),
};
// Illumination independently transforms the same raw UV0 in its second slot.
const T1_SECOND: Route = Route {
    source: 0,
    transform: Some(1),
};
const T2: Route = Route {
    source: 1,
    transform: Some(1),
};
const T2_THIRD: Route = Route {
    source: 1,
    transform: Some(2),
};
const ENV: Route = Route {
    source: -1,
    transform: None,
};

pub(super) fn routes(shader: u16, count: usize) -> Option<&'static [Route]> {
    let routes: &[Route] = if shader & 0x8000 == 0 {
        match (
            count,
            shader & 0x80 != 0,
            shader & 8 != 0,
            shader & 0x4000 != 0,
        ) {
            (1, true, _, _) => &[ENV],
            // A single T2 programme addresses a second transform outside its input
            // count; retain that native table until export can allocate it separately.
            (1, false, _, true) => return None,
            (1, false, _, false) => &[T1],
            (2, true, true, _) => &[ENV, ENV],
            (2, true, false, _) => &[ENV, T1],
            (2, false, true, _) => &[T1, ENV],
            (2, false, false, true) => &[T1, T2],
            (2, false, false, false) => &[T1, T1],
            _ => return None,
        }
    } else {
        match shader & 0x7fff {
            0..=2 | 4 | 7 | 8 | 11..=13 | 16 => &[T1, ENV],
            3 | 9 | 17 => &[T1, ENV, T1],
            5 | 6 | 10 | 14 | 23 => &[T1, T1],
            15 | 21 | 27 | 33 | 35 => &[T1, T2],
            18 => &[T1, T1, T1],
            19 | 32 | 34 => &[T1],
            20 | 24 => &[T1, ENV, T2_THIRD],
            22 => &[T1, T1, T1, T2],
            25 => &[ENV],
            26 | 28 => &[T1, T2, T1],
            29 => &[T1, T1_SECOND],
            _ => return None,
        }
    };
    (routes.len() == count).then_some(routes)
}
