use crate::design::composer::impl_graph::builder::*;
use crate::design::composer::impl_graph::{Edge, ImplementationGraph, Node};
use crate::design::{
    IFKey, Interface, LibKey, Library, Mode, NodeIFHandle, NodeKey, Project, Streamlet,
    StreamletHandle, StreamletKey, GEN_LIB,
};
use crate::{Error, Name, Result, UniqueKeyBuilder};

use std::convert::{TryFrom, TryInto};
use std::rc::Rc;

use crate::design::composer::impl_graph::patterns::MapPattern;
use crate::design::composer::GenericComponent;
use crate::error::LineErr;

use pest::iterators::{Pair};
use pest::{Parser, RuleType};
use std::collections::HashMap;

#[derive(Parser)]
#[grammar = "design/composer/impl_graph/impl.pest"]
pub struct ImplDef;

pub trait LineNum {
    fn line_num(&self) -> usize;
}
impl<'i, R: RuleType> LineNum for Pair<'i, R> {
    fn line_num(&self) -> usize {
        self.as_span().start_pos().line_col().0
    }
}

fn match_rule<T>(
    pair: Pair<Rule>,
    rule: Rule,
    mut f: impl FnMut(Pair<Rule>) -> Result<T>,
) -> Result<T> {
    if pair.as_rule() == rule {
        f(pair)
    } else {
        Err(Error::ImplParsingError(LineErr::new(
            pair.line_num(),
            format!("Expected: \"{:?}\", Actual: \"{:?}\"", rule, pair),
        )))
    }
}

pub struct ImplParser<'i> {
    project: &'i mut Project,
    //body: String,
    imp: ImplementationGraph,
}

impl<'i> ImplParser<'i> {
    pub fn try_new(project: &'i mut Project, input: &'i str) -> Result<Self> {
        let pair = ImplDef::parse(Rule::implementation, input)
            .map_err(|e| {
                Error::ImplParsingError(LineErr::new(
                    0,
                    format!("Implementation parsing error: {}", e),
                ))
            })?
            .next()
            .unwrap();

        let mut pairs = pair.into_inner();
        let streamlet_handle: StreamletHandle = pairs.next().unwrap().try_into()?;

        let s = project
            .get_lib(streamlet_handle.lib())?
            .get_streamlet(streamlet_handle.streamlet())?
            .clone();

        let gen_lib = Library::new(LibKey::try_new(GEN_LIB).unwrap());
        project.add_lib(gen_lib);

        Ok(ImplParser {
            project,
            //body: pairs.next().unwrap().as_str().to_string(),
            imp: ImplementationGraph {
                streamlet: streamlet_handle,
                edges: vec![],
                nodes: vec![(
                    NodeKey::this(),
                    Node {
                        key: NodeKey::this(),
                        item: Rc::new(s.clone()),
                    },
                )]
                .into_iter()
                .collect::<HashMap<NodeKey, Node>>(),
            },
        })
    }

    /*pub fn parse_node(&mut self, pair: Pair<Rule>) -> Result<()> {
        match_rule(pair, Rule::node, |pair| {
            let mut pairs = pair.into_inner();
            let name_pair = pairs.next().unwrap().as_str();
            let key = Name::try_from(name_pair)?;
            //println!("Pairs: {:?}", pairs.next());
            let component = pairs.next().unwrap();
            match component.as_rule() {
                Rule::streamlet_inst => {
                    let mut pairs = component.into_inner();
                    let streamlet_handle = StreamletHandle::try_from(pairs.next().unwrap())?;
                    match self
                        .project
                        .get_lib(streamlet_handle.lib())?
                        .get_streamlet(streamlet_handle.streamlet())
                    {
                        Ok(s) => {
                            let mut s_copy = s.clone();
                            s_copy.set_key(key.clone());
                            let node = Node {
                                key: key.clone(),
                                item: Rc::new(s_copy.clone()),
                            };
                            self.imp.nodes.insert(node.key().clone(), node);
                            Ok(())
                        }
                        Err(e) => Err(e),
                    }
                }
                Rule::pattern_node => {
                    println!("Here come the girls!");
                    //let mut pairs = component.into_inner();
                    self.parse_pattern_node(key, component)?;
                    Ok(())
                }
                _ => {
                    println!("Not implemented yet :( {:?}", pairs);
                    unimplemented!();
                }
            };
            Ok(())
        })
    }*/

