use std::collections::HashMap;

use marking_set::*;

pub type Marking = Vec<i32>;

#[derive(Debug, PartialEq, Eq)]
pub enum PetriParse {
    Place { name: String, initial: i32, },
    Transition { name: String },
    Arc { from: String, to: String, value: i32 },
}

#[derive(Debug, PartialEq, Eq)]
pub struct PetriNet {
    pub places: HashMap<String, usize>,
    pub transitions: HashMap<String, usize>,
    pub initial_marking: Marking,
    //first vector are incoming arcs, sencond outgoing
    pub matrix: Vec<(Vec<i32>,Vec<i32>)>,
}

pub struct Successors {
    pub source_id: MarkingId,
    current: usize,
}

impl Successors {

    pub fn new(source_id: MarkingId) -> Successors {
        Successors { source_id: source_id, current: 0 }
    }

    pub fn repeat_last(&mut self) {
        if self.current == 0 {
            panic!("No last!");
        } else {
            self.current -= 1;
        }
    }

    pub fn pop(&mut self, dest: &mut Marking, net: &PetriNet, markings: &mut MarkingSet) -> Option<MarkingId> {
        while self.current < net.matrix.len() {
            let mut valid = true;
            {
                let ref transition = net.matrix[self.current];
                let ref source = markings.get(self.source_id);
                for i in 0..dest.len() {
                    if source[i] >= transition.0[i] {
                        dest[i] = source[i] - transition.0[i] + transition.1[i];
                    } else {
                        valid = false;
                        break;
                    }
                }
            } // end source borrow
            self.current += 1;
            if valid {
                let id = markings.insert(dest);
                return Some(id);
            }
        }
        None
    }
}
