use std::collections::HashMap;

use pnml::pt_net::Net;
use pnml::pt_net::Element::*;

use storage::*;

pub type Marking = Vec<u32>;

#[derive(Debug, PartialEq, Eq)]
pub struct PetriNet {
    pub places: HashMap<String, usize>,
    pub transitions: HashMap<String, usize>,
    pub initial_marking: Marking,
    //first vector are incoming arcs, second outgoing
    pub matrix: Vec<(Vec<u32>,Vec<u32>)>,
}

impl PetriNet {

    pub fn new(pt_net: &Net) -> PetriNet {
        let initial_marking = pt_net.elements.iter()
            .filter_map(|e| match e {
                &Place { ref initial_marking, .. } => Some(initial_marking.clone()), _ => None
            }).collect::<Vec<u32>>();

        let places = pt_net.elements.iter()
            .filter_map(|e| match e { &Place { ref id, .. } => Some(id.clone()), _ => None })
            .enumerate().map(|(a, b)| (b, a))
            .collect::<HashMap<String, usize>>();

        let transitions = pt_net.elements.iter()
            .filter_map(|e| match e { &Transition { ref id } => Some(id.clone()), _ => None })
            .enumerate().map(|(a, b)| (b, a))
            .collect::<HashMap<String, usize>>();

        let mut matrix = vec![(vec![0; places.len()], vec![0; places.len()]); transitions.len()];

        for event in &pt_net.elements {
            match event {
                &Arc { ref source, ref target, ref inscription, .. } => {
                    if let Some(source_place) = places.get(&*source) {
                        //from place to transition
                        if let Some(target_transition) = transitions.get(&*target) {
                            matrix[target_transition.clone()].0[source_place.clone()] = inscription.clone();
                        } else {
                            panic!("Unknown arc target transition {}", target);
                        }
                    } else if let Some(source_transition) = transitions.get(&*source) {
                        //from transtion to place
                        if let Some(target_place) = places.get(&*target) {
                            matrix[source_transition.clone()].1[target_place.clone()] = inscription.clone();
                        } else {
                            panic!("Unknown arg target place {}", target);
                        }
                    } else {
                        panic!("Unknown arc source {}", source);
                    }
                }
                _ => {}
            }
        }

        PetriNet {
            places: places,
            transitions: transitions,
            initial_marking: initial_marking,
            matrix: matrix,
        }
    }

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
