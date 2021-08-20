use std::{collections::HashMap, convert::TryFrom};

use indexmap::IndexMap;

use crate::{
    generator::common::{Component, Mode},
    stdlib::common::architecture::{
        assignment::Assign, declaration::ObjectMode, object::ObjectType,
    },
    Error, Identify, Name, Result,
};

use super::{
    assignment::{AssignDeclaration, Assignment, AssignmentKind},
    declaration::ObjectDeclaration,
};

pub enum Statement {
    Assignment(AssignmentKind),
    PortMapping(PortMapping),
}

pub struct PortMapping {
    label: Name,
    component_name: String,
    /// The ports, in the order they were declared on the component
    ports: IndexMap<String, ObjectDeclaration>,
    /// Mappings for those ports, will be declared in the order of the original component declaration,
    /// irrespective of the order they're mapped during generation.
    mappings: HashMap<String, AssignDeclaration>,
}

impl PortMapping {
    pub fn from_component(component: &Component, label: Name) -> Result<PortMapping> {
        let mut ports = IndexMap::new();
        for port in component.ports() {
            ports.insert(
                port.identifier().to_string(),
                ObjectDeclaration::component_port(
                    port.identifier().to_string(),
                    ObjectType::try_from(port.typ().clone())?,
                    port.mode(),
                    None, // TODO: Figure out if there might be some way to determine defaults (signal omissions) at this point
                ),
            );
        }
        Ok(PortMapping {
            label,
            component_name: component.identifier().to_string(),
            ports,
            mappings: HashMap::new(),
        })
    }

    pub fn ports(&self) -> &IndexMap<String, ObjectDeclaration> {
        &self.ports
    }

    pub fn mappings(&self) -> &HashMap<String, AssignDeclaration> {
        &self.mappings
    }

    pub fn map_port(mut self, identifier: String, assignment: &Assignment) -> Result<Self> {
        let port = self
            .ports()
            .get(&identifier)
            .ok_or(Error::InvalidArgument(format!(
                "Port {} does not exist on this component",
                identifier
            )))?;
        let assigned = port.assign(assignment)?;
        self.mappings.insert(identifier, assigned);
        Ok(self)
    }

    pub fn finish(self) -> Result<Self> {
        if self.ports().len() == self.mappings().len() {
            Ok(self)
        } else {
            Err(Error::BackEndError(format!(
                "The number of mappings ({}) does not match the number of ports ({})",
                self.mappings().len(),
                self.ports().len()
            )))
        }
    }
}
