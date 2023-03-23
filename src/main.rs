/*
Source (C) Code Assumptions:
    - Valid C syntax (compiles).
    - No multi-line comments.
    - { and } on their own lines.
    - No variable name shadowing (TODO).
    - Assignments have a single = as the first = on their line.
    - One statement per line (0 or 1 semicolons per line).
*/

/*
Ranting:
    - I can't know every field of every struct in an arbitrary program, so it must be possible to kill a label the first time I see it, without it ever having been alive.
    - Avoid unwrap when possible
*/

use regex::Regex;
use std::collections::HashSet;
use std::fs;

fn main() {
    let file_path = "inputs\\ownership_smallest.c";
    let lines = read_file(file_path);

    // let mut variables: Vec<HashMap<String, String>> = Vec::new();
    // let mut dead: Vec<String> = Vec::new();
    // let mut variables: Vec<HashMap<String, bool>> = Vec::new();
    let mut dead: Vec<HashSet<String>> = Vec::new();
    for line in lines {
        // Create the mapping of variables for this line.
        if let Some(last) = dead.last() {
            dead.push(last.clone());
        } else {
            dead.push(HashSet::new());
        }
        let set = dead.last_mut().unwrap();

        // Look for new variable assignments.
        let mut killed = "".to_string();
        let equal_index = line.find('=');
        if equal_index != None && equal_index != line.find("==") {
            // Split the line before the '=' into tokens, the last one before the '=' is the variable name.
            let split = line.split_whitespace().collect::<Vec<_>>();

            let list_eq_index = split.iter().position(|&r| r == "=").unwrap();
            println!("{:?}", split);

            // If there is exactly one token after the equals (likely aliasing).
            if list_eq_index + 2 == split.len() {
                // Remove ';' from the end.
                let mut chars = split.last().unwrap().chars();
                chars.next_back();
                let right_token = chars.as_str();

                // If the RHS is a variable name, add it to the list of dead values.
                let re = Regex::new(r"[a-zA-Z_][a-zA-Z0-9_]*").unwrap();
                if re.is_match(right_token) {
                    killed = right_token.to_string();
                }
            }

            // Remove newly-live variables right away, before the dead variable check
            let left_token = split[list_eq_index - 1];
            if set.contains(left_token) {
                set.remove(left_token);
            }
        }

        // Add new dead variables after checking for dead variables, else they would always throw errors.
        if killed != "" {
            set.insert(killed);
        }
    }

    println!("{:?}", dead);
}

fn read_file(path: &str) -> Vec<String> {
    let contents = fs::read_to_string(path).expect("Could not read source file");

    let raw_lines = contents.split("\r\n");
    let mut lines: Vec<String> = Vec::new();
    for line in raw_lines {
        let trimmed_line = line.trim();

        if !trimmed_line.is_empty()
            && (trimmed_line.len() < 2 || trimmed_line[..2] != "//".to_string())
        {
            lines.push(line.trim().to_string());
        }
    }
    lines
}

// If needs more types of errors, make an enum for it.
fn has_dead(variables: &HashSet<String>, s: &str) -> Result<(), String> {
    let before = Regex::new(r"[a-zA-Z0-9_.]+").unwrap();
    let after = Regex::new(r"[a-zA-Z0-9_]+").unwrap();
    for variable in variables.iter() {
        if let Some(index) = s.find(variable) {
            let next = index + variable.len();
            // println!("{:?}", re.is_match(&s[after..after + 1]));
            if (index < 1 || !before.is_match(&s[index - 1..index]))
                && (next >= s.len() || !after.is_match(&s[next..next + 1]))
            {
                return Err(format!("Found dead variable {}", variable));
            }
        }
    }
    Ok(())
}

// RUN cargo clippy
