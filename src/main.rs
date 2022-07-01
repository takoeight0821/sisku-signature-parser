use std::{io, process::id};

use tree_sitter::{Language, Parser, Tree};

extern "C" {
    fn tree_sitter_javascript() -> Language;
}

fn dump_sexp(source_code: &[u8], tree: &Tree) {
    let mut cursor = tree.walk();

    let mut needs_newline = false;
    let mut indent_level = 0;
    let mut visited_children = false;
    loop {
        let node = cursor.node();
        let is_named = node.is_named();
        if visited_children {
            if is_named {
                print!(")");
            }
            if cursor.goto_next_sibling() {
                visited_children = false;
            } else if cursor.goto_parent() {
                visited_children = true;
                indent_level -= 1;
            } else {
                break;
            }
        } else {
            if is_named {
                if needs_newline {
                    print!("\n");
                }
                for _ in 0..indent_level {
                    print!("  ");
                }
                if let Some(field_name) = cursor.field_name() {
                    print!("{}: ", field_name);
                }
                print!("({}", node.kind());
                needs_newline = true;
            }
            if cursor.goto_first_child() {
                visited_children = false;
                indent_level += 1;
            } else {
                visited_children = true;

                let start = node.start_byte();
                let end = node.end_byte();
                let value = std::str::from_utf8(&source_code[start..end]).expect("has a string");

                print!(" `{}`", value);
            }
        }
    }
    println!("");
}

fn main() {
    let mut parser = Parser::new();
    let language = unsafe { tree_sitter_javascript() };
    parser.set_language(language).unwrap();

    let source_code = "function add(x, y) { return x + y }";
    let tree = parser.parse(source_code, None).unwrap();

    println!("{}", source_code);
    dump_sexp(source_code.as_bytes(), &tree);

    let source_code = "add(x, y)";
    let tree = parser.parse(source_code, None).unwrap();

    println!("{}", source_code);
    dump_sexp(source_code.as_bytes(), &tree);

    let source_code = "function add(x, y)";
    let tree = parser.parse(source_code, None).unwrap();

    println!("{}", source_code);
    dump_sexp(source_code.as_bytes(), &tree);

    let source_code = "const f = (x) => { return \"hoge\" }";
    let tree = parser.parse(source_code, None).unwrap();

    println!("{}", source_code);
    dump_sexp(source_code.as_bytes(), &tree);
}
