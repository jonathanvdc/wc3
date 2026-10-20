//! Original color-space parameter conversions without gamma transfer.
pub(super) fn convert(channels: [f32; 4], hsv_to_rgb: bool) -> Result<[f32; 4], &'static str> {
    if channels.iter().any(|value| !value.is_finite()) {
        return Err("color conversion requires finite components");
    }
    let [a, b, c, alpha] = channels;
    let color = if hsv_to_rgb {
        let hue = a.rem_euclid(1.) * 6.;
        let saturation = b.clamp(0., 1.);
        let value = c.max(0.);
        let chroma = value * saturation;
        let secondary = chroma * (1. - (hue.rem_euclid(2.) - 1.).abs());
        let base = value - chroma;
        let components = match hue as u32 {
            0 => [chroma, secondary, 0.],
            1 => [secondary, chroma, 0.],
            2 => [0., chroma, secondary],
            3 => [0., secondary, chroma],
            4 => [secondary, 0., chroma],
            _ => [chroma, 0., secondary],
        };
        [
            components[0] + base,
            components[1] + base,
            components[2] + base,
            alpha,
        ]
    } else {
        let maximum = a.max(b).max(c);
        let minimum = a.min(b).min(c);
        let range = maximum - minimum;
        if range <= 1e-6 {
            [0., 0., maximum, alpha]
        } else {
            let sector = if maximum == a {
                (b - c) / range
            } else if maximum == b {
                (c - a) / range + 2.
            } else {
                (a - b) / range + 4.
            };
            [sector.rem_euclid(6.) / 6., range / maximum, maximum, alpha]
        }
    };
    if color.iter().any(|component| !component.is_finite()) {
        return Err("color conversion produced nonfinite components");
    }
    Ok(color)
}

#[cfg(test)]
#[path = "color_tests.rs"]
mod tests;
