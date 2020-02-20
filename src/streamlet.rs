//! Streamlet definition.

use crate::logical::LogicalStreamType;
use crate::util::UniquelyNamedBuilder;
use crate::{Error, Name, Result};
use std::collections::HashSet;
use std::convert::TryInto;
use std::str::FromStr;

/// Streamlet interface definition.
#[derive(Clone, Debug, PartialEq)]
pub struct Streamlet {
    name: Name,
    interfaces: Vec<Interface>,
}

impl Streamlet {
    fn get_interface(&self, name: impl AsRef<str>) -> Option<&Interface> {
        self.interfaces
            .iter()
            .find(|interface| interface.name.as_ref() == name.as_ref())
    }

    pub fn from_builder(name: Name, builder: UniquelyNamedBuilder<Interface>) -> Result<Self> {
        Ok(Streamlet {
            name,
            interfaces: builder.finish()?,
        })
    }
}

impl crate::traits::Name for Streamlet {
    fn name(&self) -> &str {
        self.name.as_ref()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Mode {
    Out,
    In,
}

impl FromStr for Mode {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self> {
        match input {
            "in" => Ok(Mode::In),
            "out" => Ok(Mode::Out),
            _ => Err(Error::InvalidArgument(format!(
                "{} is not a valid interface Mode. Expected \"in\" or \"out\"",
                input
            ))),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Interface {
    name: Name,
    mode: Mode,
    typ: LogicalStreamType,
}

impl crate::traits::Name for Interface {
    fn name(&self) -> &str {
        self.name.as_ref()
    }
}

type BoxedStdError = Box<dyn std::error::Error>;

impl Interface {
    pub fn new(name: impl Into<Name>, mode: Mode, typ: impl Into<LogicalStreamType>) -> Self {
        Interface {
            name: name.into(),
            mode,
            typ: typ.into(),
        }
    }
    pub fn try_new(
        name: impl TryInto<Name, Error = impl Into<BoxedStdError>>,
        mode: Mode,
        typ: impl TryInto<LogicalStreamType, Error = impl Into<BoxedStdError>>,
    ) -> Result<Self> {
        Ok(Interface {
            name: name.try_into().map_err(Into::into)?,
            mode,
            typ: typ.try_into().map_err(Into::into)?,
        })
    }
}