    /*pub fn parse_streamlet_inst(&self, pair: Pair<Rule>) -> Result<Node> {

    }*/

    /*pub fn parse_pattern_node(&mut self, key: Name, pair: Pair<Rule>) -> Result<()> {
        let mut pairs = pair.into_inner();
        let handle_pair = pairs.next().unwrap().clone();
        let handle = NodeIFHandle::try_from(handle_pair)?;
        let pattern_chain = pairs.next().unwrap();
        let patterns = pattern_chain.into_inner();
        let mut prev_output = handle.clone();

        let mut nodes: Vec<Node> = vec![];
        let mut edges: Vec<Edge> = vec![];

        let _instance_name =
            Name::try_from(format!("{}_{}_map", prev_output.node, prev_output.iface))?;

        //prinn!("Input iface: {:?}", input_iface.clone());

        for p in patterns {
            let pattern = p.into_inner().next().unwrap();
            let node = self.imp.get_node(prev_output.clone().node)?;
            let input_iface = node.iface(prev_output.clone().iface)?;
            match pattern.as_rule() {
                Rule::map => {
                    let _instance_name_str =
                        format!("{}_{}_map", prev_output.clone().node, handle.clone().iface);
                    let instance_name =
                        Name::try_from(format!("{}_{}_map", prev_output.node, prev_output.iface))?;
                    //let instance_name = key.clone();
                    let op_streamlet_handle = StreamletHandle::try_from(
                        pattern
                            .into_inner()
                            .next()
                            .unwrap()
                            .into_inner()
                            .next()
                            .unwrap(),
                    )?;
                    let op = self.project.get_streamlet(op_streamlet_handle)?;
                    let map = MapPattern::try_new(
                        instance_name.as_ref(),
                        op.clone(),
                        input_iface.clone(),
                        StreamletHandle {
                            lib: Name::try_from(GEN_LIB)?,
                            streamlet: instance_name.clone(),
                        },
                    )?;
                    let node = Node {
                        key: instance_name.clone(),
                        item: Rc::new(map),
                    };

                    nodes.push(node.clone());
                    edges.push(Edge {
                        source: prev_output,
                        sink: node.io("in")?,
                    });
                    self.project
                        .get_lib_mut(Name::try_from(GEN_LIB).unwrap())?
                        .add_streamlet(node.item.streamlet().clone());
                    println!("Streamlet stored: {:?}", node.item.streamlet().key().clone());
                    prev_output = node.io("out")?;
                }
                _ => {
                    println!("That sucks mate! {:?}", pattern.as_rule());
                    unimplemented!();
                }
            }
        }

        //Create the wrapper streamlet
        let streamlet = Streamlet::from_builder(
            key.clone(),
            UniqueKeyBuilder::new().with_items(vec![
                nodes
                    .first()
                    .unwrap()
                    .item
                    .streamlet()
                    .inputs()
                    .next()
                    .unwrap()
                    .clone(),
                nodes
                    .last()
                    .unwrap()
                    .item
                    .streamlet()
                    .outputs()
                    .next()
                    .unwrap()
                    .clone(),
            ]),
            None,
        )
        .unwrap();

        self.project
            .get_lib_mut(Name::try_from(GEN_LIB).unwrap())?
            .add_streamlet(streamlet.clone());
        let mut impl_builder = BasicGraphBuilder::new(
            streamlet.clone(),
            StreamletHandle {
                lib: Name::try_from(GEN_LIB)?,
                streamlet: key.clone(),
            },
        );
        impl_builder.append_nodes(nodes);
        impl_builder.append_edges(&mut edges);
        let implementation = impl_builder.finish();
        self.project
            .get_lib_mut(Name::try_from(GEN_LIB)?)
            .unwrap()
            .get_streamlet_mut(key)
            .unwrap()
            .attach_implementation(implementation);

        Ok(())
        //println!("Iffffffhandle: {:?}", handle);
        //unimplemented!()
    }*/

    /*pub fn parse_pattern_chain(&self, pair: Pair<Rule>) -> Result<Node> {

    }*/

