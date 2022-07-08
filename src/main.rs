use std::{
    collections::HashMap,
    fs::File,
    io::{self, Error, ErrorKind, Read},
};

use clap::Parser;
use tree_sitter::{Language, Tree};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser)]
    input: Option<String>,
    #[clap(short, long, value_parser)]
    language: String,
}

extern "C" {
    fn tree_sitter_javascript() -> Language;
}

#[derive(PartialEq, Debug, Clone)]
enum SExp {
    Kind(String),
    Field(String),
    Value(String),
    List(Vec<SExp>),
    /// Use only as a deliminator in `to_sexp`
    Token(String),
}

fn to_sexp(source_code: &[u8], tree: &Tree) -> SExp {
    /* Translate the tree into the s-expression.
     *
     * `to_sexp` uses "(" as a delimiter of a complete s-expression list and an incomplete one.
     * When it walks into a named node, it pushes a Token("(") to the `incomplete`-stack.
     * When it walks into another node, it pops incomplete-stack until a Token("(")
     * and pushes a list constructed from these elements.
     */
    let mut cursor = tree.walk();

    let mut is_children_visited = false;
    let mut incomplete = Vec::new();
    loop {
        let node = cursor.node();
        let is_named = node.is_named();
        if is_children_visited {
            if is_named {
                // End of list
                construct_list(&mut incomplete);
            }
            if cursor.goto_next_sibling() {
                is_children_visited = false;
            } else if !cursor.goto_parent() {
                // There are no more nodes.
                break;
            }
        } else {
            if is_named {
                // Beginning of list
                if let Some(field_name) = cursor.field_name() {
                    incomplete.push(SExp::Field(field_name.to_string()));
                }
                incomplete.push(SExp::Token("(".to_string()));
                incomplete.push(SExp::Kind(node.kind().to_string()));
            }
            if !cursor.goto_first_child() {
                // When the node doesn't have any children,
                // push the text that represents the node.
                // This process collects tokens as s-expression term.
                is_children_visited = true;

                let start = node.start_byte();
                let end = node.end_byte();
                let value = std::str::from_utf8(&source_code[start..end]).expect("has a string");

                incomplete.push(SExp::Value(value.to_string()));
            }
        }
    }
    if incomplete.len() == 1 {
        incomplete[0].clone()
    } else {
        SExp::List(incomplete)
    }
}

fn construct_list(exp_stack: &mut Vec<SExp>) {
    let mut elems: Vec<SExp> = Vec::new();
    while let Some(elem) = exp_stack.pop() {
        if elem == SExp::Token("(".to_string()) {
            if elems.len() == 1 {
                exp_stack.push(elems[0].clone());
            } else {
                elems.reverse();
                exp_stack.push(SExp::List(elems));
            }
            break;
        } else {
            elems.push(elem);
        }
    }
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let mut buffer = String::new();
    match args.input {
        None => io::stdin().read_to_string(&mut buffer)?,
        Some(filepath) => File::open(filepath)?.read_to_string(&mut buffer)?,
    };

    let mut languages = HashMap::new();
    languages.insert("javascript", unsafe { tree_sitter_javascript() });

    let mut parser = tree_sitter::Parser::new();
    let language = languages
        .get(args.language.as_str())
        .ok_or(Error::new(ErrorKind::Other, "unexpected language"))?;
    parser
        .set_language(*language)
        .map_err(|_err| Error::new(ErrorKind::Other, "cannot set language"))?;

    let source_code: &str = buffer.as_str();
    let tree = parser.parse(source_code, None).unwrap();

    println!("{}", source_code);
    println!("{:?}", to_sexp(source_code.as_bytes(), &tree));

    Ok(())
}
