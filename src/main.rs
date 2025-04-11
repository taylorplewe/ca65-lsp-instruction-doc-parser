use std::{
    io::BufRead,
    fs::File,
    collections::HashMap,
};
use serde::Serialize;

fn print_error_and_exit(msg: &str) {
    eprintln!("\x1b[31mERROR\x1b[0m {msg}");
    std::process::exit(1);
}

enum DocParserState {
    Opcodes,
    Description,
}

#[derive(Serialize)]
struct IndexedDocumentation {
    keys_to_doc: HashMap<String, String>,
    keys_with_shared_doc: HashMap<String, String>,
}

pub fn main() {
    let json_file_path = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/instruction-json-location.txt"));
    let doc_file = File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/65816-opcodes.md")).expect("Could not open opcode documentation file.");
    let doc_file_lines = std::io::BufReader::new(doc_file)
        .lines()
        .map_while(Result::ok);

    let mut instruction_doc = IndexedDocumentation {
        keys_to_doc: HashMap::<String, String>::new(),
        keys_with_shared_doc: HashMap::<String, String>::new(),
    };

    let mut state = DocParserState::Opcodes;
    let mut curr_opcodes: Vec<String> = vec![];
    let mut curr_description = String::new();
    for line in doc_file_lines {
        match state {
            DocParserState::Opcodes => {
                if line == "{:}" {
                    state = DocParserState::Description;
                } else if let Some(opcode) = line.strip_prefix('{').and_then(|s| s.strip_suffix('}')) {
                    curr_opcodes.push(opcode.to_string());
                }
            },
            DocParserState::Description => {
                if line == "{.}" {
                    let first_opcode = curr_opcodes.pop().expect("No opcodes preceded a documentation block");
                    instruction_doc.keys_to_doc.insert(first_opcode.to_owned(), curr_description.to_owned());
                    for opcode in curr_opcodes.drain(..) {
                        instruction_doc.keys_with_shared_doc.insert(opcode.to_owned(), first_opcode.to_owned());
                    }
                    curr_description.clear();
                    state = DocParserState::Opcodes;
                } else {
                    curr_description.push_str(&line);
                    curr_description.push('\n');
                }
            }
        }
    }

    // write JSON-serialized data to output file
    if let Ok(json) = serde_json::to_string_pretty(&instruction_doc) {
        if std::fs::write(json_file_path, json).is_err() {
            print_error_and_exit(&format!("could not write to JSON file at {json_file_path}"));
        } else {
            println!("\x1b[32mSuccessfully wrote JSON to \x1b[0m{json_file_path}");
        }
    } else {
        print_error_and_exit("could not serialize markdown hashmap to JSON");
    }
}