    pub fn transform_impl(&mut self) -> Result<()> {
        let body = self.body.clone();
        /*let pair = ImplDef::parse(Rule::implementation, body.as_str())
            .map_err(|e| {
                Error::ImplParsingError(LineErr::new(
                    0,
                    format!("Implementation body parsing error: {}", e),
                ))
            })?
            .next()
            .unwrap();*/

        //let pairs = pair.into_inner();

        match_rule(pair, Rule::streamlet_handle, |pair| {
            let mut pairs = pair.into_inner();
            println!("Structural!");
            Ok(())
        });



        //let streamlet_handle_p = pairs

        /*for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::structural => {
                    println!("It's a connection! <3");
                    //let edge = Edge::try_from(pair)?;
                    //self.connect(edge);
                }
                Rule::node => {
                    println!("It's a instantiation! <3");
                    //let _node = self.parse_node(pair)?;
                    //self.imp.nodes.insert(node.key().clone(), node);
                }
                _ => {
                    println!("Not implemented yet :( : {:?}", pair);
                }
            }
        }*/

        Ok(())
    }

    pub fn connect(&mut self, edge: Edge) -> Result<()> {
        self.imp.edges.push(edge);
        Ok(())
    }

    pub fn this(&self) -> Node {
        // We can unwrap safely here because the "this" node should always exist.
        self.imp.nodes.get(&NodeKey::this()).unwrap().clone()
    }

    pub fn finish(self) -> ImplementationGraph {
        self.imp
    }
}

impl<'i> TryFrom<Pair<'i, Rule>> for StreamletHandle {
    type Error = Error;
    fn try_from(pair: Pair<Rule>) -> Result<Self> {
        match_rule(pair, Rule::streamlet_handle, |pair| {
            let mut pairs = pair.into_inner();
            let lib = pairs.next().unwrap().as_str();
            let streamlet = pairs.next().unwrap().as_str();
            let lib_key = LibKey::try_from(lib)?;
            let streamlet_key = StreamletKey::try_from(streamlet)?;
            println!("StreamletHandle: {}.{}", lib, streamlet);
            Ok(StreamletHandle {
                lib: lib_key,
                streamlet: streamlet_key,
            })
        })
    }
}

impl<'i> TryFrom<Pair<'i, Rule>> for Name {
    type Error = Error;
    fn try_from(pair: Pair<Rule>) -> Result<Self> {
        match_rule(pair, Rule::ident, |pair| {
            let mut pairs = pair.into_inner();
            Name::try_from(pairs.next().unwrap().as_str())
        })
    }
}

impl<'i> TryFrom<Pair<'i, Rule>> for NodeIFHandle {
    type Error = Error;
    fn try_from(pair: Pair<Rule>) -> Result<Self> {
        match_rule(pair, Rule::node_if_handle, |pair| {
            let mut pairs = pair.into_inner();
            let node = pairs.next().unwrap().as_str();
            let node = Name::try_from(node)?;

            let iface = pairs.next().unwrap().as_str();
            let iface = Name::try_from(iface)?;
            println!("NodeIfHandle: {}.{}", node, iface);
            Ok(NodeIFHandle { node, iface })
        })
    }
}

