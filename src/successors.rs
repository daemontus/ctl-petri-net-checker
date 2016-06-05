use graph::Graph;
use storage::MarkingId;
use petri_net::Marking;

///A successor iterator. Note that the iterator doesn't have access to the marking or even the
///marking id of the source. This is mainly to simplify the API.
///Also note that the pop method takes a marking cache which greatly reduces allocation pressure
///during computation. However, not that the iterator is not obliged to use it and therefore
///one should not rely on the contenst of this variable.
pub trait Successors {
    ///Create new iterator
    fn new() -> Self;
    ///Find next enabled transition, fire it, save the new marking into the graph and return it's id
    fn pop(&mut self, source_id: MarkingId, graph: &mut Graph, cache: &mut Marking) -> Option<MarkingId>;
    ///Repeat the last encountered successor marking
    fn repeat_last(&mut self);
}

///Fully On-the-fly implementation of successor iterator
pub struct OTFSuccessors {
    next_transition: usize,
}

impl Successors for OTFSuccessors {

    fn new() -> OTFSuccessors {
        OTFSuccessors { next_transition: 0 }
    }

    fn repeat_last(&mut self) {
        if self.next_transition == 0 {
            panic!("No last!");
        } else {
            self.next_transition -= 1;
        }
    }

    fn pop(&mut self, source_id: MarkingId, graph: &mut Graph, cache: &mut Marking) -> Option<MarkingId> {
        while self.next_transition < graph.net.matrix.len() {
            let mut valid = true;
            {
                let ref transition = graph.net.matrix[self.next_transition];
                let ref source = graph.markings.get(source_id);
                for i in 0..source.len() {
                    if source[i] >= transition.0[i] {
                        cache[i] = source[i] - transition.0[i] + transition.1[i];
                    } else {
                        valid = false;
                        break;
                    }
                }
            } // end source borrow
            self.next_transition += 1;
            if valid {
                let id = graph.markings.insert(&cache);
                return Some(id);
            }
        }
        None
    }
}

/*
pub struct Successors<S> {
    pub source_id: MarkingId,
    function: S,
}

impl <S: SuccessorFunction> Successors<S> {

    pub fn new(source_id: MarkingId) -> Successors<S> {
        Successors { source_id: source_id, function: S::new() }
    }

    pub fn repeat_last(&mut self) {
        self.function.repeat_last();
        /*if self.cache_index == 0 {
            panic!("No last!");
        } else {
            self.cache_index -= 1;
        }*/
    }

    pub fn pop(&mut self, dest: &mut Marking, net: &PetriNet, markings: &mut MarkingSet) -> Option<MarkingId> {
        self.function.pop(self.source_id, dest, net, markings)
        /*if self.cache_index < markings.successors[self.source_id].len() {
            self.cache_index += 1;
            return Some(markings.successors[self.source_id][self.cache_index - 1]);
        } else {
            let mut next_transition = markings.computed[self.source_id];
            while next_transition < net.matrix.len() {
                let mut valid = true;
                {
                    let ref transition = net.matrix[next_transition];
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
                next_transition += 1;
                if valid {
                    let id = markings.insert(dest);
                    markings.successors[self.source_id].push(id.clone());
                    markings.computed[self.source_id] = next_transition;
                    self.cache_index += 1;
                    return Some(id);
                }
            }
        }
        None*/
    }
}
*/
