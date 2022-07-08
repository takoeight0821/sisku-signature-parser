use tree_sitter::{Language, Parser, Tree};

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
    println!("{:?}", to_sexp(source_code.as_bytes(), &tree));

    let source_code = "add(x, y)";
    let tree = parser.parse(source_code, None).unwrap();

    println!("{}", source_code);
    dump_sexp(source_code.as_bytes(), &tree);
    println!("{:?}", to_sexp(source_code.as_bytes(), &tree));

    let source_code = "function add(x, y)";
    let tree = parser.parse(source_code, None).unwrap();

    println!("{}", source_code);
    dump_sexp(source_code.as_bytes(), &tree);
    println!("{:?}", to_sexp(source_code.as_bytes(), &tree));

    let source_code = "const f = (x) => { return \"hoge\" }";
    let tree = parser.parse(source_code, None).unwrap();

    println!("{}", source_code);
    dump_sexp(source_code.as_bytes(), &tree);
    println!("{:?}", to_sexp(source_code.as_bytes(), &tree));
}
