//! Prepared sequence-name activation rules.
#[derive(Clone)]
pub(super) struct VisibilityGuide {
    default: bool,
    exceptions: Vec<String>,
}
impl Default for VisibilityGuide {
    fn default() -> Self {
        Self::compile("")
    }
}
impl VisibilityGuide {
    pub fn compile(source: &str) -> Self {
        let mut always = None;
        let mut enabled = Vec::new();
        let mut disabled = Vec::new();
        for token in source
            .split(',')
            .map(str::trim)
            .filter(|token| !token.is_empty())
        {
            let (name, value) = token
                .split_once('=')
                .map_or((token, true), |(name, value)| {
                    (name.trim(), value.trim().eq_ignore_ascii_case("on"))
                });
            let name = name.to_ascii_lowercase();
            if name.is_empty() {
                continue;
            }
            if name == "always" {
                always = Some(value);
            } else if value {
                enabled.push(name);
            } else {
                disabled.push(name);
            }
        }
        let default = always.unwrap_or(enabled.is_empty());
        Self {
            default,
            exceptions: if default { disabled } else { enabled },
        }
    }
    pub fn visible(&self, name: &str) -> bool {
        // Native animation names are ASCII. Avoid allocating a lowercase sample name.
        let matched = self.exceptions.iter().any(|pattern| {
            name.as_bytes()
                .windows(pattern.len())
                .any(|window| window.eq_ignore_ascii_case(pattern.as_bytes()))
        });
        self.default != matched
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sequence_rules_keep_defaults_and_case_insensitive_substrings() {
        for (guide, name, visible) in [
            ("", "Stand", true),
            ("Attack, Death=on", "Stand", false),
            ("Attack, Death=on", "Spell Attack One", true),
            ("attack=off", "Stand", true),
            ("attack=off", "ATTACK 2", false),
            ("always=on, death=off", "Death", false),
            ("always=off, attack=on", "Attack", true),
            ("always=off, attack=on", "Stand", false),
        ] {
            assert_eq!(
                VisibilityGuide::compile(guide).visible(name),
                visible,
                "{guide}/{name}"
            );
        }
    }
}
