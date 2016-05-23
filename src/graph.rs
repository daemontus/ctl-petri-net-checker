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

//https://github.com/nrc/r4cppp/blob/master/graphs/README.md
/*
type MarkingId = usize;
type QueryId = usize;
type ConfigId = (MarkingId, QueryId);

type Edge = Vec<ConfigId>;

#[derive(Debug, Clone, Eq, PartialEq)]
enum Assignment {
    One, Zero, Pending, Unknown
}

pub fn search(
    net: &PetriNet, query: &Query,
    initial: &Marking,
    values: &mut AssignmentMap) -> bool {
    match query {
        &Atom(ref proposition) -> {
            proposition()
        }
    }
}

pub fn search_ef(
    net: &PetriNet, query: &Query,
    initial: &Marking, initial_id: MarkingId,
    values: &mut AssignmentMap
) -> bool {
    if (values[query][initial_id] == Assignment::Unknown) {
        let mut list: Vec<(MarkingId, Successors) = Vec::new();
        let mut working: Marking = Vec::new();
        while let Some((mut m_id, mut succ)) = list.pop() {
            let ref marking = values.find_marking(m_id);
            if (succ.next(&mut working)) {
                let id = values.insert(working);
                match values[query][working] {
                    One -> {
                        // found it!
                        while let Some((id, _)) = list.pop() {
                            values[query][id] = One
                        }
                    }
                    Zero, Pending -> { } // skip!
                    Unknown -> {
                        //explore
                        if (search(net, query.inner, &working, id, values)) {
                            values[query][id] = One
                            while let Some((id, _)) = list.pop() {
                                values[query][id] = One
                            }
                        }
                    }
                }
            } else {
                //no successors
                values[query][id] = Zero
            }
        }
    }
    values[query][initial_id] == Assignment::One
}

pub fn search(net: &PetriNet, query: &Query) -> bool {
    let mut map = AssignmentMap::new();
    let mut list: Vec<(ConfigId, Vec<Edge>)> = Vec::new();
    let root = map.insert(&net.initial_marking, query);
    {
        let mut succ = successors(net, &mut map, &root);
        list.push((root, succ));
    }
    while let Some((mut source, mut succ)) = list.pop() {
        //TODO: For negation, flip logic
        if let Some(mut e) = succ.pop() {
            //we have an edge, try propagate one
            if let Some(target) = e.pop() {
                //we have a target, investigate
                match map[target] {
                    Assignment::One => { succ.push(e); } //one = push edge back without the target
                    Assignment::Zero => { e.push(target); } //zero = do nothing (edge is removed)
                    Assignment::Pending => { e.push(target); } //pending = edge is useless (cycle)
                    Assignment::Unknown => {
                        //Unknown = explore further
                        let mut succ = successors(net, &mut map, &root);
                        list.push((target, succ));
                    }
                }
            } else {
                //no targets, one by default
                map[source] = Assignment::One;
            }
        } else {
            //no edges to propagate one, assign 0
            map[source] = Assignment::Zero;
        }
    }
    map[root] == Assignment::One
}

fn successors(net: &PetriNet, map: &mut AssignmentMap, config: &ConfigId) -> Vec<Edge> {
    Vec::new()
}

struct AssignmentMap {
    storage: Vec<Vec<Assignment>>,
    markings: Vec<Marking>,
    ids: HashMap<Marking, MarkingId>,
}

impl Index<ConfigId> for AssignmentMap {

    type Output = Assignment;

    fn index(&self, index: ConfigId) -> &Assignment {
        &self.storage[index.0][index.1]
    }

}

impl <'a> Index<&'a ConfigId> for AssignmentMap {
    type Output = Assignment;
    fn index(&self, index: &ConfigId) -> &Assignment {
        &self.storage[index.0][index.1]
    }
}

impl <'a> Index<&'a mut ConfigId> for AssignmentMap {
    type Output = Assignment;
    fn index(&self, index: &mut ConfigId) -> &Assignment {
        &self.storage[index.0][index.1]
    }
}

impl Index<MarkingId> for AssignmentMap {

    type Output = Marking;

    fn index(&self, index: MarkingId) -> &Marking {
        &self.markings[index]
    }

}

impl IndexMut<ConfigId> for AssignmentMap {
    fn index_mut(&mut self, index: ConfigId) -> &mut Assignment {
        &mut self.storage[index.0][index.1]
    }
}

impl <'a> IndexMut<&'a ConfigId> for AssignmentMap {
    fn index_mut(&mut self, index: &ConfigId) -> &mut Assignment {
        &mut self.storage[index.0][index.1]
    }
}

impl <'a> IndexMut<&'a mut ConfigId> for AssignmentMap {
    fn index_mut(&mut self, index: &mut ConfigId) -> &mut Assignment {
        &mut self.storage[index.0][index.1]
    }
}

impl AssignmentMap {

    fn new() -> AssignmentMap {
        AssignmentMap { storage: Vec::new(), markings: Vec::new(), ids: HashMap::new() }
    }

    fn insert(&mut self, marking: &Marking, query: &Query) -> ConfigId {
        if self.ids.contains_key(marking) {
            (self.ids[marking], get_query_id(query))
        } else {
            let marking_id = self.storage.len();
            self.markings.push(marking.clone());
            self.ids.insert(marking.clone(), marking_id);
            self.storage.push(vec![Assignment::Unknown; max_query_id()]);
            (marking_id, get_query_id(query))
        }
    }
}

fn get_query_id(query: &Query) -> QueryId {
    0
}

fn max_query_id() -> QueryId {
    1
}

/*
type C i32;


enum ConfigType<'a> {
    One, Zero, Unknown,
    Active {
        notify: Vec<&'a Config<'a>>,
        successors: Vec<&'a Edge<'a>>
    }
}

struct Config<'a> {
    marking: Marking,   //TODO this is a copy, make it a reference
    //query: Query,
    value: ConfigType<'a>,
}

enum Edge<'a> {
    Negation { target: &'a Config<'a> },
    Hyper { targets: Vec<&'a Config<'a>> }
}

struct Graph<'a> {
    net: PetriNet,
    //query: &Query<'a>,
    node_set: HashMap<Marking, Vec<Config<'a>>>
}

impl <'a> Graph<'a> {

    fn explore(&'a mut self, config: &'a mut Config<'a>) {

    }

    fn explore_ef(&'a mut self, config: &'a mut Config<'a>) {
        let mut successors = vec![];
        for t in &self.net.matrix {
            if let Some(s) = self.apply_transition(&config.marking, &t) {
                successors.push(s);
            }
        }
        let mut configs = vec![];
        let nodes = &mut self.node_set;
        for marking in &successors {
            if let Some(c) = nodes.get_mut(marking) {
                let new_config = Config { marking: marking.clone(), value: ConfigType::Unknown };
                c.push(new_config);
                let last: &Config<'a> = c.last().unwrap();
                //allow the reference to escape the lifeline of this closure
                //unsafe { mem::transmute::<&Config, &Config>(last) }
                configs.push(last);
            }

        }
        /*let configs: Vec<&'a Config<'a>> = successors.iter().map(|marking| {
            let mut c: &mut Vec<Config<'a>> = self.node_set.entry(marking.clone()).or_insert(vec![]);
            //TODO find the right config if it already exists
            let new_config = Config { marking: marking.clone(), value: ConfigType::Unknown };
            c.push(new_config);
            let last: &Config<'a> = c.last().unwrap();
            //allow the reference to escape the lifeline of this closure
            //unsafe { mem::transmute::<&Config, &Config>(last) }
            last
        }).collect();*/

        /*let suc = config.marking.iter()
            .zip(&t).fold(Some(vec![]), |acc, it| {
                acc.and_then(|new| {
                    let (current, (remove, add)) = it;
                    if remove <= current {
                        Some(new += current - remove + add)
                    } else {
                        None
                    }
                })
            });*/
    }

    fn apply_transition(&'a self, m: &Marking, t: &(Marking, Marking)) -> Option<Marking> {
        let mut removed = m.iter().zip(&t.0).map(|it| {
            let (current, remove) = it;
            current - remove
        });
        if removed.all(|x| x >= 0) {
            Some(removed.zip(&t.1).map(|it| {
                let (current, add) = it;
                current + add
            }).collect::<Marking>())
        } else {
            None
        }
    }

}
/*
pub struct Arena<T> {
    chunks: RefCell<ChunkList<T>>,
}

struct ChunkList<T> {
    current: Vec<T>,
    rest: Vec<Vec<T>>,
}

impl<T> Arena<T> {
    pub fn new() -> Arena<T> {
        let size = cmp::max(1, mem::size_of::<T>());
        Arena::with_capacity(INITIAL_SIZE / size)
    }

    pub fn with_capacity(n: usize) -> Arena<T> {
        let n = cmp::max(MIN_CAPACITY, n);
        Arena {
            chunks: RefCell::new(ChunkList {
                current: Vec::with_capacity(n),
                rest: vec![]
            }),
        }
    }

    pub fn alloc(&self, value: T) -> &mut T {
        let mut chunks = self.chunks.borrow_mut();

        // At this point, the current chunk must have free capacity.
        let next_item_index = chunks.current.len();
        chunks.current.push(value);
        let new_item_ref = {
            let new_item_ref = &mut chunks.current[next_item_index];

            // Extend the lifetime from that of `chunks_borrow` to that of `self`.
            // This is OK because we’re careful to never move items
            // by never pushing to inner `Vec`s beyond their initial capacity.
            // The returned reference is unique (`&mut`):
            // the `Arena` never gives away references to existing items.
            unsafe { mem::transmute::<&mut T, &mut T>(new_item_ref) }
        };

        if chunks.current.len() == chunks.current.capacity() {
            chunks.grow();
        }

        new_item_ref
    }

    pub fn into_vec(self) -> Vec<T> {
        let mut chunks = self.chunks.into_inner();
        // keep order of allocation in the resulting Vec
        let n = chunks.rest.iter().fold(chunks.current.len(), |a, v| a + v.len());
        let mut result = Vec::with_capacity(n);
        for mut vec in chunks.rest {
            result.append(&mut vec);
        }
        result.append(&mut chunks.current);
        result
    }
}

impl<T> ChunkList<T> {
    #[inline(never)]
    #[cold]
    fn grow(&mut self) {
        // Replace the current chunk with a newly allocated chunk.
        let new_capacity = self.current.capacity().checked_mul(2).unwrap();
        let chunk = mem::replace(&mut self.current, Vec::with_capacity(new_capacity));
        self.rest.push(chunk);
    }
}
*/
*/
*/
