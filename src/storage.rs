use std::collections::HashMap;
use twox_hash::RandomXxHashBuilder;
use graph::Value;
use typed_arena::Arena;

use petri_net::*;

pub type MarkingId = usize;

pub struct MarkingSet<'a> {
    storage: &'a Arena<Marking>,
    markings: Vec<&'a Marking>,
    id_map: HashMap<&'a Marking, MarkingId, RandomXxHashBuilder>
}

impl <'a> MarkingSet<'a> {

    pub fn new<'b>(arena: &'b Arena<Marking>) -> MarkingSet<'b> {
        MarkingSet { storage: arena, markings: Vec::new(), id_map: Default::default() }
    }

    pub fn insert(&mut self, marking: &Marking) -> MarkingId {
        if self.id_map.contains_key(marking) {
            self.id_map[marking]
        } else {
            let marking_ref = self.storage.alloc(marking.clone());
            let new_id = self.markings.len();
            if new_id % 100000 == 0 {
                println!("Markings: {:?}", new_id);
            }
            self.markings.push(marking_ref);
            self.id_map.insert(marking_ref, new_id);
            new_id
        }
    }

    pub fn get(&self, id: MarkingId) -> &Marking {
        &self.markings[id]
    }

}

///Used to store previously computed successors
pub struct SuccessorCache {
    successors: Vec<Vec<MarkingId>>,
    next_transition: Vec<usize>,
}

impl SuccessorCache {

    pub fn new() -> SuccessorCache {
        SuccessorCache { successors: Vec::new(), next_transition: Vec::new() }
    }

    ///Get successor with given index assuming that successor is cached.
    pub fn get(&self, marking: MarkingId, index: usize) -> Option<MarkingId> {
        self.successors.get(marking).and_then(|v| v.get(index)).map(|s| s.clone())
    }

    ///Notify the cache that next transition has been explored
    pub fn pop_transition(&mut self, marking: MarkingId) -> usize {
        if marking >= self.next_transition.len() {
            self.next_transition.resize(marking + 1, 0);
        }
        let t = self.next_transition[marking];
        self.next_transition[marking] += 1;
        t
    }

    ///Asociate successor with a marking
    pub fn push_successor(&mut self, marking: MarkingId, successor: MarkingId) {
        if marking >= self.successors.len() {
            self.successors.resize(marking + 1, Vec::new());
        }
        self.successors[marking].push(successor);
    }

}

#[derive(Debug, Clone)]
pub struct AssignmentSet {
    assignment: Vec<Value>
}

impl AssignmentSet {

    pub fn new() -> AssignmentSet {
        AssignmentSet { assignment : Vec::new() }
    }

    pub fn get(&self, id: MarkingId) -> Value {
        if id < self.assignment.len() {
            self.assignment[id].clone()
        } else {
            Value::Unknown
        }
    }

    pub fn set(&mut self, id: MarkingId, value: Value) {
        if id >= self.assignment.len() {
            self.assignment.resize(id + 1, Value::Unknown);
        }
        self.assignment[id] = value;
    }
}
