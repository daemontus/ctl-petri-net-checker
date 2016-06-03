extern crate typed_arena;
extern crate pnml;
extern crate ctl;

mod petri_net;
mod query;
mod graph;
mod storage;

use ctl::parser::read_formula_list_file;
use pnml::pt_net::parser::read_pt_file;
use typed_arena::Arena;
use std::env;
use query::*;
use graph::*;
use petri_net::*;
use storage::*;

fn main() {
    let args = env::args().collect::<Vec<String>>();
    if args.len() < 4 {
        panic!("Expecting two arguments: model file, query file and query number");
    }
    let pt_net = read_pt_file(&args[1]);
    let petri_net = PetriNet::new(&pt_net);
    let formulas = read_formula_list_file(&args[2]);
    let query_num: isize = args[3].parse().unwrap();
    let arena = Arena::new();
    let mut markings = MarkingSet::new(&arena);
    if query_num >= 0 {
        let (query, _) = Query::from_formula(&formulas[query_num as usize], &petri_net, 0);
        println!("Query: {:?}", query);
        let mut graph = Graph::new(&query, &mut markings);
        println!("Result: {:?}", graph.search(&petri_net, &query));
    } else {
        //batch
        /*for formula in formulas {
            let (query, _) = Query::from_formula(&formula, &petri_net, 0);
            println!("Query: {:?}", query);
            let mut graph = Graph::new(&query, &mut markings);
            println!("Result: {:?}", graph.search(&petri_net, &query));
        }*/
    }
    //search(&petri_net, &query);
}
