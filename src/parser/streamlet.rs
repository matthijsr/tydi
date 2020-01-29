use nom::{
    character::complete::newline,
    combinator::map,
    IResult,
    multi::{many1, separated_nonempty_list},
    sequence::tuple,
};

use crate::{parser::identifier, parser::river, Streamlet};

/// Parses a Streamlet interface definition.
///
/// A streamlet interface definition consists of one or more input River types
/// followed by one or more output River types, separated by a newline.
///
/// # Example
///
/// ```text
/// MuhStreamlet
///
/// Bits<1>
/// Bits<2>
///
/// Group<Bits<3>, Bits<4>>
/// Bits<4>
/// ```
///
pub fn streamlet_interface_definition(input: &str) -> IResult<&str, Streamlet> {
    map(
        tuple((
            identifier,
            many1(newline),
            separated_nonempty_list(newline, river::river_type),
            many1(newline),
            separated_nonempty_list(newline, river::river_type),
        )),
        |(identifier, _, input, _, output)| Streamlet { identifier, inputs: input, outputs: output },
    )(input)
}

#[cfg(test)]
mod tests {
    use crate::River::{Bits, Group};

    use super::*;

    #[test]
    fn parse_streamlet_interface_definition() {
        assert_eq!(
            streamlet_interface_definition(
                r#"MuhStreamlet

a: Bits<1>
b: Bits<2>

c: Group<Bits<3>, Bits<4>>
d: Bits<4>"#
            )
                .unwrap(),
            (
                "",
                Streamlet {
                    identifier: "MuhStreamlet".to_string(),
                    inputs: vec![
                        Bits {
                            identifier: Some("a".to_string()),
                            width: 1,
                        },
                        Bits {
                            identifier: Some("b".to_string()),
                            width: 2,
                        }
                    ],
                    outputs: vec![
                        Group {
                            identifier: Some("c".to_string()),
                            inner: vec![
                                Bits {
                                    identifier: None,
                                    width: 3,
                                },
                                Bits {
                                    identifier: None,
                                    width: 4,
                                }
                            ],
                        },
                        Bits {
                            identifier: Some("d".to_string()),
                            width: 4,
                        }
                    ],
                }
            )
        );
    }
}
