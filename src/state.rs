use serde::{Deserialize, Serialize};

/// Types of notifications a pane can have.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum NotificationType {
    /// Needs user action/input (e.g. permission prompt)
    Waiting,
    /// Actively working (transient live status)
    Executing,
    /// Task has finished
    Completed,
}
