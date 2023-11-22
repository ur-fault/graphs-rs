/// This module documents and implements format of graphs in the text form.
///
/// # Format
/// ## Example
/// 1 -> { ->3 > 2 > (6 -> *5 -> 4) > 7 > (8 -> 9)-> }
///
/// ## Syntax
/// connection      :== <exportable> (<connection-type> <expresion>)+
/// connection-type :== "->" | "<-" | "<->"
/// list            :== "(" (<exportable> ";")* <expression> ")"
/// full            :== "[ " <exportable> ("," <expresion>)+ " ]"
/// cycle           :== "{" <exportable> ("->" <expression>)+ "}"
/// exportable      :== "*" <expression>
/// expression      :== <node> | <list> | <full> | <cycle> | <connection>
/// node            :== <number> | <name>
/// number          :== <digit>+
/// name            :== "\"" <char>+ "\""
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    multi::separated_list1,
    sequence::{delimited, pair, preceded, terminated},
    IResult, Parser,
};

enum Node {
    Name(String),
    Number(u32),
}

enum ConnectionType {
    Forward,
    Backward,
    Both,
}

struct Connection {
    left: Exportable,
    rest: Vec<(ConnectionType, Expression)>,
}

struct List(Vec<Exportable>);

struct Full(Vec<Exportable>);

struct Cycle(Vec<Exportable>);

enum Expression {
    Node(Node),
    List(List),
    Full(Full),
    Cycle(Cycle),
}

struct Exportable {
    expression: Expression,
    exported: bool,
}

fn name(input: &str) -> IResult<&str, String> {
    delimited(
        tag("\""),
        take_while1(|c: char| c.is_alphanumeric() || c == '_' || c == '-' || c == '.')
            .map(|n: &str| n.to_string()),
        tag("\""),
    )(input)
}

fn number(input: &str) -> IResult<&str, u32> {
    take_while1(|c: char| c.is_digit(10))
        .map(|s: &str| s.parse().unwrap())
        .parse(input)
}

fn node(input: &str) -> IResult<&str, Node> {
    alt((name.map(|n| Node::Name(n)), number.map(|n| Node::Number(n))))(input)
}

fn expression(input: &str) -> IResult<&str, Expression> {
    todo!()
}

fn exportable(input: &str) -> IResult<&str, Exportable> {
    alt((
        preceded(tag("*"), expression).map(|e| Exportable {
            expression: e,
            exported: true,
        }),
        expression.map(|e| Exportable {
            expression: e,
            exported: false,
        }),
    ))(input)
}

fn cycle(input: &str) -> IResult<&str, Cycle> {
    delimited(
        tag("{"),
        delimited(
            tag("->"),
            pair(
                terminated(exportable, tag(">")),
                separated_list1(tag(">"), exportable),
            )
            .map(|(first, mut rest)| {
                rest.insert(0, first);
                Cycle(rest)
            }),
            tag("->"),
        ),
        tag("}"),
    )(input)
}
