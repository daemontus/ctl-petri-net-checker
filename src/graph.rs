use query::*;
use query::Operator::*;
use petri_net::*;

use successors::*;
use storage::*;
use graph::Value::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Value { One, Zero, Unknown }

pub struct Graph<'a> {
    //first dimension is indexed by queries, second by markings
    assignments: Vec<AssignmentSet>,
    pub markings: &'a mut MarkingSet<'a>,
    pub cache: SuccessorCache,
    pub net: &'a PetriNet,
}

impl <'a> Graph<'a> {

    pub fn new<'b>(net: &'b PetriNet, markings: &'b mut MarkingSet<'b>) -> Graph<'b> {
        Graph { assignments: vec![], markings: markings, cache: SuccessorCache::new(), net: net }
    }

    pub fn search<S: Successors>(&mut self, query: &Query) -> bool {
        self.assignments.clear();
        self.assignments.resize(query.id + 1, AssignmentSet::new());
        let id = self.markings.insert(&self.net.initial_marking);
        self.search_inner::<S>(id, query)
    }

    fn search_inner<S: Successors>(&mut self, root_id: MarkingId, query: &Query) -> bool {
        let q_id = query.id;
        //Note: This simple cache actually helps A LOT (25% speed-up)
        let mut marking_cache = self.net.initial_marking.clone();
        macro_rules! next {
            ($inner:ident, $all:expr) => {{
                let mut succ = S::new();
                while let Some(next_id) = succ.pop(root_id, self, &mut marking_cache) {
                    if $all != self.search_inner::<S>(next_id, $inner) {
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
                    let mut stack: Vec<(MarkingId, S)> = vec![(root_id, S::new())]; //DFS stack
                    let mut visited: Vec<MarkingId> = vec![root_id];
                    while let Some((source_id, mut succ)) = stack.pop() {
                        macro_rules! found_it { () => {{
                                self.assignments[q_id].set(source_id, One);
                                for s in &visited {
                                    self.assignments[q_id].set(*s, One)
                                };
                                return true;
                        }}}
                        if self.assignments[q_id].get(source_id) == Unknown &&
                            self.search_inner::<S>(source_id, $reach) {
                            found_it![];
                        } else {
                            self.assignments[q_id].set(source_id, Zero);
                            if $until && !self.search_inner::<S>(source_id, $path) {
                                continue;
                            }
                            while let Some(next_id) = succ.pop(source_id, self, &mut marking_cache) {
                                match self.assignments[q_id].get(next_id) {
                                    Zero => continue,       //skip!
                                    One => found_it![],     //found something true from previous run
                                    Unknown => {            //we have to go deeper!
                                        stack.push((source_id, succ));   //repush this config so that we can return to it
                                        stack.push((next_id, S::new()));
                                        visited.push(next_id);
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
                    let mut stack: Vec<(MarkingId, S)> = vec![(root_id, S::new())];
                    while let Some((source_id, mut succ)) = stack.pop() {
                        if self.assignments[q_id].get(source_id) == Unknown &&
                            self.search_inner::<S>(source_id, $reach) {
                            self.assignments[q_id].set(source_id, One);
                        } else {
                            self.assignments[q_id].set(source_id, Zero);
                            if $until && !self.search_inner::<S>(source_id, $path) {
                                continue;
                            }
                            let mut all_one = true;
                            let mut not_empty = false;
                            while let Some(next_id) = succ.pop(source_id, self, &mut marking_cache) {
                                not_empty = true;
                                match self.assignments[q_id].get(next_id) {
                                    Zero => return false,   //dead end
                                    One => continue,        //found something true from previous run
                                    Unknown => {            //we have to go deeper!
                                        all_one = false;
                                        succ.repeat_last();
                                        stack.push((source_id, succ));  //repush this config so that we can return to it
                                        stack.push((next_id, S::new()));
                                        break;
                                    }
                                }
                            }   //else: no more successors, config stays zero
                            if all_one && not_empty {
                                self.assignments[q_id].set(source_id, One);
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
            Not(ref inner) => !self.search_inner::<S>(root_id, inner),
            And(ref items) => items.into_iter().all(|i| self.search_inner::<S>(root_id, i)),
            Or(ref items) => items.into_iter().any(|i| self.search_inner::<S>(root_id, i)),
            EX(ref inner) => next![inner, false],
            AX(ref inner) => next![inner, true],
            EF(ref inner) => exists_path![inner],
            EU(ref path, ref reach) => exists_path![reach, path],
            AF(ref inner) => all_paths![inner],
            AU(ref path, ref reach) => all_paths![reach, path],
        }
    }

}
