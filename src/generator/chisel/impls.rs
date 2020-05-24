//! Implementations of Chisel traits for common representation.

use crate::error::Error::BackEndError;
use crate::generator::chisel::{DeclareChiselType, ChiselMode};
use crate::generator::chisel::{Analyze, ChiselIdentifier, DeclareChisel, FieldMode};
use crate::generator::common::{Component, Field, Mode, Package, Port, Record, Type};
use crate::traits::Identify;
use crate::{cat, Document, Result};
use std::collections::HashMap;



impl ChiselIdentifier for Mode {
    fn chisel_identifier(&self) -> Result<String> {
        match self {
            Mode::In => Ok("Input".to_string()),
            Mode::Out => Ok("Output".to_string()),
        }
    }
}

impl ChiselIdentifier for ChiselMode {
    fn chisel_identifier(&self) -> Result<String> {
        match self {
            ChiselMode::In => Ok("Input".to_string()),
            ChiselMode::Out => Ok("Output".to_string()),
            ChiselMode::Forward => Ok("".to_string()),
            ChiselMode::Reverse => Ok("Flipped".to_string()),
        }
    }
}

impl ChiselIdentifier for Type {
    fn chisel_identifier(&self) -> Result<String> {
        // Records and arrays use type definitions.
        // Any other types are used directly.
        match self {
            Type::Record(rec) =>  Ok(format!("new {}", rec.chisel_identifier()?)),
            _ => self.declare(true),
        }
    }
}

impl ChiselIdentifier for Record {
    fn chisel_identifier(&self) -> Result<String> {
        Ok(cat!(self.identifier().to_string(), "type"))
    }
}

impl FieldMode for Field {
    fn field_mode(&self) -> Result<ChiselMode> {
        match self.typ() {
            Type::Record(_r) => {
                if self.is_reversed() {
                    return Ok(ChiselMode::Reverse);
                } else {
                    return Ok(ChiselMode::Forward);
                }
            }
            _ => {
                if self.is_reversed() {
                    return Ok(ChiselMode::Out);
                } else {
                    return Ok(ChiselMode::In);
                }
            }
        }
    }
}

impl FieldMode for Port {
    fn field_mode(&self) -> Result<ChiselMode> {
        match self.typ() {
            Type::Record(_r) => {
                match self.mode() {
                    Mode::In => {
                        Ok(ChiselMode::Forward)
                    },
                    Mode::Out => {
                        Ok(ChiselMode::Reverse)
                    }
                }
            }
            _ => {
                match self.mode() {
                    Mode::In => {
                        Ok(ChiselMode::In)
                    },
                    Mode::Out => {
                        Ok(ChiselMode::Out)
                    }
                }
            }
        }
    }
}

fn declare_rec(rec: &Record) -> Result<String> {
    let mut children = String::new();
    let mut this = format!(
        "class {} extends Bundle {{ \n",
        cat!(rec.chisel_identifier()?)
    );

    for field in rec.fields() {
        // Declare all nested record types first.
        if let Type::Record(nested) = field.typ() {
            children.push_str(nested.declare(false)?.clone().as_str());
            children.push_str("\n\n");
        };

        // Declare this record.
        this.push_str(
            format!(
                "   val {} = {}({})\n",
                field.identifier(),
                field.field_mode()?.chisel_identifier()?,
                field.typ().chisel_identifier()?
            )
            .as_str(),
        );
    }
    this.push_str("}");
    if !children.is_empty() {
        Ok(format!("{}{}", children, this))
    } else {
        Ok(this)
    }
}

impl DeclareChiselType for Type {
    fn declare(&self, is_root_type: bool) -> Result<String> {
        match self {
            Type::Bit => Ok("Bool()".to_string()),
            Type::BitVec { width } => {
                let actual_width = if *width == 0 { 1 } else { *width };
                Ok(format!("UInt({}.W)", actual_width))
            }
            Type::Record(rec) => rec.declare(is_root_type),
        }
    }
}

impl ChiselIdentifier for Port {
    fn chisel_identifier(&self) -> Result<String> {
        Ok(self.identifier().to_string())
    }
}

impl DeclareChisel for Port {
    fn declare(&self) -> Result<String> {
        let mut result = String::new();
        if let Some(doc) = self.doc() {
            result.push_str("//");
            result.push_str(doc.replace("\n", "\n    //").as_str());
            result.push_str("\n    ");
        }

        result.push_str(
            format!(
                "val {} = {}({})",
                self.identifier(),
                self.field_mode()?.chisel_identifier()?,
                self.typ().chisel_identifier()?
            )
                .as_str(),
        );

        Ok(result)
    }
}

