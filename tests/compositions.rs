/// Composition examples
extern crate tydi;#[cfg(test)]

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;
    use tydi::design::*;
    use tydi::logical::LogicalType;
    use tydi::parser::nom::interface;
    use tydi::{Name, Result, UniqueKeyBuilder};
    use super::*;
    use tydi::design::composer::impl_graph::parser::ImplParser;
    use tydi::generator::dot::DotBackend;
    use tydi::generator::GenerateProject;

    pub(crate) fn composition_test_proj() -> Result<Project> {
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

        let _top = lib_comp
            .add_streamlet(
                Streamlet::from_builder(
                    StreamletKey::try_from("Top_level").unwrap(),
                    UniqueKeyBuilder::new().with_items(vec![
                        interface("in: in Stream<Bits<32>, d=1>").unwrap().1,
                        interface("out: out Stream<Bits<32>, d=0>").unwrap().1,
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

        let _test_op = lib
            .add_streamlet(
                Streamlet::from_builder(
                    StreamletKey::try_from("test_op").unwrap(),
                    UniqueKeyBuilder::new().with_items(vec![
                        interface("in: in Stream<Bits<32>, d=0>").unwrap().1,
                        interface("out: out Stream<Bits<32>, d=0>").unwrap().1,
                    ]),
                    None,
                )
                    .unwrap(),
            )
            .unwrap();

        let mut prj = Project::new(Name::try_new("TestProj").unwrap());
        prj.add_lib(lib)?;
        prj.add_lib(lib_comp)?;
        Ok(prj)
    }

    pub(crate) fn impl_parser_test() -> Result<Project> {
        let mut prj = composition_test_proj()?;
        let top_impl = include_str!("implementations/top.impl");

        /*let mut builder = ImplementationBuilder::new(&prj);
        builder.parse_implementation(&top_impl)?;
        let imp = builder.finish();
        prj.add_streamlet_impl(top, imp)?;*/

        /*let mut builder = ImplementationBuilder::new(&prj);
        builder.parse_implementation(&map_impl)?;
        let imp = builder.finish();
        prj.add_streamlet_impl(map, imp)?;*/

        let mut builder = ImplParser::try_new(&mut prj, &top_impl)?;
        builder.transform_body().unwrap();
        let imp = builder.finish();
        prj.add_streamlet_impl(
            StreamletHandle {
                lib: Name::try_from("compositions")?,
                streamlet: Name::try_from("Top_level")?,
            },
            imp,
        )?;
        Ok(prj)
    }

    #[test]
    fn parser() -> Result<()> {
        let mut prj = composition_test_proj()?;

        let top_impl = include_str!("implementations/top.impl");

        /*let mut builder = ImplementationBuilder::new(&prj);
        builder.parse_implementation(&top_impl)?;
        let imp = builder.finish();
        prj.add_streamlet_impl(top, imp)?;*/

        /*let mut builder = ImplementationBuilder::new(&prj);
        builder.parse_implementation(&map_impl)?;
        let imp = builder.finish();
        prj.add_streamlet_impl(map, imp)?;*/

        let mut builder =ImplParser::try_new(&mut prj, &top_impl).unwrap();
        builder.transform_body().unwrap();
        let _imp = builder.finish();
        Ok(())
    }

    #[test]
    fn dot_impl() {
        let tmpdir = tempfile::tempdir().unwrap();

        let prj = impl_parser_test().unwrap();
        //let prj = pow2_example().unwrap();
        let dot = DotBackend {};
        // TODO: implement actual test.

        assert!(dot.generate(&prj, tmpdir).is_ok());
    }

}