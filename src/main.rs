use std::{io::Read, process::exit};

use colored::Colorize;
use ebnf::{io::MarkableReader, parser::Parser, Error, Syntax};
use toml::{Table, Value};

use crate::generate::generate;

mod generate;

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
    let syntax = ebnf::lex::parse_str("main.ebnf", &definition_file).unwrap_or_else(|e| {
        print_error(e, &definition_file, "Lexing error");
        exit(-1);
    });
    let config = ebnf::parser::graph::GraphConfig::new();
    let syntax = Syntax::new(syntax).unwrap_or_else(|e| {
        print_error(e, &definition_file, "Syntax error");
        exit(-2);
    });
    let graph = ebnf::parser::graph::LexGraph::compile(&syntax, &config);

    // Go over all tests
    let mut total_pos = 0;
    let mut total_neg = 0;
    let mut failed = 0;
    let mut generate = Vec::new();
    let tests = tests_file.parse::<Table>().unwrap();
    for (name, set) in tests {
        if let Value::Table(set) = set {
            let mut parser = Parser::new(&graph, &name).unwrap_or_else(|| {
                println!("The name '{name}' is not defined");
                exit(-3)
            });
            if let Some(set) = set.get("positive") {
                let mut positive = 0;
                let mut negative = 0;
                if let Value::Array(tests) = set {
                    total_pos += tests.len();
                    for test in tests {
                        if let Value::String(test) = test {
                            match parser.parse(&mut MarkableReader::new(test, test.as_str().into()))
                            {
                                Ok(_) => positive += 1,
                                Err(e) => {
                                    print_error(e, test, "Positive test failed");
                                    negative += 1;
                                    failed += 1;
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
                    show_examples(&name, &syntax, 3);
                } else {
                    println!("{} - {} positive tests", name.green(), positive);
                }
            }
            if let Some(set) = set.get("negative") {
                let mut positive = 0;
                let mut negative = 0;
                if let Value::Array(tests) = set {
                    total_neg += tests.len();
                    for test in tests {
                        if let Value::String(test) = test {
                            match parser.parse(&mut MarkableReader::new(test, test.as_str().into()))
                            {
                                Ok(_) => {
                                    println!("   {}: '{test}'", "Negative test failed".red());
                                    positive += 1;
                                    failed += 1;
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
                    show_examples(&name, &syntax, 3);
                } else {
                    println!("{} - {} negative tests", name.green(), negative);
                }
            }
            if let Some(num) = set.get("generate") {
                if let Value::Integer(num) = num {
                    generate.push((name.clone(), *num));
                } else {
                    panic!("The toml test file should be an integer for '{name}' 'generate'");
                }
            }
        } else {
            panic!("The toml test file should be a table for '{name}'");
        }
    }
    println!();
    if failed == 0 {
        println!(
            "{} - {} positive and {} negative tests",
            "Passed".green(),
            total_pos,
            total_neg
        );
    } else {
        println!(
            "{} - {} failed tests out of {} positive and {} negative tests",
            "Failed".red(),
            failed,
            total_pos,
            total_neg
        );
    }

    // Go over generations
    for (name, num) in generate {
        show_examples(&name, &syntax, u64::try_from(num).unwrap());
    }
}

fn print_error(error: Error, text: &str, error_type: &str) {
    println!(
        "  {}: {}\n   | {}\n     {}{}\n  {}",
        error_type.red(),
        error.location.name,
        text.lines()
            .nth((error.location.lines - 1) as usize)
            .unwrap(),
        " ".repeat((error.location.columns - 1) as usize),
        "^".red(),
        error.message.blue(),
    )
}

fn show_examples(name: &str, syntax: &Syntax, num: u64) {
    for n in 0..num {
        println!(
            "  {} {n}: {}",
            "Example".blue(),
            generate(syntax.get_syntax_rule(name).unwrap(), syntax, n).yellow(),
        );
    }
    println!();
}
