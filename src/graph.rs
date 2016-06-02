use query::*;
use query::Operator::*;
use petri_net::*;

use marking_set::*;
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
        match query.operator {
            //TODO Implement EG/AG as maximum fixed point
            Atom(ref proposition) => {  //evaluate proposition
                let ref marking = self.markings.get(root_id);
                let res = proposition(marking);
                //println!("Prop: {:?}", res);
                return res;
            }
            Not(ref inner) => {
                let res = !self.search_inner(net, root_id, inner);
                //println!("Not: {:?}", res);
                return res;
            }
            And(ref left, ref right) => {
                let res = self.search_inner(net, root_id, right) && self.search_inner(net, root_id, left);
                //println!("And: {:?}", res);
                return res;
            }
            Or(ref left, ref right) => {
                let res = self.search_inner(net, root_id, right) || self.search_inner(net, root_id, left);
                //println!("Or: {:?}", res);
                return res;
            }
            EX(ref inner) => {
                if self.assignments[q_id].get(root_id) == Unknown {
                    let mut working: Marking = net.initial_marking.clone(); //it has to be the same length
                    self.assignments[q_id].set(root_id, Zero);
                    let mut config = Successors::new(root_id);
                    while let Some(next_id) = config.pop(&mut working, net, &mut self.markings) {
                        if self.search_inner(net, next_id, inner) {
                            self.assignments[q_id].set(root_id, One);
                            break;
                        }
                    }
                }
                return self.assignments[q_id].get(root_id) == One;
            }
            AX(ref inner) => {
                if self.assignments[q_id].get(root_id) == Unknown {
                    let mut working: Marking = net.initial_marking.clone(); //it has to be the same length
                    self.assignments[q_id].set(root_id, One);
                    let mut config = Successors::new(root_id);
                    while let Some(next_id) = config.pop(&mut working, net, &mut self.markings) {
                        if !self.search_inner(net, next_id, inner) {
                            self.assignments[q_id].set(root_id, Zero);
                            break;
                        }
                    }
                }
                return self.assignments[q_id].get(root_id) == One;
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
        }
        //panic!("Unsupported!");
    }

}
