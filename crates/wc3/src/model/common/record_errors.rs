//! Generation of format-qualified bounded record diagnostics.
macro_rules! record_errors {
    ($format:literal) => {
        /// A malformed document or native record.
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct ReadError {
            /// Byte offset within the addressed source; zero when unavailable.
            pub offset: usize,
            /// Description of the failed format requirement.
            pub field: &'static str,
        }
        impl ReadError {
            /// Reports an invalid value or bounds violation.
            pub fn invalid(offset: usize, field: &'static str) -> Self {
                Self { offset, field }
            }
        }
        impl fmt::Display for ReadError {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(
                    f,
                    concat!("invalid ", $format, " {} at byte {}"),
                    self.field, self.offset
                )
            }
        }
        impl Error for ReadError {}
        /// A value that cannot be represented by the native document format.
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct WriteError {
            /// Description of the failed format requirement.
            pub field: &'static str,
        }
        impl WriteError {
            /// Reports an unrepresentable value or layout.
            pub fn invalid(field: &'static str) -> Self {
                Self { field }
            }
        }
        impl fmt::Display for WriteError {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, concat!("cannot encode ", $format, " {}"), self.field)
            }
        }
        impl Error for WriteError {}
    };
}
pub(crate) use record_errors;
