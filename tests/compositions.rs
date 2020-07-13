/// Composition examples
extern crate tydi;

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;
    use std::fs;
    use tydi::design::composer::impl_graph::parser::ImplParser;
    use tydi::design::*;
    use tydi::generator::chisel::ChiselBackEnd;
    use tydi::generator::dot::DotBackend;
    use tydi::generator::GenerateProject;
    
    use tydi::parser::nom::interface;
    use tydi::{Name, Result, UniqueKeyBuilder};

}
