use std::io::Read;

use ebnf::{io::MarkableReader, parser::Parser, Syntax};

fn main() {
    // Parse EBNF
    let mut definition_file = String::new();
    let _file = std::fs::File::open("main.ebnf")
        .unwrap()
        .read_to_string(&mut definition_file)
        .unwrap();
    let syntax = ebnf::lex::parse_str("main.ebnf", &definition_file)
        .unwrap_or_else(|e| panic!("Lexing error:\n{:#?}", e));
    let config = ebnf::parser::graph::GraphConfig::new();
    let syntax = Syntax::new(syntax).unwrap_or_else(|e| panic!("Syntax error:\n{:#?}", e));
    let graph = ebnf::parser::graph::LexGraph::compile(&syntax, &config);
    if graph.index_of_label("main").is_none() {
        panic!("The 'main' rule is not defined, this is needed as an entry point")
    }
    let mut parser = Parser::new(&graph, "main").expect("Parser error");

    // Positive examples
    let mut positive_examples = String::new();
    let _file = std::fs::File::open("positive.txt")
        .unwrap()
        .read_to_string(&mut positive_examples)
        .unwrap();
    let mut positive = 0;
    for line in positive_examples.lines() {
        parser
            .parse(&mut MarkableReader::new(line, line.into()))
            .unwrap_or_else(|e| panic!("Positive example not parsed:\n{:#?}", e));
        positive += 1;
    }
    println!("Handled {positive} positive examples");

    // Negative examples
    let mut negative_examples = String::new();
    let _file = std::fs::File::open("negative.txt")
        .unwrap()
        .read_to_string(&mut negative_examples)
        .unwrap();
    let mut positive = 0;
    let mut negative = 0;
    for line in positive_examples.lines() {
        match parser.parse(&mut MarkableReader::new(line, line.into())) {
            Ok(_) => positive += 1,
            Err(_) => negative += 1,
        }
    }
    println!(
        "Handled {} negative examples, {} where not negative",
        negative + positive,
        positive
    );
}
