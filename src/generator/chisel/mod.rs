//! Chisel back-end.

use crate::design::Project;
use crate::generator::common::*;
use crate::generator::GenerateProject;
use crate::{Error, Result, Reversed};
use log::debug;
use std::path::Path;

use crate::cat;
use crate::generator::common::convert::Packify;
use crate::traits::Identify;
use std::str::FromStr;
#[cfg(feature = "cli")]
use structopt::StructOpt;
use crate::logical::Direction;

mod impls;

/// Modes for Chisel elements.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ChiselMode {
    /// Input.
    In,
    /// Output.
    Out,
    /// Forward.
    Forward,
    /// Reverse.
    Reverse,
}

/// Generate trait for generic Chisel declarations.
pub trait DeclareChisel {
    /// Generate a Chisel declaration from self.
    fn declare(&self) -> Result<String>;
}

/// Generate trait for Chisel type declarations.
pub trait DeclareChiselType {
    /// Generate a Chisel declaration from self.
    fn declare(&self, is_root_type: bool) -> Result<String>;
}

/// Generate trait for Chisel package declarations.
pub trait DeclareChiselLibrary {
    /// Generate a Chisel declaration from self.
    fn declare(&self, abstraction: AbstractionLevel) -> Result<String>;
}

/// Generate trait for Chisel identifiers.
pub trait ChiselIdentifier {
    /// Generate a Chisel identifier from self.
    fn chisel_identifier(&self) -> Result<String>;
}

/// Convert field direction to mode
pub trait FieldMode {
    /// Return Mode with related to the default direction, which is Input
    fn field_mode(&self) -> Result<ChiselMode>;
}

/// Analyze trait for Chisel objects.
pub trait Analyze {
    /// List all record types used.
    fn list_record_types(&self) -> Vec<Type>;
}

/// Chisel back-end code generation result
#[allow(dead_code)]
type ChiselResult = Result<String>;

/// Abstraction levels
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "cli", derive(StructOpt))]
pub enum AbstractionLevel {
    Canonical,
    Fancy,
}

impl Default for AbstractionLevel {
    fn default() -> Self {
        AbstractionLevel::Fancy
    }
}

impl FromStr for AbstractionLevel {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "canon" => Ok(AbstractionLevel::Canonical),
            "fancy" => Ok(AbstractionLevel::Fancy),
            _ => Err(Error::InvalidArgument(s.to_string())),
        }
    }
}

/// VHDL back-end configuration parameters.
#[derive(Debug)]
#[cfg_attr(feature = "cli", derive(StructOpt))]
pub struct ChiselConfig {
    /// Abstraction level of generated files.
    /// Possible options: canonical, fancy.
    ///   canonical: generates the canonical Tydi representation of streamlets as components in a
    ///              package.
    ///   fancy: generates the canonical components that wrap a more user-friendly version for the
    ///          user to implement.
    #[cfg_attr(feature = "cli", structopt(short, long))]
    abstraction: Option<AbstractionLevel>,

    /// Suffix of generated files. Default = "gen", such that
    /// generated files are named <name>.gen.vhd.
    #[cfg_attr(feature = "cli", structopt(short, long))]
    suffix: Option<String>,
}

impl ChiselConfig {
    pub fn abstraction(&self) -> AbstractionLevel {
        self.abstraction.unwrap_or_default()
    }
}

impl Default for ChiselConfig {
    fn default() -> Self {
        ChiselConfig {
            abstraction: None,
            suffix: None
        }
    }
}

/// A configurable VHDL back-end entry point.
#[derive(Default)]
#[allow(dead_code)]
pub struct ChiselBackEnd {
    /// Configuration for the VHDL back-end.
    config: ChiselConfig,
}

impl ChiselBackEnd {
    pub fn config(&self) -> &ChiselConfig {
        &self.config
    }
}

impl From<ChiselConfig> for ChiselBackEnd {
    fn from(config: ChiselConfig) -> Self {
        ChiselBackEnd { config }
    }
}

impl GenerateProject for ChiselBackEnd {
    fn generate(&self, project: &Project, path: impl AsRef<Path>) -> Result<()> {
        // Create the project directory.
        let mut dir = path.as_ref().to_path_buf();
        dir.push(project.identifier());
        std::fs::create_dir_all(dir.as_path())?;

        for lib in project.libraries() {
            let mut pkg = dir.clone();
            pkg.push(format!("{}_pkg", lib.identifier()));
            pkg.set_extension(match self.config.suffix.clone() {
                None => "vhd".to_string(),
                Some(s) => format!("{}.vhd", s),
            });
            /*std::fs::write(
                pkg.as_path(),
                match self.config().abstraction() {
                    AbstractionLevel::Canonical => lib.canonical(),
                    AbstractionLevel::Fancy => lib.fancy(),
                }
                    .declare()?,
            )?;*/
            debug!("Wrote {}.", pkg.as_path().to_str().unwrap_or(""));
        }
        Ok(())
    }
}

