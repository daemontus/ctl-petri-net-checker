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
    markings: &'a mut MarkingSet<'a>,
}

impl <'a> Graph<'a> {

    pub fn new<'b>(query: &Query, markings: &'b mut MarkingSet<'b>) -> Graph<'b> {
        Graph { assignments: vec![AssignmentSet::new(); query.id + 1], markings: markings }
    }

    pub fn search(&mut self, net: &PetriNet, query: &Query) -> bool {
        let id = self.markings.insert(&net.initial_marking);
        self.search_inner(net, id, query)
    }

    fn search_inner(&mut self, net: &PetriNet, root_id: MarkingId, query: &Query) -> bool {
        let q_id = query.id;
        let mut working: Marking = net.initial_marking.clone(); //it has to be the same length
        macro_rules! next {
            ($inner:ident, $all:expr) => {{
                let mut succ = Successors::new(root_id);
                while let Some(next_id) = succ.pop(&mut working, net, &mut self.markings) {
                    if $all != self.search_inner(net, next_id, $inner) {
                        return !$all;
                    }
                }
                $all
            }};
        }
        macro_rules! exists_path {
            ($reach:ident) => (exists_path![$reach, false, $reach]);
            ($reach:ident, $path:ident) => (exists_path![$reach, true, $path]);
            ($reach:ident, $until: expr, $path:ident) => {{
                if self.assignments[q_id].get(root_id) == Unknown {
                    let mut stack: Vec<Successors> = vec![Successors::new(root_id)]; //DFS stack
                    while let Some(mut succ) = stack.pop() {
                        macro_rules! found_it { () => {{
                                self.assignments[q_id].set(succ.source_id, One);
                                for s in &stack { self.assignments[q_id].set(s.source_id, One) };
                                return true;
                        }}}
                        if self.assignments[q_id].get(succ.source_id) == Unknown &&
                            self.search_inner(net, succ.source_id, $reach) {
                            found_it![];
                        } else {
                            self.assignments[q_id].set(succ.source_id, Zero);
                            if $until && !self.search_inner(net, succ.source_id, $path) {
                                continue;
                            }
                            while let Some(next_id) = succ.pop(&mut working, net, &mut self.markings) {
                                match self.assignments[q_id].get(next_id) {
                                    Zero => continue,       //skip!
                                    One => found_it![],     //found something true from previous run
                                    Unknown => {            //we have to go deeper!
                                        stack.push(succ);   //repush this config so that we can return to it
                                        stack.push(Successors::new(next_id));
                                        break;
                                    }
                                }
                            }   //else: no successors, config stays zero
                        }
                    }
                }
                self.assignments[q_id].get(root_id) == One
            }}
        }
        macro_rules! all_paths {
            ($reach:ident) => (all_paths![$reach, false, $reach]);
            ($reach:ident, $path:ident) => (all_paths![$reach, true, $path]);
            ($reach:ident, $until:expr, $path:ident) => {{
                if self.assignments[q_id].get(root_id) == Unknown {
                    let mut stack: Vec<Successors> = vec![Successors::new(root_id)];
                    while let Some(mut succ) = stack.pop() {
                        if self.assignments[q_id].get(succ.source_id) == Unknown &&
                            self.search_inner(net, succ.source_id, $reach) {
                            self.assignments[q_id].set(succ.source_id, One);
                        } else {
                            self.assignments[q_id].set(succ.source_id, Zero);
                            if $until && !self.search_inner(net, succ.source_id, $path) {
                                continue;
                            }
                            let mut all_one = true;
                            let mut not_empty = false;
                            let id_copy = succ.source_id;
                            while let Some(next_id) = succ.pop(&mut working, net, &mut self.markings) {
                                not_empty = true;
                                match self.assignments[q_id].get(next_id) {
                                    Zero => return false,   //dead end
                                    One => continue,        //found something true from previous run
                                    Unknown => {            //we have to go deeper!
                                        all_one = false;
                                        succ.repeat_last();
                                        stack.push(succ);  //repush this config so that we can return to it
                                        stack.push(Successors::new(next_id));
                                        break;
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
            }}
        }
        match query.operator {
            //TODO Implement EG/AG as maximum fixed point
            //TODO consider caching the EX/AX answers
            Atom(ref proposition) => proposition(self.markings.get(root_id)),
            Not(ref inner) => !self.search_inner(net, root_id, inner),
            And(ref items) => items.into_iter().all(|i| self.search_inner(net, root_id, i)),
            Or(ref items) => items.into_iter().any(|i| self.search_inner(net, root_id, i)),
            EX(ref inner) => next![inner, false],
            AX(ref inner) => next![inner, true],
            EF(ref inner) => exists_path![inner],
            EU(ref path, ref reach) => exists_path![reach, path],
            AF(ref inner) => all_paths![inner],
            AU(ref path, ref reach) => all_paths![reach, path],
            _ => panic!("Unsupported!"),
        }
    }

}
