//! Macro for track groups used by MDX records.
macro_rules! track_group {
    ($vis:vis enum $group:ident { $($variant:ident : $kind:ident),+ $(,)? }) => {
        #[derive(Clone, Debug, PartialEq)]
        $vis enum $group {
            $( $variant($crate::animation::AnimationTrack<$crate::animation::$kind>), )+
        }
        impl $group {
            pub fn tag(&self) -> $crate::animation::TrackTag {
                match self { $(Self::$variant(_) => <$crate::animation::$kind as $crate::animation::TrackKind>::TAG_KIND,)+ }
            }
            pub fn accepts_bytes(tag: $crate::Tag) -> bool {
                false $(|| tag == <$crate::animation::$kind as $crate::animation::TrackKind>::TAG)+
            }
        }
        impl $crate::Readable for $group {
            fn read_from(cursor: &mut $crate::Cursor<'_>) -> Result<Self, $crate::DecodeError> {
                let offset = cursor.absolute_position();
                let tag: $crate::Tag = cursor.peek_exact(4)?.try_into().expect("four-byte tag");
                $(if tag == <$crate::animation::$kind as $crate::animation::TrackKind>::TAG {
                    return Ok(Self::$variant(cursor.read::<$crate::animation::AnimationTrack<$crate::animation::$kind>>()?));
                })+
                Err($crate::DecodeError::MalformedRecord { tag, offset })
            }
        }
        impl $crate::Writable for &$group {
            fn write_to(self, encoder: &mut $crate::Encoder<'_>) {
                match self { $( $group::$variant(track) => encoder.write(track), )+ }
            }
        }
        $(impl From<$crate::animation::AnimationTrack<$crate::animation::$kind>> for $group {
            fn from(track: $crate::animation::AnimationTrack<$crate::animation::$kind>) -> Self { Self::$variant(track) }
        })+
    };
}
pub(crate) use track_group;
