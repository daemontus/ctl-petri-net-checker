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

///fire specific transition if possible and save result in dest. If transition is not valid,
///return false and contents of dest are undefined.
fn fire_transition(dest: &mut Marking, source: &Marking, transition: &(Vec<u32>, Vec<u32>)) -> bool {
    let mut valid = true;
    for i in 0..source.len() {
        if source[i] >= transition.0[i] {
            dest[i] = source[i] - transition.0[i] + transition.1[i];
        } else {
            valid = false;
            break;
        }
    }
    valid
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
        self.next_transition = self.next_transition.checked_sub(1).unwrap();
    }

    fn pop(&mut self, source_id: MarkingId, graph: &mut Graph, cache: &mut Marking) -> Option<MarkingId> {
        while self.next_transition < graph.net.matrix.len() {
            let fired;
            {
                let ref transition = graph.net.matrix[self.next_transition];
                let ref source = graph.markings.get(source_id);
                fired = fire_transition(cache, source, transition);
            } //end source borrow so that we can access the graph
            self.next_transition += 1;
            if fired {
                let id = graph.markings.insert(&cache);
                return Some(id);
            }
        }
        None
    }
}

pub struct CachedSuccessors {
    next_index: usize,
}

impl Successors for CachedSuccessors {

    fn new() -> CachedSuccessors {
        CachedSuccessors { next_index: 0 }
    }

    fn repeat_last(&mut self) {
        self.next_index = self.next_index.checked_sub(1).unwrap();
    }

    fn pop(&mut self, source_id: MarkingId, graph: &mut Graph, cache: &mut Marking) -> Option<MarkingId> {
        if let Some(id) = graph.cache.get(source_id, self.next_index) {
            self.next_index += 1;
            return Some(id);
        } else {
            let mut next_transition = graph.cache.pop_transition(source_id);
            while next_transition < graph.net.matrix.len() {
                let fired;
                {
                    let ref transition = graph.net.matrix[next_transition];
                    let ref source = graph.markings.get(source_id);
                    fired = fire_transition(cache, source, transition)
                } //end source borrow so that we can access the graph
                if fired {
                    let id = graph.markings.insert(cache);
                    graph.cache.push_successor(source_id, id);
                    self.next_index += 1;
                    return Some(id);
                }
                next_transition = graph.cache.pop_transition(source_id);
            }
        }
        None
    }
}
