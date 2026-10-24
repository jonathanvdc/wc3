use crate::TextureInput;
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use wc3::model::{ClipId, DrawId, ImageResourceId, InstanceId, LookId};

/// A selection that does not belong to its assembly source model.
/// The enclosing [`crate::ModelError`] identifies that source and original part.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SelectionError {
    /// The application's model views omit the selected resource.
    MissingModel,
    /// An absent or foreign material look.
    MaterialLook(LookId),
    /// An absent or foreign logical draw.
    Draw(DrawId),
    /// An absent or foreign instance selected for hiding.
    Instance(InstanceId),
    /// An absent or foreign image dependency selected for replacement.
    ImageResource(ImageResourceId),
    /// A material or emitter input absent from this model's texture requests.
    ImageInput(TextureInput),
    /// A correspondence whose host or child clip is absent from its model.
    ClipMapping {
        /// Clip in the root model.
        host: ClipId,
        /// Clip in the component model.
        child: ClipId,
    },
}
impl Display for SelectionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::MissingModel => f.write_str("missing assembly model"),
            Self::MaterialLook(id) => write!(
                f,
                "assembly selection references an absent or foreign material look {id:?}"
            ),
            Self::Draw(id) => write!(f, "assembly selection references an absent draw {id:?}"),
            Self::Instance(id) => {
                write!(f, "assembly selection references an absent instance {id:?}")
            }
            Self::ImageResource(id) => write!(
                f,
                "assembly replacement references an absent image resource {id:?}"
            ),
            Self::ImageInput(input) => write!(
                f,
                "assembly replacement references an absent image input {input:?}"
            ),
            Self::ClipMapping { host, child } => write!(
                f,
                "assembly clip mapping references an absent clip ({host:?} to {child:?})"
            ),
        }
    }
}
impl Error for SelectionError {}
