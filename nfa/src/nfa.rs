use bit_vec::BitVec;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::utils::*;

#[derive(Clone, Debug)]
pub struct NFA {
    state_num: usize,
    alphabet: Vec<char>,
    transitions: Vec<HashMap<char, BitVec>>,
    start_state: usize,
    accept_states: BitVec,
}

impl NFA {
    pub fn new() -> NFA {
        NFA {
            state_num: 0,
            alphabet: Vec::new(),
            transitions: Vec::new(),
            start_state: 0,
            accept_states: BitVec::new(),
        }
    }

    pub fn load(filename: &str) -> Result<NFA, LoadError> {
        let mut nfa = NFA::new();
        let fin = File::open(filename)?;
        let mut fin = BufReader::new(fin);
        let mut buffer = String::new();

        fin.read_line(&mut buffer)?;
        buffer.retain(|c| !c.is_whitespace()); // eliminate all spaces
        nfa.state_num = buffer.parse::<usize>()?;
        buffer.clear();

        fin.read_line(&mut buffer)?;
        buffer.retain(|c| !c.is_whitespace()); // eliminate all spaces
        nfa.alphabet = buffer.chars().collect();
        buffer.clear();

        nfa.alphabet.insert(0, '_'); // epsilon

        nfa.transitions = vec![HashMap::new(); nfa.state_num];

        for i in 0..nfa.state_num {
            fin.read_line(&mut buffer)?;
            buffer.retain(|c| !c.is_whitespace()); // eliminate all spaces
            let parts = buffer.split(':').collect::<Vec<&str>>();
            if parts[0].parse::<usize>()? != i {
                return Err(LoadError::FileFormat("State number mismatch".to_string()));
            }
            if parts.len() != 2 {
                return Err(LoadError::FileFormat(
                    "Wrong number of colons in line".to_string(),
                ));
            }
            let transitions = parts[1].split('/').collect::<Vec<&str>>();
            if transitions.len() != nfa.alphabet.len() {
                return Err(LoadError::FileFormat(
                    "Wrong number of transitions".to_string(),
                ));
            }
            for j in 0..nfa.alphabet.len() {
                let mut bitvec = BitVec::from_elem(nfa.state_num, false);
                if nfa.alphabet[j] == '_' {
                    bitvec.set(i, true);
                }
                let next_states = transitions[j].split(',').collect::<Vec<&str>>();
                for next_state in next_states {
                    if next_state.is_empty() {
                        continue;
                    }
                    let next_state_int = next_state.parse::<usize>()?;
                    if next_state_int >= nfa.state_num {
                        return Err(LoadError::FileFormat("Next state out of range".to_string()));
                    }
                    bitvec.set(next_state_int, true);
                }
                nfa.transitions[i].insert(nfa.alphabet[j], bitvec);
            }
            buffer.clear();
        }

        fin.read_line(&mut buffer)?;
        buffer.retain(|c| !c.is_whitespace()); // eliminate all spaces
        nfa.start_state = buffer.parse::<usize>()?;
        if nfa.start_state >= nfa.state_num {
            return Err(LoadError::FileFormat(
                "Start state out of range".to_string(),
            ));
        }
        buffer.clear();

        nfa.accept_states = BitVec::from_elem(nfa.state_num, false);
        fin.read_line(&mut buffer)?;
        buffer.retain(|c| !c.is_whitespace()); // eliminate all spaces
        let accept_states = buffer.split(',').collect::<Vec<&str>>();
        for state in accept_states.iter() {
            let state_int = state.parse::<usize>()?;
            if state_int >= nfa.state_num {
                return Err(LoadError::FileFormat(
                    "Accept state out of range".to_string(),
                ));
            }
            nfa.accept_states.set(state_int, true);
        }
        buffer.clear();

        fin.read_line(&mut buffer)?;
        if buffer.trim() != "END" {
            return Err(LoadError::MissingEnd);
        }
        buffer.clear();

        Ok(nfa)
    }

    fn bitvec_to_string(bitvec: &BitVec) -> String {
        let mut string = String::new();
        string.push_str("[");
        for i in 0..bitvec.len() {
            if bitvec[i] {
                string.push_str(&i.to_string());
                string.push_str(",");
            }
        }
        string.pop();
        string.push_str("]");
        string
    }

    pub fn is_accepted(self, input: &str, verbose: bool) -> bool {
        let mut current_states = BitVec::from_elem(self.state_num, false);
        let mut next_states: BitVec;
        current_states.set(self.start_state, true);

        if verbose {
            println!(
                "  ({}, \"{}\")",
                NFA::bitvec_to_string(&current_states),
                input
            );
        }

        for (i, c) in input.chars().enumerate() {
            if c == '_' || !self.alphabet.contains(&c) {
                return false;
            }
            next_states = BitVec::from_elem(self.state_num, false);
            for i in 0..self.state_num {
                if current_states[i] {
                    if let Some(states) = self.transitions[i].get(&'_') {
                        next_states.or(states);
                    } else {
                        panic!("BitVec does not exist for specified state and letter");
                    }
                }
            }
            current_states = next_states;
            if verbose {
                println!(
                    "|-({}, \"{}\")",
                    NFA::bitvec_to_string(&current_states),
                    &input[i..]
                );
            }
            next_states = BitVec::from_elem(self.state_num, false);
            for i in 0..self.state_num {
                if current_states[i] {
                    if let Some(states) = self.transitions[i].get(&c) {
                        next_states.or(states);
                    } else {
                        panic!("BitVec does not exist for specified state and letter");
                    }
                }
            }
            current_states = next_states;
            if verbose {
                println!(
                    "|-({}, \"{}\")",
                    NFA::bitvec_to_string(&current_states),
                    &input[i + 1..]
                );
            }
        }
        next_states = BitVec::from_elem(self.state_num, false);
        for i in 0..self.state_num {
            if current_states[i] {
                if let Some(states) = self.transitions[i].get(&'_') {
                    next_states.or(states);
                } else {
                    panic!("BitVec does not exist for specified state and letter");
                }
            }
        }
        current_states = next_states;
        if verbose {
            println!("|-({}, \"{}\")", NFA::bitvec_to_string(&current_states), "");
        }

        current_states.and(&self.accept_states);
        if current_states.count_ones() > 0 {
            true
        } else {
            false
        }
    }
}
