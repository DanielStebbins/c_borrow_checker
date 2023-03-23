/*
Source (C) Code Assumptions:
    - Valid C syntax (compiles).
    - No multi-line comments.
    - { and } on their own lines.
    - No variable name shadowing (TODO).
    - Assignments have a single = as the first = on their line.
    - One statement per line (0 or 1 semicolons per line).
    - Spaces around ' = '
*/

/*
Ranting:
    - I can't know every field of every struct in an arbitrary program, so it must be possible to kill a label the first time I see it, without it ever having been alive.
    - Avoid unwrap() when possible
*/

/*
TODO:
    - { } scope levels.
    -
*/

use regex::Regex;
use std::collections::HashSet;
use std::fs;

fn main() {
    let file_path = "inputs\\ownership_smallest.c";
    let lines = read_file(file_path);

    let mut dead: Vec<HashSet<String>> = Vec::new();
    for line in lines {
        // Create the mapping of variables for this line.
        if let Some(last) = dead.last() {
            dead.push(last.clone());
        } else {
            dead.push(HashSet::new());
        }
        let set = dead.last_mut().unwrap();

        let mut killed = "".to_string();
        let equal_index = line.find('=');
        if equal_index.is_some() && equal_index != line.find("==") {
            // Line is an assignment.
            // Split the line before the '=' into tokens, the last one before the '=' is the variable name.
            // let split = line
            //     .split('=')
            //     .map(|line| line.trim())
            //     // .filter(|token| !token.is_empty())
            //     .collect::<Vec<_>>();
            // println!("{split:?}");
            let split = line.split_whitespace().collect::<Vec<_>>();
            let list_eq_index = split.iter().position(|&r| r == "=").unwrap();

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
        } else {
            // Line is not an assignment.
            if let Err(err_message) = has_dead(set, line.as_str()) {
                println!("{err_message} in line '{line}'");
            }
        }

        // Add new dead variables after checking for dead variables, else they would always throw errors.
        if !killed.is_empty() {
            set.insert(killed);
        }
    }

    println!("{dead:?}");
}

fn read_file(path: &str) -> Vec<String> {
    let contents = fs::read_to_string(path).expect("Could not read source file");

    // Change \r\n to \n on Linux.
    let lines = contents.split("\r\n");
    lines
        .into_iter()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && (line.len() < 2 || !line.starts_with("//")))
        .map(|line| line.to_owned())
        .collect()
}

// If needs more types of errors, make an enum for it.
fn has_dead(variables: &HashSet<String>, s: &str) -> Result<(), String> {
    // Checks if character before is a piece of a variable name or a period (not dead if match).
    let before = Regex::new(r"[a-zA-Z0-9_.]+").unwrap();

    // Checks if character after is a piece of a variable (not dead if match).
    let after = Regex::new(r"[a-zA-Z0-9_]+").unwrap();

    for variable in variables.iter() {
        // If s contains the current variable.
        if let Some(index) = s.find(variable) {
            let next_index = index + variable.len();
            if (index < 1 || !before.is_match(&s[index - 1..index]))
                && (next_index >= s.len() || !after.is_match(&s[next_index..next_index + 1]))
            {
                // Here, we can be sure s contains a reference to a dead variable based on the regex checks.
                return Err(format!("Found dead variable '{variable}'"));
            }
        }
    }
    Ok(())
}

// RUN                         cargo clippy            to view
// git commit -m ""     ->     cargo clippy --fix      to fix
