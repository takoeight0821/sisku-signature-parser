use tree_sitter::{Language, Parser};

extern "C" {
    fn tree_sitter_javascript() -> Language;
}

fn main() {
    let mut parser = Parser::new();
    let language = unsafe { tree_sitter_javascript() };
    parser.set_language(language).unwrap();

    let source_code = "function add(x, y) { return x + y; }";
    let tree = parser.parse(source_code, None).unwrap();

    println!("{}", source_code);
    println!("{}", tree.root_node().to_sexp());

    let source_code = "add(x, y)";
    let tree = parser.parse(source_code, None).unwrap();

    println!("{}", source_code);
    println!("{}", tree.root_node().to_sexp());

    let source_code = "function add(x, y)";
    let tree = parser.parse(source_code, None).unwrap();

    println!("{}", source_code);
    println!("{}", tree.root_node().to_sexp());
}
