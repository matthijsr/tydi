//! Constructs that are used to generate hardware designs, that are not
//! part of the specification (yet).

use crate::Name;
use crate::{Error, Result};

pub mod composer;
pub mod library;
pub mod project;
pub mod streamlet;

pub use library::Library;
pub use project::Project;
pub use streamlet::{Interface, Mode, Streamlet};

/// Index types
pub type LibKey = Name;
pub type IFKey = Name;
pub type StreamletKey = Name;

/// Handles for objects inside a project, through project hierarchy
#[derive(Clone, Debug, PartialEq)]
pub struct StreamletHandle {
    lib: Name,
    streamlet: Name,
}

impl StreamletHandle {
    pub fn lib(&self) -> LibKey {
        self.lib.clone()
    }
    pub fn streamlet(&self) -> StreamletKey {
        self.streamlet.clone()
    }
}