impl<'i> TryFrom<Pair<'i, Rule>> for Edge {
    type Error = Error;
    fn try_from(pair: Pair<Rule>) -> Result<Self> {
        match_rule(pair, Rule::connection, |pair| {
            let mut pairs = pair.into_inner();

            let sink = NodeIFHandle::try_from(pairs.next().unwrap())?;
            let source = NodeIFHandle::try_from(pairs.next().unwrap())?;

            println!("Edge: {:?}.{:?}", source, sink);
            Ok(Edge { source, sink })
        })
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    use crate::design::*;
    use crate::logical::LogicalType;
    use crate::parser::nom::interface;
    use crate::{Name, Result, UniqueKeyBuilder};
    use std::convert::TryFrom;

    pub(crate) fn impl_parser_test() -> Result<Project> {
        let key1 = LibKey::try_new("primitives").unwrap();
        let key2 = LibKey::try_new("compositions").unwrap();
        let mut lib = Library::new(key1.clone());

        let mut lib_comp = Library::new(key2.clone());

        //Add streamlet
        let _test1 = lib
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

        let _test2 = lib
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
                        interface("in: in Stream<Bits<32>, d=1>").unwrap().1,
                        interface("out: out Stream<Bits<32>, d=1>").unwrap().1,
                    ]),
                    None,
                )
                .unwrap(),
            )
            .unwrap();

        let _map = lib
            .add_streamlet(
                Streamlet::from_builder(
                    StreamletKey::try_from("Magic").unwrap(),
                    UniqueKeyBuilder::new().with_items(vec![
                        interface("in: in Stream<Bits<32>, d=1>").unwrap().1,
                        interface("out: out Stream<Bits<32>, d=1>").unwrap().1,
                    ]),
                    None,
                )
                .unwrap(),
            )
            .unwrap();

        let _map = lib
            .add_streamlet(
                Streamlet::from_builder(
                    StreamletKey::try_from("Magic2").unwrap(),
                    UniqueKeyBuilder::new().with_items(vec![
                        interface("in: in Stream<Group<a: Bits<32>, b: Stream<Bits<32>,d=1>>, d=1>").unwrap().1,
                        interface("out: out Stream<Bits<32>, d=1>").unwrap().1,
                    ]),
                    None,
                )
                    .unwrap(),
            )
            .unwrap();

        let _sqrt = lib
            .add_streamlet(
                Streamlet::from_builder(
                    StreamletKey::try_from("Sqrt").unwrap(),
                    UniqueKeyBuilder::new().with_items(vec![
                        interface("in: in Stream<Bits<32>>").unwrap().1,
                        interface("out: out Stream<Bits<32>>").unwrap().1,
                    ]),
                    None,
                )
                .unwrap(),
            )
            .unwrap();


        let mut prj = Project::new(Name::try_new("TestProj").unwrap());
        prj.add_lib(lib);
        prj.add_lib(lib_comp);

        let top_impl = include_str!("../../../../tests/top.impl");
        let _map_impl = include_str!("../../../../tests/map.impl");

        /*let mut builder = ImplementationBuilder::new(&prj);
        builder.parse_implementation(&top_impl)?;
        let imp = builder.finish();
        prj.add_streamlet_impl(top, imp)?;*/

        /*let mut builder = ImplementationBuilder::new(&prj);
        builder.parse_implementation(&map_impl)?;
        let imp = builder.finish();
        prj.add_streamlet_impl(map, imp)?;*/

        let mut builder = ImplParser::try_new(&mut prj, &top_impl)?;
        builder.transform_impl().unwrap();
        let imp = builder.finish();
        prj.add_streamlet_impl(top, imp)?;

        Ok(prj)
    }

    #[test]
    fn parser() {
        let key1 = LibKey::try_new("primitives").unwrap();
        let key2 = LibKey::try_new("compositions").unwrap();
        let mut lib = Library::new(key1.clone());

        let mut lib_comp = Library::new(key2.clone());

        //Add streamlet
        let _test1 = lib
            .add_streamlet(
                Streamlet::from_builder(
                    StreamletKey::try_from("Magic").unwrap(),
                    UniqueKeyBuilder::new().with_items(vec![
                        interface("in: in Stream<Bits<32>, d=1>").unwrap().1,
                        interface("out: out Stream<Bits<32>, d=1>").unwrap().1,
                    ]),
                    None,
                )
                .unwrap(),
            )
            .unwrap();

        let _test2 = lib
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

        let _top = lib_comp
            .add_streamlet(
                Streamlet::from_builder(
                    StreamletKey::try_from("Top_level").unwrap(),
                    UniqueKeyBuilder::new().with_items(vec![
                        interface("in: in Stream<Bits<32>, d=1>").unwrap().1,
                        interface("out: out Stream<Bits<32>, d=1>").unwrap().1,
                    ]),
                    None,
                )
                .unwrap(),
            )
            .unwrap();

        let _map = lib_comp
            .add_streamlet(
                Streamlet::from_builder(
                    StreamletKey::try_from("Magic").unwrap(),
                    UniqueKeyBuilder::new().with_items(vec![
                        interface("in: in Stream<Group<size: Bits<32>, elem: Stream<Bits<32>>>>")
                            .unwrap()
                            .1,
                        interface("out: out Stream<Group<size: Bits<32>, elem: Stream<Bits<32>>>>")
                            .unwrap()
                            .1,
                    ]),
                    None,
                )
                .unwrap(),
            )
            .unwrap();

        lib_comp
            .add_streamlet(
                Streamlet::from_builder(
                    StreamletKey::try_from("Sqrt").unwrap(),
                    UniqueKeyBuilder::new().with_items(vec![
                        interface("in: in Stream<Bits<32>>").unwrap().1,
                        interface("out: out Stream<Bits<32>>").unwrap().1,
                    ]),
                    None,
                )
                .unwrap(),
            )
            .unwrap();

        lib_comp
            .add_streamlet(
                Streamlet::from_builder(
                    StreamletKey::try_from("Sequence").unwrap(),
                    UniqueKeyBuilder::new().with_items(vec![
                        interface("out: out Stream<Group<size: Bits<32>, elem: Stream<Bits<32>>>>")
                            .unwrap()
                            .1,
                        interface("length: in Stream<Bits<32>>").unwrap().1,
                        interface("elem: in Stream<Bits<32>>").unwrap().1,
                    ]),
                    None,
                )
                .unwrap(),
            )
            .unwrap();

        lib_comp
            .add_streamlet(
                Streamlet::from_builder(
                    StreamletKey::try_from("Split").unwrap(),
                    UniqueKeyBuilder::new().with_items(vec![
                        interface("in: in Stream<Group<size: Bits<32>, elem: Stream<Bits<32>>>>")
                            .unwrap()
                            .1,
                        interface("length: out Stream<Bits<32>>").unwrap().1,
                        interface("elem: out Stream<Bits<32>>").unwrap().1,
                    ]),
                    None,
                )
                .unwrap(),
            )
            .unwrap();

        let mut prj = Project::new(Name::try_new("TestProj").unwrap());
        prj.add_lib(lib);
        prj.add_lib(lib_comp);

        let top_impl = include_str!("../../../../tests/top.impl");
        let _map_impl = include_str!("../../../../tests/map.impl");

        /*let mut builder = ImplementationBuilder::new(&prj);
        builder.parse_implementation(&top_impl)?;
        let imp = builder.finish();
        prj.add_streamlet_impl(top, imp)?;*/

        /*let mut builder = ImplementationBuilder::new(&prj);
        builder.parse_implementation(&map_impl)?;
        let imp = builder.finish();
        prj.add_streamlet_impl(map, imp)?;*/

        let mut builder = ImplParser::try_new(&mut prj, &top_impl).unwrap();
        builder.transform_impl().unwrap();
        //let imp = builder.finish();
    }

    pub(crate) fn pow2_example() -> Result<Project> {
        let key1 = LibKey::try_new("primitives").unwrap();
        let key2 = LibKey::try_new("compositions").unwrap();
        let mut lib = Library::new(key1.clone());

        let mut lib_comp = Library::new(key2.clone());

        let top = lib_comp
            .add_streamlet(
                Streamlet::from_builder(
                    StreamletKey::try_from("Top_level").unwrap(),
                    UniqueKeyBuilder::new().with_items(vec![
                        interface("in: in Stream<Bits<32>, d=1>").unwrap().1,
                        interface("out: out Stream<Bits<32>, d=1>").unwrap().1,
                    ]),
                    None,
                )
                    .unwrap(),
            )
            .unwrap();

        let _sqrt = lib
            .add_streamlet(
                Streamlet::from_builder(
                    StreamletKey::try_from("Pow2").unwrap(),
                    UniqueKeyBuilder::new().with_items(vec![
                        interface("in: in Stream<Bits<32>>").unwrap().1,
                        interface("out: out Stream<Bits<32>>").unwrap().1,
                    ]),
                    None,
                )
                    .unwrap(),
            )
            .unwrap();




        let mut prj = Project::new(Name::try_new("TestProj").unwrap());
        prj.add_lib(lib);
        prj.add_lib(lib_comp);

        let top_impl = include_str!("../../../../tests/pow2.impl");
        /*let mut builder = ImplementationBuilder::new(&prj);
        builder.parse_implementation(&top_impl)?;
        let imp = builder.finish();
        prj.add_streamlet_impl(top, imp)?;*/

        /*let mut builder = ImplementationBuilder::new(&prj);
        builder.parse_implementation(&map_impl)?;
        let imp = builder.finish();
        prj.add_streamlet_impl(map, imp)?;*/

        let mut builder = ImplParser::try_new(&mut prj, &top_impl)?;
        builder.transform_impl().unwrap();
        let imp = builder.finish();
        prj.add_streamlet_impl(top, imp)?;

        Ok(prj)
    }
}
