use petri_net::*;
use ctl::*;
use ctl::CTLFormula::*;
use ctl::Value::*;
use std::fmt;
use query::Operator as Op;

type Evaluable = Box<Fn(&Marking) -> i32>;

pub type QueryId = usize;
pub type Proposition = Box<Fn(&Marking) -> bool>;

pub struct Query {
    pub id: QueryId,
    pub operator: Operator
}

pub enum Operator {
    Atom(Proposition),
    Not(Box<Query>),
    EF(Box<Query>),
    AF(Box<Query>),
    EX(Box<Query>),
    AX(Box<Query>),
    And(Box<Query>, Box<Query>),
    Or(Box<Query>, Box<Query>),
    AU(Box<Query>, Box<Query>),
    EU(Box<Query>, Box<Query>),
}

impl fmt::Debug for Operator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Op::Atom(_) => write!(f, "atom"),
            &Op::Not(ref inner) => write!(f, "!({:?})", inner),
            &Op::EF(ref inner) => write!(f, "EF({:?})", inner),
            &Op::AF(ref inner) => write!(f, "AF({:?})", inner),
            &Op::EX(ref inner) => write!(f, "EX({:?})", inner),
            &Op::AX(ref inner) => write!(f, "AX({:?})", inner),
            &Op::And(ref left, ref right) => write!(f, "({:?} && {:?})", left, right),
            &Op::Or(ref left, ref right) => write!(f, "({:?} || {:?})", left, right),
            &Op::EU(ref left, ref right) => write!(f, "({:?} EU {:?})", left, right),
            &Op::AU(ref left, ref right) => write!(f, "({:?} AU {:?})", left, right),
        }
    }
}

impl fmt::Debug for Query {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{:?}: {:?}]", self.id, self.operator)
    }
}

impl Query {

    pub fn from_formula(formula: &CTLFormula, net: &PetriNet, next_id: QueryId) -> (Query, QueryId) {
        match formula {
            &LT(ref left, ref right) => as_atom(as_proposition(left, right, net, create_lt), next_id),
            &LE(ref left, ref right) => as_atom(as_proposition(left, right, net, create_le), next_id),
            &GT(ref left, ref right) => as_atom(as_proposition(left, right, net, create_gt), next_id),
            &GE(ref left, ref right) => as_atom(as_proposition(left, right, net, create_ge), next_id),
            &And(ref left, ref right) => as_binary_query(left, right, net, Op::And, next_id),
            &Or(ref left, ref right) => as_binary_query(left, right, net, Op::Or, next_id),
            &AU(ref left, ref right) => as_binary_query(left, right, net, Op::AU, next_id),
            &EU(ref left, ref right) => as_binary_query(left, right, net, Op::EU, next_id),
            &Not(ref inner) => as_unary_query(inner, net, Op::Not, next_id),
            &AX(ref inner) => as_unary_query(inner, net, Op::AX, next_id),
            &EX(ref inner) => as_unary_query(inner, net, Op::EX, next_id),
            &AF(ref inner) => as_unary_query(inner, net, Op::AF, next_id),
            &EF(ref inner) => as_unary_query(inner, net, Op::EF, next_id),
            &AG(ref inner) => {
                let (inner_not, next_id) = as_unary_query(inner, net, Op::Not, next_id);
                let (ef, next_id) = (Query { id: next_id, operator: Op::EF(Box::new(inner_not)) }, next_id + 1);
                (Query { id: next_id, operator: Op::Not(Box::new(ef)) }, next_id + 1)
            }
            &EG(ref inner) => {
                let (inner_not, next_id) = as_unary_query(inner, net, Op::Not, next_id);
                let (af, next_id) = (Query { id: next_id, operator: Op::AF(Box::new(inner_not)) }, next_id + 1);
                (Query { id: next_id, operator: Op::Not(Box::new(af)) }, next_id + 1)
            }
        }
    }
}

fn as_atom(prop: Proposition, next_id: QueryId) -> (Query, QueryId) {
    (Query { id: next_id, operator: Op::Atom(prop) }, next_id + 1)
}

fn as_binary_query<F>(
    left: &Box<CTLFormula>, right: &Box<CTLFormula>,
    net: &PetriNet, combine: F, next_id: QueryId
) -> (Query, QueryId) where F : Fn(Box<Query>, Box<Query>) -> Operator {
    let (r_query, next_id) = Query::from_formula(&*right, net, next_id);    //right side will have smaller ids
    let (l_query, next_id) = Query::from_formula(&*left, net, next_id);
    (Query { id: next_id, operator: combine(Box::new(l_query), Box::new(r_query)) }, next_id + 1)
}

fn as_unary_query<F>(inner: &Box<CTLFormula>, net: &PetriNet, combine: F, next_id: QueryId) -> (Query, QueryId)
    where F : Fn(Box<Query>) -> Operator {
    let (inner_query, next_id) = Query::from_formula(&*inner, net, next_id);
    (Query { id: next_id, operator: combine(Box::new(inner_query)) }, next_id + 1)
}

fn as_proposition<F>(left: &Value, right: &Value, net: &PetriNet, combine: F) -> Proposition
    where F : Fn(Evaluable, Evaluable) -> Proposition {
    let l_eval = left.as_evaluable(net);
    let r_eval = right.as_evaluable(net);
    combine(l_eval, r_eval)
}

fn create_lt(left: Evaluable, right: Evaluable) -> Proposition {
    Box::new(move |m| left(m) < right(m))
}

fn create_gt(left: Evaluable, right: Evaluable) -> Proposition {
    Box::new(move |m| left(m) > right(m))
}

fn create_le(left: Evaluable, right: Evaluable) -> Proposition {
    Box::new(move |m| left(m) <= right(m))
}

fn create_ge(left: Evaluable, right: Evaluable) -> Proposition {
    Box::new(move |m| left(m) >= right(m))
}

impl Value {
    fn as_evaluable(&self, net: &PetriNet) -> Evaluable {
        match self {
            &Const(v) => Box::new(move |_| v),
            &Ref(ref name) => {
                if let Some(&index) = net.places.get(&*name) {
                    Box::new(move |m| m[index])
                } else {
                    panic!("Place not found: {}", name);
                }
            }
        }
    }
}
