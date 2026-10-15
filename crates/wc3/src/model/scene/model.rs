//! Runtime actions in the model scene vocabulary.
/// Timestamp-only action code and external game-data identifier.
#[derive(Clone, Debug)]
pub struct Event {
    /// Four-character action identifier.
    pub identifier: [u8; 4],
    /// Event-specific external game-data identity.
    pub data: u32,
}
