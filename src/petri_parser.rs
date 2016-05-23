use std::io::Read;
use xml::reader::EventReader;
use xml::reader::XmlEvent::*;
use xml::attribute::OwnedAttribute;
use xml_util::*;
use petri_net::*;
use petri_net::PetriParse::*;
use std::collections::HashMap;

//Read a petri net from parser (creating the transition matrix etc.)
pub fn read_net<T: Read>(parser: &mut EventReader<T>) -> PetriNet {
    let events: Vec<PetriParse> = collect_inside("net", parser, |p, event| {
        match event {
            &StartElement { ref name, ref attributes, .. } if name.local_name == "place" => {
                Some(read_place(attributes, p))
            },
            &StartElement { ref name, ref attributes, .. } if name.local_name == "transition" => {
                Some(read_transition(attributes))
            },
            &StartElement { ref name, ref attributes, .. } if name.local_name == "arc" => {
                Some(read_arc(attributes, p))
            }
            _ => None
        }
    });

    let initial_marking = events.iter()
        .filter_map(|e| match e { &Place { ref initial, .. } => Some(initial.clone()), _ => None })
        .collect::<Vec<i32>>();

    let places = events.iter()
        .filter_map(|e| match e { &Place { ref name, .. } => Some(name.clone()), _ => None })
        .enumerate().map(|(a, b)| (b, a))
        .collect::<HashMap<String, usize>>();

    let transitions = events.iter()
        .filter_map(|e| match e { &Transition { ref name } => Some(name.clone()), _ => None })
        .enumerate().map(|(a, b)| (b, a))
        .collect::<HashMap<String, usize>>();

    let mut matrix = vec![(vec![0; places.len()], vec![0; places.len()]); transitions.len()];

    for event in events {
        match event {
            Arc { from, to, value } => {
                if let Some(source_place) = places.get(&*from) {
                    //from place to transition
                    if let Some(target_transition) = transitions.get(&*to) {
                        matrix[target_transition.clone()].0[source_place.clone()] = value;
                    } else {
                        panic!("Unknown arc target transition {}", to);
                    }
                } else {
                    if let Some(source_transition) = transitions.get(&*from) {
                        //from transtion to place
                        if let Some(target_place) = places.get(&*to) {
                            matrix[source_transition.clone()].1[target_place.clone()] = value;
                        } else {
                            panic!("Unknown arg target place {}", to);
                        }
                    } else {
                        panic!("Unknown arc source {}", from);
                    }
                }
            }
            _ => {}
        }
    }

    println!("Places: {:?}", places);
    println!("Transitions: {:?}", transitions);
    println!("Initial marking: {:?}", initial_marking);

    PetriNet {
        places: places,
        transitions: transitions,
        initial_marking: initial_marking,
        matrix: matrix,
    }
}


fn read_place<T: Read>(attributes: &Vec<OwnedAttribute>, parser: &mut EventReader<T>) -> PetriParse {
    if let Some(id_attr) = attributes.iter().find(|a| a.name.local_name == "id") {
        if let Some(initial) = find_until("initialMarking", "place", parser, next_text) {
            if let Ok(count) = initial.parse() {
                Place { name: id_attr.value.clone(), initial: count }
            } else {
                panic!("Can't parse initial marking");
            }
        } else {
            Place { name: id_attr.value.clone(), initial: 0 }
        }
    } else {
        panic!("Place has no name");
    }
}

fn read_transition(attributes: &Vec<OwnedAttribute>) -> PetriParse {
    if let Some(name) = attributes.iter().find(|a| a.name.local_name == "id") {
        Transition { name: name.value.clone() }
    } else {
        panic!("Transition has no name");
    }
}

fn read_arc<T: Read>(attributes: &Vec<OwnedAttribute>, parser: &mut EventReader<T>) -> PetriParse {
    if let Some(source_attr) = attributes.iter().find(|a| a.name.local_name == "source") {
        if let Some(target_attr) = attributes.iter().find(|a| a.name.local_name == "target") {
            if let Some(inscription) = find_until("inscription", "arc", parser, next_text) {
                if let Ok(value) = inscription.parse() {
                    Arc { from: source_attr.value.clone(), to: target_attr.value.clone(), value: value }
                } else {
                    panic!("Can't parse integer");
                }
            } else {
                Arc { from: source_attr.value.clone(), to: target_attr.value.clone(), value: 1 }
            }
        } else {
            panic!("Arc has no target");
        }
    } else {
        panic!("Arc has no source");
    }
}
