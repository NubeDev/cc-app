//! The durable `room` record — orchestrator-owned schema.

use serde::{Deserialize, Serialize};
use std::fmt;

/// A `room` record (workspace-scoped, belongs to a center).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Room {
    /// Display name (e.g. "Possums", "Koalas").
    pub name: String,
    /// The owning center's id (`care.center.<id>`).
    pub center_id: String,
    /// Soft-delete flag — mirrors `Center.archived`.
    pub archived: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoomError {
    InvalidId(String),
    AlreadyExists(String),
    StoreDenied(String),
}

impl fmt::Display for RoomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RoomError::InvalidId(s) => write!(f, "invalid id: {s:?}"),
            RoomError::AlreadyExists(s) => write!(f, "room already exists: {s}"),
            RoomError::StoreDenied(what) => write!(f, "store denied: {what}"),
        }
    }
}

impl std::error::Error for RoomError {}
