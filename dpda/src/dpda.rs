use bit_vec::BitVec;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::utils::*;

#[derive(Clone, Debug)]
pub struct DPDA {
    state_num: usize,
    input_alphabet: Vec<char>,
    stack_alphabet: Vec<char>,
    transitions: Vec<HashMap<(char, char), (usize, Vec<char>)>>,
    start_state: usize,
    accept_states: BitVec,
}

impl DPDA {
    pub fn new() -> DPDA {
        DPDA {
            state_num: 0,
            input_alphabet: Vec::new(),
            stack_alphabet: Vec::new(),
            transitions: Vec::new(),
            start_state: 0,
            accept_states: BitVec::new(),
        }
    }

    pub fn load(filename: &str) -> Result<DPDA, LoadError> {
        let mut dpda = DPDA::new();
        let fin = File::open(filename)?;
        let mut fin = BufReader::new(fin);
        let mut buffer = String::new();

        fin.read_line(&mut buffer)?;
        buffer.retain(|c| !c.is_whitespace()); // eliminate all spaces
        dpda.state_num = buffer.parse::<usize>()?;
        buffer.clear();

        fin.read_line(&mut buffer)?;
        buffer.retain(|c| !c.is_whitespace()); // eliminate all spaces
        dpda.input_alphabet = buffer.chars().collect();
        buffer.clear();
        dpda.input_alphabet.insert(0, '_'); // epsilon

        fin.read_line(&mut buffer)?;
        buffer.retain(|c| !c.is_whitespace()); // eliminate all spaces
        dpda.stack_alphabet = buffer.chars().collect();
        buffer.clear();
        if dpda.stack_alphabet.len() == 0 || dpda.stack_alphabet[0] != '$' {
            return Err(LoadError::FileFormat(
                "The first stack alphabet must be $".to_string(),
            ));
        }

        dpda.transitions = vec![HashMap::new(); dpda.state_num];

        for i in 0..dpda.state_num {
            fin.read_line(&mut buffer)?;
            buffer.retain(|c| !c.is_whitespace()); // eliminate all spaces
            let parts = buffer.split(':').collect::<Vec<&str>>();
            if parts.len() != 2 {
                return Err(LoadError::FileFormat(
                    "Wrong number of colons in line".to_string(),
                ));
            }
            if parts[0] != i.to_string() {
                return Err(LoadError::FileFormat("State number mismatch".to_string()));
            }
            let tr_num = parts[1].parse::<usize>()?;
            buffer.clear();

            for _ in 0..tr_num {
                fin.read_line(&mut buffer)?;
                buffer.retain(|c| !c.is_whitespace()); // eliminate all spaces
                if buffer[0..1] != "(".to_string()
                    || buffer[buffer.len() - 1..buffer.len()] != ")".to_string()
                {
                    return Err(LoadError::FileFormat("Parenthesis mismatch".to_string()));
                }

                let rule = buffer[1..buffer.len() - 1]
                    .split(',')
                    .collect::<Vec<&str>>();
                if rule.len() != 4 {
                    return Err(LoadError::FileFormat(
                        "Wrong number of commas in line".to_string(),
                    ));
                }
                let input_letter = rule[0].parse::<char>()?;
                let pop_letter = rule[1].parse::<char>()?;
                let next_state = rule[2].parse::<usize>()?;
                let push_letters = rule[3].chars().collect::<Vec<char>>();
                if !dpda.input_alphabet.contains(&input_letter) {
                    return Err(LoadError::FileFormat(
                        "Input letter not in alphabet".to_string(),
                    ));
                }
                if !dpda.stack_alphabet.contains(&pop_letter) {
                    return Err(LoadError::FileFormat(
                        "Pop letter not in alphabet".to_string(),
                    ));
                }
                if next_state >= dpda.state_num {
                    return Err(LoadError::FileFormat("Next state out of range".to_string()));
                }
                if !(push_letters
                    .iter()
                    .all(|&c| dpda.stack_alphabet.contains(&c))
                    || (push_letters.len() == 1 && push_letters[0] == '_'))
                {
                    return Err(LoadError::FileFormat(
                        "Wrong format of push letter".to_string(),
                    ));
                }
                for ch in dpda.input_alphabet.iter() {
                    if input_letter == '_' || *ch == '_' {
                        match dpda.transitions[i].get(&(*ch, pop_letter)) {
                            Some(_) => {
                                return Err(LoadError::IncorrectDPDA);
                            }
                            None => {}
                        }
                    }
                }
                if push_letters.len() == 1 && push_letters[0] == '_' {
                    dpda.transitions[i]
                        .insert((input_letter, pop_letter), (next_state, Vec::new()));
                } else {
                    dpda.transitions[i]
                        .insert((input_letter, pop_letter), (next_state, push_letters));
                }

                buffer.clear();
            }
        }

        fin.read_line(&mut buffer)?;
        buffer.retain(|c| !c.is_whitespace()); // eliminate all spaces
        dpda.start_state = buffer.parse::<usize>()?;
        if dpda.start_state >= dpda.state_num {
            return Err(LoadError::FileFormat(
                "Start state out of range".to_string(),
            ));
        }
        buffer.clear();

        dpda.accept_states = BitVec::from_elem(dpda.state_num, false);
        fin.read_line(&mut buffer)?;
        buffer.retain(|c| !c.is_whitespace()); // eliminate all spaces
        let accept_states = buffer.split(',').collect::<Vec<&str>>();
        for state in accept_states.iter() {
            let state_int = state.parse::<usize>()?;
            if state_int >= dpda.state_num {
                return Err(LoadError::FileFormat(
                    "Accept state out of range".to_string(),
                ));
            }
            dpda.accept_states.set(state_int, true);
        }
        buffer.clear();

        fin.read_line(&mut buffer)?;
        if buffer.trim() != "END" {
            return Err(LoadError::MissingEnd);
        }
        buffer.clear();

        Ok(dpda)
    }

    fn stack_to_string(stack: &Vec<char>) -> String {
        String::from_iter(stack.iter().rev())
    }

    pub fn is_accepted(self, input: &str, verbose: bool) -> bool {
        let mut stack: Vec<char> = vec!['$'];
        let mut current_state: usize;
        current_state = self.start_state;

        if verbose {
            println!(
                "  ({}, \"{}\", \"{}\")",
                current_state,
                input,
                DPDA::stack_to_string(&stack)
            );
        }

        for (i, c) in input.chars().enumerate() {
            if c == '_' || !self.input_alphabet.contains(&c) {
                return false;
            }
            if let Some((next_state, push_letters)) =
                self.transitions[current_state].get(&(c, stack[stack.len() - 1]))
            {
                current_state = *next_state;
                stack.pop();
                for ch in push_letters.iter().rev() {
                    stack.push(*ch);
                }
            } else {
                if let Some((next_state, push_letters)) =
                    self.transitions[current_state].get(&('_', stack[stack.len() - 1]))
                {
                    current_state = *next_state;
                    stack.pop();
                    for ch in push_letters.iter().rev() {
                        stack.push(*ch);
                    }
                } else {
                    return false;
                }
            }
            if verbose {
                println!(
                    "|-({}, \"{}\", \"{}\")",
                    current_state,
                    &input[i + 1..],
                    DPDA::stack_to_string(&stack)
                );
            }
            if stack.is_empty() {
                return false;
            }
        }
        if stack.pop() != Some('$') {
            return false;
        }
        self.accept_states[current_state] && stack.is_empty()
    }
}
