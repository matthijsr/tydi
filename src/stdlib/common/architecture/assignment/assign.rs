use std::convert::TryInto;

use super::{Assign, AssignDeclaration, Assignment};
use crate::{stdlib::common::architecture::declaration::ObjectDeclaration, Error, Result};

impl Assign for ObjectDeclaration {
    fn assign(
        &mut self,
        assignment: &(impl Into<Assignment> + Clone),
    ) -> Result<AssignDeclaration> {
        let true_assignment = assignment.clone().into();
        self.set_mode(self.typ().can_assign(&true_assignment, self.mode())?)?;
        Ok(AssignDeclaration::new(self.clone(), true_assignment))
    }
}

impl<T> Assign for T
where
    T: TryInto<ObjectDeclaration, Error = Error> + Clone,
{
    fn assign(
        &mut self,
        assignment: &(impl Into<Assignment> + Clone),
    ) -> Result<AssignDeclaration> {
        let mut decl = self.clone().try_into()?;
        decl.assign(assignment)
    }
}
