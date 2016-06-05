use std::collections::HashMap;

use pnml::pt_net::Net;
use pnml::pt_net::Element::*;

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
