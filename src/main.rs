use std::io::Read;

use colored::Colorize;
use ebnf::{io::MarkableReader, parser::Parser, Error, Syntax};
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
                                    print_error(e, test);
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
                if negative > 0 {
                    println!(
                        "{} - {} positive tests, failed {}",
                        name.red(),
                        positive + negative,
                        negative
                    );
                } else {
                    println!("{} - {} positive tests", name.green(), positive);
                }
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
                if positive > 0 {
                    println!(
                        "{} - {} negative tests, failed {}",
                        name.red(),
                        positive + negative,
                        positive
                    );
                } else {
                    println!("{} - {} negative tests", name.green(), negative);
                }
            }
        } else {
            panic!("The toml test file should be a table for '{name}'");
        }
    }
}

fn print_error(error: Error, text: &str) {
    println!(
        "  {}: {}\n   | {}\n     {}{}\n  {}\n",
        "Error".red(),
        error.location.name,
        text,
        " ".repeat((error.location.columns - 1) as usize),
        "^".red(),
        error.message
    )
}
