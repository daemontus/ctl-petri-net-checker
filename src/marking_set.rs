use std::collections::HashMap;
use graph::Value;
use typed_arena::Arena;

use petri_net::*;

pub type MarkingId = usize;

pub struct MarkingSet {
    storage: Arena<Marking>,
    markings: Vec<Marking>,
    id_map: HashMap<Marking, MarkingId>
}

impl MarkingSet {

    pub fn new() -> MarkingSet {
        MarkingSet { markings: Vec::new(), id_map: HashMap::new() }
    }

    pub fn insert(&mut self, marking: &Marking) -> MarkingId {
        if self.id_map.contains_key(marking) {
            self.id_map[marking]
        } else {
            let new_id = self.markings.len();
            if new_id % 100000 == 0 {
                println!("Markings: {:?}", new_id);
            }
            self.markings.push(marking.clone());
            self.id_map.insert(marking.clone(), new_id);
            new_id
        }
    }

    pub fn get(&self, id: MarkingId) -> &Marking {
        &self.markings[id]
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
