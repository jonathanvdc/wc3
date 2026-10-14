//! Open-ended authoring entries preserve order and permit repeated property names.
use std::borrow::Cow;
use std::mem::replace;

/// One named authoring entry with an integer and textual annotation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MetadataEntry<'a> {
    /// Name used by the authoring tool or application.
    pub name: Cow<'a, str>,
    /// Integer value associated with the name.
    pub value: i64,
    /// Text associated with the same entry; empty when no annotation is authored.
    pub text: Cow<'a, str>,
}

/// Authored entries in their original order; names need not be unique.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Metadata<'a> {
    entries: Vec<MetadataEntry<'a>>,
}
impl<'a> Metadata<'a> {
    /// Creates a collection without interpreting application-specific property names.
    pub fn new(entries: Vec<MetadataEntry<'a>>) -> Self {
        Self { entries }
    }
    /// Borrows entries in authored order.
    pub fn entries(&self) -> &[MetadataEntry<'a>] {
        &self.entries
    }
    /// Appends an entry, including when the name already occurs.
    pub fn push(&mut self, entry: MetadataEntry<'a>) {
        self.entries.push(entry);
    }
    /// Borrows all entries with the requested name in authored order.
    pub fn named<'s>(&'s self, name: &'s str) -> impl Iterator<Item = &'s MetadataEntry<'a>> {
        self.entries.iter().filter(move |entry| entry.name == name)
    }
    /// Replaces an entry by its position, returning the previous value when present.
    pub fn replace(&mut self, index: usize, entry: MetadataEntry<'a>) -> Option<MetadataEntry<'a>> {
        let previous = self.entries.get_mut(index)?;
        Some(replace(previous, entry))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn duplicate_names_and_text_remain_independently_editable() {
        let mut entries = Metadata::new(vec![
            MetadataEntry {
                name: "tag".into(),
                value: 1,
                text: "first".into(),
            },
            MetadataEntry {
                name: "tag".into(),
                value: 2,
                text: "".into(),
            },
        ]);
        assert_eq!(
            entries
                .named("tag")
                .map(|entry| entry.value)
                .collect::<Vec<_>>(),
            [1, 2]
        );
        entries
            .replace(
                1,
                MetadataEntry {
                    name: "tag".into(),
                    value: 3,
                    text: "second".into(),
                },
            )
            .unwrap();
        assert_eq!(entries.entries()[0].text, "first");
        assert_eq!(entries.entries()[1].value, 3);
    }
}