impl DeclareChisel for Component {
    fn declare(&self) -> Result<String> {
        let mut result = String::new();
        if let Some(doc) = self.doc() {
            result.push_str("//");
            result.push_str(doc.replace("\n", "\n//").as_str());
            result.push_str("\n");
        }
        result.push_str(format!("class {} extends Module {{\n", self.identifier()).as_str());
        if !self.ports().is_empty() {
            let mut ports = self.ports().iter().peekable();
            result.push_str(" val io =  IO(new Bundle{\n");
            while let Some(p) = ports.next() {
                result.push_str("    ");
                result.push_str(p.declare()?.to_string().as_str());

                if ports.peek().is_some() {
                    result.push_str(";\n");
                } else {
                    result.push_str("\n");
                }
            }
            result.push_str("  })\n")
        }
        result.push_str("\n\n// User code\n\n");
        result.push_str("}");
        Ok(result)
    }
}

impl Analyze for Component {
    fn list_record_types(&self) -> Vec<Type> {
        let mut result: Vec<Type> = vec![];
        for p in self.ports().iter() {
            if let Type::Record(_) = p.typ() {
                result.push(p.typ())
            }
        }
        result
    }
}

impl DeclareChisel for Package {
    fn declare(&self) -> Result<String> {
        // Whatever generated the common representation is responsible to not to use the same
        // identifiers for different types.
        // Use a set to remember which type identifiers we've already used, so we don't declare
        // them twice.
        let mut type_ids = HashMap::<String, Type>::new();
        let mut result = String::new();
        result.push_str(format!("package {};\n\nimport chisel3._ \n\n", self.identifier).as_str());
        for c in &self.components {
            let comp_records = c.list_record_types();
            for r in comp_records.iter() {
                match type_ids.get(&r.chisel_identifier()?) {
                    None => {
                        type_ids.insert(r.chisel_identifier()?, r.clone());
                        result.push_str(format!("{}\n\n", r.declare(true)?).as_str());
                    }
                    Some(already_defined_type) => {
                        if r != already_defined_type {
                            return Err(BackEndError(format!(
                                "Type name conflict: {}",
                                already_defined_type
                                    .chisel_identifier()
                                    .unwrap_or_else(|_| "".to_string())
                            )));
                        }
                    }
                }
            }
            result.push_str(format!("{}\n\n", c.declare()?).as_str());
        }
        Ok(result)
    }
}

impl DeclareChiselType for Record {
    fn declare(&self, is_root_type: bool) -> Result<String> {
        let mut result = String::new();
        result.push_str(declare_rec(&self)?.as_str());
        Ok(result)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::generator::chisel::DeclareChiselType;
    use crate::generator::common::test::*;

    #[test]
    fn mode_decl() {
        let m0 = Mode::In;
        let m1 = Mode::Out;
        assert_eq!(m0.chisel_identifier().unwrap(), "Input");
        assert_eq!(m1.chisel_identifier().unwrap(), "Output");
    }

    #[test]
    fn prim_type_decl() {
        let t0 = Type::Bit;
        println!("{}", t0.declare(true).unwrap());

        let t1 = Type::BitVec { width: 8 };
        println!("{}", t1.declare(true).unwrap());
    }

    #[test]
    fn record_type_decl() {
        let t0 = records::rec_rev("rec");
        println!("{}", t0.declare(true).unwrap());

        let t1 = records::rec_rev_nested("rec");
        println!("{}", t1.declare(true).unwrap());
    }

    #[test]
    fn port_decl() {
        let p0 = Port::new("test", Mode::In, Type::BitVec { width: 10 });
        println!("{}", p0.declare().unwrap());
        let p1 = Port::new("test", Mode::Out, Type::BitVec { width: 10 });
        println!("{}", p1.declare().unwrap());
    }

    #[test]
    fn comp_decl() {
        let c = test_comp().with_doc(" My awesome\n Component".to_string());

        println!("{}", c.declare().unwrap());
    }

    #[test]
    fn package_decl() {
        let p = Package {
            identifier: "test".to_string(),
            components: vec![test_comp()],
        };
        println!("{}", p.declare().unwrap());
    }
}
