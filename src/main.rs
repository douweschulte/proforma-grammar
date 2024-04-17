use std::io::Read;

use ebnf::{io::MarkableReader, parser::Parser, Syntax};
use toml::{Table, Value};

// TODO: Potential future nice things
// * Amount of rules tested and passed overview
// * Better display of the errors
// * Finish ProForma definition

fn main() {
    // Parse EBNF
    let mut definition_file = String::new();
    let _file = std::fs::File::open("main.ebnf")
        .unwrap()
        .read_to_string(&mut definition_file)
        .unwrap();
    let mut tests_file = String::new();
    let _file = std::fs::File::open("test.toml")
        .unwrap()
        .read_to_string(&mut tests_file)
        .unwrap();
    let syntax = ebnf::lex::parse_str("main.ebnf", &definition_file)
        .unwrap_or_else(|e| panic!("Lexing error:\n{:#?}", e));
    let config = ebnf::parser::graph::GraphConfig::new();
    let syntax = Syntax::new(syntax).unwrap_or_else(|e| panic!("Syntax error:\n{:#?}", e));
    let graph = ebnf::parser::graph::LexGraph::compile(&syntax, &config);

    let tests = tests_file.parse::<Table>().unwrap();
    for (name, set) in tests {
        if let Value::Table(set) = set {
            let mut parser = Parser::new(&graph, &name)
                .unwrap_or_else(|| panic!("The name '{name}' is not defined"));
            if let Some(set) = set.get("positive") {
                let mut positive = 0;
                let mut negative = 0;
                if let Value::Array(tests) = set {
                    for test in tests {
                        if let Value::String(test) = test {
                            match parser.parse(&mut MarkableReader::new(test, test.as_str().into()))
                            {
                                Ok(_) => positive += 1,
                                Err(e) => {
                                    println!("Positive example failed: {:#?}", e);
                                    negative += 1;
                                }
                            }
                        } else {
                            panic!("The toml test file should be a string for '{name}' 'positive'");
                        }
                    }
                } else {
                    panic!("The toml test file should be a array for '{name}' 'positive'");
                }
                println!(
                    "{name} - Handled {} positive examples, failed {}",
                    positive + negative,
                    negative
                );
            }
            if let Some(set) = set.get("negative") {
                let mut positive = 0;
                let mut negative = 0;
                if let Value::Array(tests) = set {
                    for test in tests {
                        if let Value::String(test) = test {
                            match parser.parse(&mut MarkableReader::new(test, test.as_str().into()))
                            {
                                Ok(_) => {
                                    println!("Negative example failed: '{test}'",);
                                    positive += 1
                                }
                                Err(_) => {
                                    negative += 1;
                                }
                            }
                        } else {
                            panic!("The toml test file should be a string for '{name}' 'negative'");
                        }
                    }
                } else {
                    panic!("The toml test file should be a array for '{name}' 'negative'");
                }
                println!(
                    "{name} - Handled {} negative examples, failed {}",
                    positive + negative,
                    positive
                );
            }
        } else {
            panic!("The toml test file should be a table for '{name}'");
        }
    }
}
