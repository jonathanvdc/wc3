//! Macro for track groups used by MDX records.
//! The default form adds MDL dispatch for groups with unique property names.
//! Use @binary and handwritten MDL codecs when dispatch requires block context.
macro_rules! track_group {
    ($vis:vis enum $group:ident { $($variant:ident : $kind:ident),+ $(,)? }) => {
        $crate::model::animation::track_group! {
            @binary
            #[derive($crate::model::mdl::Read, $crate::model::mdl::Write)]
            #[mdl(tagged)]
            $vis enum $group {
                $(
                    #[mdl(name = <$crate::model::animation::$kind as $crate::model::animation::TrackKind>::MDL_NAME, delegate)]
                    $variant: $kind,
                )+
            }
        }
    };
    (@binary $(#[$attr:meta])* $vis:vis enum $group:ident { $($(#[$variant_attr:meta])* $variant:ident : $kind:ident),+ $(,)? }) => {
        #[derive(Clone, Debug, PartialEq)]
        $(#[$attr])*
        $vis enum $group {
            $( $(#[$variant_attr])* $variant($crate::model::animation::AnimationTrack<$crate::model::animation::$kind>), )+
        }
        impl $group {
            pub fn tag(&self) -> $crate::model::animation::TrackTag {
                match self { $(Self::$variant(_) => <$crate::model::animation::$kind as $crate::model::animation::TrackKind>::TAG_KIND,)+ }
            }
            pub fn accepts_bytes(tag: $crate::model::Tag) -> bool {
                false $(|| tag == <$crate::model::animation::$kind as $crate::model::animation::TrackKind>::TAG)+
            }
        }
        impl $crate::model::mdx::Read for $group {
            fn read_mdx(cursor: &mut $crate::model::Cursor<'_>) -> Result<Self, $crate::model::ReadError> {
                let offset = cursor.absolute_position();
                let tag: $crate::model::Tag = cursor.peek_bytes(4)?.try_into().expect("four-byte tag");
                $(if tag == <$crate::model::animation::$kind as $crate::model::animation::TrackKind>::TAG {
                    return Ok(Self::$variant(cursor.read::<$crate::model::animation::AnimationTrack<$crate::model::animation::$kind>>()?));
                })+
                Err($crate::model::ReadError::MalformedRecord { tag, offset })
            }
        }
        impl $crate::model::mdx::Write for $group {
            fn write_mdx(&self, encoder: &mut $crate::model::Encoder<'_>) -> Result<(), $crate::model::WriteError> {
                match self { $( $group::$variant(track) => encoder.write(track), )+ }
            }
        }
        $(impl From<$crate::model::animation::AnimationTrack<$crate::model::animation::$kind>> for $group {
            fn from(track: $crate::model::animation::AnimationTrack<$crate::model::animation::$kind>) -> Self { Self::$variant(track) }
        })+
    };
}
pub(crate) use track_group;
