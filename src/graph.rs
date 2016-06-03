use query::*;
use query::Operator::*;
use petri_net::*;

use storage::*;
use graph::Value::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Value { One, Zero, Unknown }

pub struct Graph<'a> {
    //first dimension is indexed by queries, second by markings
    assignments: Vec<AssignmentSet>,
    markings: &'a mut MarkingSet,
}

impl <'a> Graph<'a> {

    pub fn new<'b>(query: &Query, markings: &'b mut MarkingSet) -> Graph<'b> {
        Graph { assignments: vec![AssignmentSet::new(); query.id + 1], markings: markings }
    }

    pub fn search(&mut self, net: &PetriNet, query: &Query) -> bool {
        let id = self.markings.insert(&net.initial_marking);
        self.search_inner(net, id, query)
    }

    fn search_inner(&mut self, net: &PetriNet, root_id: MarkingId, query: &Query) -> bool {
        let q_id = query.id;
        let mut working: Marking = net.initial_marking.clone(); //it has to be the same length
        match query.operator {
            //TODO Implement EG/AG as maximum fixed point
            //TODO consider caching the EX/AX answers
            Atom(ref proposition) => proposition(self.markings.get(root_id)),
            Not(ref inner) => !self.search_inner(net, root_id, inner),
            And(ref items) => items.into_iter().all(|i| self.search_inner(net, root_id, i)),
            Or(ref items) => items.into_iter().any(|i| self.search_inner(net, root_id, i)),
            EX(ref inner) => {
                let mut successors = Successors::new(root_id);
                while let Some(next_id) = successors.pop(&mut working, net, &mut self.markings) {
                    if self.search_inner(net, next_id, inner) {
                        return true;
                    }
                }
                false
            }
            AX(ref inner) => {
                let mut successors = Successors::new(root_id);
                while let Some(next_id) = successors.pop(&mut working, net, &mut self.markings) {
                    if ! self.search_inner(net, next_id, inner) {
                        return false;
                    }
                }
                true
            }
            EF(ref inner) => {
                if self.assignments[q_id].get(root_id) == Unknown {
                    let mut list: Vec<Successors> = Vec::new();
                    let mut working: Marking = net.initial_marking.clone(); //it has to be the same length
                    list.push(Successors::new(root_id));
                    while let Some(mut config) = list.pop() {
                        if self.assignments[q_id].get(config.source_id) == Unknown && self.search_inner(net, config.source_id, inner) {
                            self.assignments[q_id].set(config.source_id, One);
                            while let Some(config) = list.pop() {
                                self.assignments[q_id].set(config.source_id, One);
                            }
                        } else {
                            self.assignments[q_id].set(config.source_id, Zero);
                            while let Some(next_id) = config.pop(&mut working, net, &mut self.markings) {
                                match self.assignments[q_id].get(next_id) {
                                    Zero => {   //skip!
                                        continue;
                                    }
                                    Unknown => {    //we have to go deeper!
                                        list.push(config);  //repush this config so that we can return to it
                                        list.push(Successors::new(next_id));
                                        break;
                                    }
                                    One => {    //found something true from previous run
                                        self.assignments[q_id].set(config.source_id, One);
                                        while let Some(config) = list.pop() {
                                            self.assignments[q_id].set(config.source_id, One);
                                        }
                                        break;
                                    }
                                }
                            }   //else: no successors, config stays zero
                        }
                    }
                }
                return self.assignments[q_id].get(root_id) == One;
            }
            EU(ref path, ref reach) => {
                if self.assignments[q_id].get(root_id) == Unknown {
                    let mut list: Vec<Successors> = Vec::new();
                    let mut working: Marking = net.initial_marking.clone(); //it has to be the same length
                    list.push(Successors::new(root_id));
                    while let Some(mut config) = list.pop() {
                        if self.assignments[q_id].get(config.source_id) == Unknown && self.search_inner(net, config.source_id, reach) {
                            self.assignments[q_id].set(config.source_id, One);
                            while let Some(config) = list.pop() {
                                self.assignments[q_id].set(config.source_id, One);
                            }
                        } else {
                            self.assignments[q_id].set(config.source_id, Zero);
                            if self.search_inner(net, config.source_id, path) {
                                while let Some(next_id) = config.pop(&mut working, net, &mut self.markings) {
                                    match self.assignments[q_id].get(next_id) {
                                        Zero => {   //skip!
                                            continue;
                                        }
                                        Unknown => {    //we have to go deeper!
                                            list.push(config);  //repush this config so that we can return to it
                                            list.push(Successors::new(next_id));
                                            break;
                                        }
                                        One => {    //found something true from previous run
                                            self.assignments[q_id].set(config.source_id, One);
                                            while let Some(config) = list.pop() {
                                                self.assignments[q_id].set(config.source_id, One);
                                            }
                                            break;
                                        }
                                    }
                                }   //else: no successors, config stays zero
                            }
                        }
                    }
                }
                return self.assignments[q_id].get(root_id) == One;
            }
            AF(ref inner) => {
                if self.assignments[q_id].get(root_id) == Unknown {
                    let mut list: Vec<Successors> = Vec::new();
                    let mut working: Marking = net.initial_marking.clone(); //it has to be the same length
                    list.push(Successors::new(root_id));
                    while let Some(mut config) = list.pop() {
                        if self.assignments[q_id].get(config.source_id) == Unknown && self.search_inner(net, config.source_id, inner) {
                            self.assignments[q_id].set(config.source_id, One);
                        } else {
                            self.assignments[q_id].set(config.source_id, Zero);
                            let mut all_one = true;
                            let mut not_empty = false;
                            let id_copy = config.source_id;
                            while let Some(next_id) = config.pop(&mut working, net, &mut self.markings) {
                                not_empty = true;
                                match self.assignments[q_id].get(next_id) {
                                    Zero => {
                                        return false;
                                    }
                                    Unknown => {    //we have to go deeper!
                                        all_one = false;
                                        config.repeat_last();
                                        list.push(config);  //repush this config so that we can return to it
                                        list.push(Successors::new(next_id));
                                        break;
                                    }
                                    One => {    //found something true from previous run
                                        continue;
                                    }
                                }
                            }   //else: no more successors, config stays zero
                            if all_one && not_empty {
                                self.assignments[q_id].set(id_copy, One);
                            }
                        }
                    }
                }
                return self.assignments[q_id].get(root_id) == One;
            }
            AU(ref path, ref reach) => {
                if self.assignments[q_id].get(root_id) == Unknown {
                    let mut list: Vec<Successors> = Vec::new();
                    let mut working: Marking = net.initial_marking.clone(); //it has to be the same length
                    list.push(Successors::new(root_id));
                    while let Some(mut config) = list.pop() {
                        if self.assignments[q_id].get(config.source_id) == Unknown && self.search_inner(net, config.source_id, reach) {
                            self.assignments[q_id].set(config.source_id, One);
                        } else {
                            self.assignments[q_id].set(config.source_id, Zero);
                            if self.search_inner(net, config.source_id, path) {
                                let mut all_one = true;
                                let mut not_empty = false;
                                let id_copy = config.source_id;
                                while let Some(next_id) = config.pop(&mut working, net, &mut self.markings) {
                                    not_empty = true;
                                    match self.assignments[q_id].get(next_id) {
                                        Zero => {
                                            return false;
                                        }
                                        Unknown => {    //we have to go deeper!
                                            all_one = false;
                                            config.repeat_last();
                                            list.push(config);  //repush this config so that we can return to it
                                            list.push(Successors::new(next_id));
                                            break;
                                        }
                                        One => {    //found something true from previous run
                                            continue;
                                        }
                                    }
                                }   //else: no more successors, config stays zero
                                if all_one && not_empty {
                                    self.assignments[q_id].set(id_copy, One);
                                }
                            }
                        }
                    }
                }
                return self.assignments[q_id].get(root_id) == One;
            }
            _ => panic!("Unsupported!"),
        }
    }

}
