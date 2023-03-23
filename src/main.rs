/*
Source (C) Code Assumptions:
    - Valid C syntax (compiles).
    - No multi-line comments.
    - { and } on their own lines.
    - Assignments have a single = as the first = on their line.
    - One statement per line (0 or 1 semicolons per line).
*/

/*
Ranting:
    - I can't know every field of every struct in an arbitrary program, so it must be possible to kill a label the first time I see it, without it ever having been alive.
    - Avoid unwrap() when possible
*/

/*
TODO:
    - { } scope levels.
    - Variable name shadowing.
    - Better parenthesis recognition for function calls.
*/

use regex::Regex;
use std::collections::HashSet;
use std::fs;

fn main() {
    let file_path = "inputs\\ownership_smallest.c";
    let lines = read_file(file_path);
    check(lines);
    //     let line = "x = foo(a, b);";
    //     let out = function_arguments_in(line);
    //     println!("{out:?}");
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

fn check(lines: Vec<String>) {
    let mut dead: Vec<HashSet<String>> = Vec::new();
    // Includes periods for struct fields.
    let variable_regex = Regex::new(r"^(?:[a-zA-Z_][a-zA-Z0-9_.]*)$").unwrap();

    for line in lines {
        // Create the mapping of variables for this line.
        if let Some(last) = dead.last() {
            dead.push(last.clone());
        } else {
            dead.push(HashSet::new());
        }
        let set = dead.last_mut().unwrap();

        let mut killed: HashSet<String> = HashSet::new();
        let mut unkilled: HashSet<String> = HashSet::new();

        let equal_index = line.find('=');
        if equal_index.is_some() && equal_index != line.find("==") {
            // Line is an assignment.
            // Split the line before the '=' into tokens, the last one before the '=' is the variable name.
            let sides = line.split_once('=').unwrap();

            // RHS of the Assignment.
            let mut chars = sides.1.trim().chars();
            chars.next_back();
            let rhs = chars.as_str();
            if variable_regex.is_match(rhs) {
                // Right Hand Side is a variable name implies this is assignment is creating an alias.
                if let Err(err_message) = starts_with_dead(set, rhs) {
                    println!("{err_message} on RHS of assignment in line '{line}'");
                }
                killed.insert(rhs.to_string());
            }

            // LHS of the Assignment.
            let lhs = sides.0.split_whitespace().collect::<Vec<_>>();
            let variable = lhs.last().unwrap();
            if lhs.len() == 1 {
                // Previously declared variable, might be a field of a dead variable.
                if let Err(err_message) = starts_with_dead_lhs(set, variable) {
                    println!("{err_message} on LHS of assignment in line '{line}'");
                }
                unkilled.insert(variable.to_string());
            }
        } else {
            // Line is not an assignment.
            if let Err(err_message) = has_dead(set, line.as_str()) {
                println!("{err_message} in line '{line}'");
            }
        }

        // Killing the variables that were passed to functions.
        // killed.extend(function_arguments_in(&line));

        // Updating the dead set with this assignment's values.
        set.extend(killed);
        set.retain(|variable| !unkilled.contains(variable));
    }

    println!("{dead:?}");
}

// For the LHS of an assignment, a dead variable should not be flagged. It's fields however, should be.
fn starts_with_dead_lhs(variables: &HashSet<String>, s: &str) -> Result<(), String> {
    for variable in variables.iter() {
        if s.starts_with(variable) {
            let next_index = variable.len();
            if next_index < s.len() && s.chars().nth(next_index).unwrap() == '.' {
                // Here, we can be sure s contains a reference to a dead variable, but is not a dead variable.
                return Err(format!("Found dead variable '{variable}'"));
            }
        }
    }
    Ok(())
}

fn starts_with_dead(variables: &HashSet<String>, s: &str) -> Result<(), String> {
    for variable in variables.iter() {
        if s.starts_with(variable) {
            let next_index = variable.len();
            if next_index >= s.len() || s.chars().nth(next_index).unwrap() == '.' {
                // Here, we can be sure s contains a reference to a dead variable.
                return Err(format!("Found dead variable '{variable}'"));
            }
        }
    }
    Ok(())
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

fn function_arguments_in(line: &str) -> HashSet<String> {
    let function_char_regex = Regex::new(r"[a-zA-Z0-9_]$").unwrap();

    let killed: HashSet<String> = HashSet::new();
    let split = line.split(['(', ')'].as_ref());
    for element in split {
        if function_char_regex.is_match(element) {
            // Last character is part of an identifier.
        }
    }
    killed
}

// RUN                         cargo clippy            to view
// git commit -m ""     ->     cargo clippy --fix      to fix
