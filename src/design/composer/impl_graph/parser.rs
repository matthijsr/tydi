use crate::design::composer::impl_graph::builder::*;
use crate::design::composer::impl_graph::{Edge, ImplementationGraph, Node};
use crate::design::{LibKey, NodeIFHandle, NodeKey, Project, StreamletHandle, StreamletKey};
use crate::{Error, Result};

use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::rc::Rc;

use pest::Parser;
#[derive(Parser)]
#[grammar = "design/composer/impl_graph/impl.pest"]
pub struct ImplParser;

pub struct ImplementationBuilder<'a> {
    project: &'a Project,
    imp: Option<ImplementationGraph>,
}

impl<'a> ImplementationBuilder<'a> {
    pub fn new(project: &'a Project) -> Self {
        ImplementationBuilder { project, imp: None }
    }

    pub fn parse_Implementation(&mut self, input: &str) -> Result<()> {
        let parsed = ImplParser::parse(Rule::Implementation, input)
            .expect("unsuccessful parse") // unwrap the parse result
            .next()
            .unwrap();

        println!("input: {}", input);

        for elem in parsed.into_inner() {
            match elem.as_rule() {
                //Streamlet identifier, builder can be crated now
                Rule::StreamletHandle => {
                    let handle = parse_StreamletHandle(elem.into_inner())?;
                    let builder = GraphBuilder::try_new(self.project, handle)?;
                    self.imp = Option::from(builder.finish())
                }
                Rule::Instantiation => {
                    println!("That's an Instantiation!");
                    parse_Instantiation(elem.into_inner());
                }
                _ => {
                    println!("Bad news! Supposedly unreachable: {}", elem.as_str());
                }
            }
        }

        Ok(())
    }

    pub fn instantiate(&mut self, streamlet_handle: StreamletHandle, key: NodeKey) -> Result<()> {
        match self
            .project
            .get_lib(streamlet_handle.lib())?
            .get_streamlet(streamlet_handle.streamlet())
        {
            Ok(s) => {
                let node = Node {
                    key: key.clone(),
                    item: Rc::new(s.clone()),
                };
                self.imp.nodes.insert(key.clone(), node.clone());
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub fn finish(self) -> Option<ImplementationGraph> {
        self.imp
    }
}

fn parse_StreamletHandle(mut pair: pest::iterators::Pairs<Rule>) -> Result<StreamletHandle> {
    let lib = pair.next().unwrap().as_str();
    let streamlet = pair.next().unwrap().as_str();
    let lib_key = LibKey::try_from(lib)?;
    let streamlet_key = StreamletKey::try_from(streamlet)?;
    println!("StreamletHandle: {}.{}", lib, streamlet);
    Ok(StreamletHandle {
        lib: lib_key,
        streamlet: streamlet_key,
    })
}

fn parse_StreamletInst(pair: pest::iterators::Pairs<Rule>) -> Result<()> {
    Ok(())
}

fn parse_GenericInst(pair: pest::iterators::Pairs<Rule>) {}

fn parse_Connection(pair: pest::iterators::Pairs<Rule>) -> Result<Vec<Edge>> {

}

fn parse_Instantiation(mut pair: pest::iterators::Pairs<Rule>) -> Result<()> {
    let nodekey = pair.next().unwrap().as_str();
    println!("Nodekey: {}", nodekey);

    let pair = pair.next().unwrap();
    match pair.as_rule() {
        Rule::StreamletInst => {
            println!(
                "This is a magnificent streamlet instance: {}",
                pair.as_str()
            );
        }
        _ => {
            println!("Bad news! Supposedly unreachable: {}", pair.as_str());
        }
    }

    Ok(())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    use crate::design::composer::impl_graph::*;

    use crate::design::*;
    use crate::logical::LogicalType;
    use crate::{Name, Result, UniqueKeyBuilder};
    use std::convert::TryFrom;
    use std::fs;

    #[test]
    fn impl_parser_test() {
        let key1 = LibKey::try_new("primitives").unwrap();
        let key2 = LibKey::try_new("compositions").unwrap();
        let mut lib = Library::new(key1.clone());
        let mut lib_comp = Library::new(key2.clone());

        //Add streamlet
        let test1 = lib
            .add_streamlet(
                Streamlet::from_builder(
                    StreamletKey::try_from("Test1").unwrap(),
                    UniqueKeyBuilder::new().with_items(vec![
                        Interface::try_new("a", Mode::In, LogicalType::Null, None).unwrap(),
                        Interface::try_new("b", Mode::Out, LogicalType::Null, None).unwrap(),
                    ]),
                    None,
                )
                .unwrap(),
            )
            .unwrap();

        let test2 = lib
            .add_streamlet(
                Streamlet::from_builder(
                    StreamletKey::try_from("Test2").unwrap(),
                    UniqueKeyBuilder::new().with_items(vec![
                        Interface::try_new("c", Mode::In, LogicalType::Null, None).unwrap(),
                        Interface::try_new("d", Mode::Out, LogicalType::Null, None).unwrap(),
                    ]),
                    None,
                )
                .unwrap(),
            )
            .unwrap();

        let top = lib_comp
            .add_streamlet(
                Streamlet::from_builder(
                    StreamletKey::try_from("Top_level").unwrap(),
                    UniqueKeyBuilder::new().with_items(vec![
                        Interface::try_new("e", Mode::In, LogicalType::Null, None).unwrap(),
                        Interface::try_new("f", Mode::Out, LogicalType::Null, None).unwrap(),
                    ]),
                    None,
                )
                .unwrap(),
            )
            .unwrap();

        let mut prj = Project::new(Name::try_new("TestProj").unwrap());
        prj.add_lib(lib);
        prj.add_lib(lib_comp);

        let unparsed_file = include_str!("../../../../tests/test.imp");

        let mut builder = ImplementationBuilder::new(&prj);
        builder.parse_Implementation(&unparsed_file).unwrap();

        //prj.add_streamlet_impl(top, imp);
    }
}
