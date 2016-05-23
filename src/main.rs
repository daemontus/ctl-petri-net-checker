extern crate xml;
extern crate typed_arena;

mod ctl;
mod ctl_parser;
mod xml_util;
mod petri_net;
mod petri_parser;
mod query;
mod graph;
mod marking_set;

use ctl_parser::*;
use petri_parser::*;
use xml_util::parse_file;
use std::env;
use query::*;
use graph::*;
use marking_set::*;

fn main() {
    let args = env::args().collect::<Vec<String>>();
    if args.len() < 4 {
        panic!("Expecting two arguments: model file, query file and query number");
    }
    let petri_net = parse_file(&args[1], read_net);
    let formulas = parse_file(&args[2], read_set);
    let query_num: isize = args[3].parse().unwrap();
    let mut markings = MarkingSet::new();
    if query_num >= 0 {
        let (query, _) = Query::from_formula(&formulas[query_num as usize], &petri_net, 0);
        println!("Query: {:?}", query);
        let mut graph = Graph::new(&query, &mut markings);
        println!("Result: {:?}", graph.search(&petri_net, &query));
    } else {
        //batch
        for formula in formulas {
            let (query, _) = Query::from_formula(&formula, &petri_net, 0);
            println!("Query: {:?}", query);
            let mut graph = Graph::new(&query, &mut markings);
            println!("Result: {:?}", graph.search(&petri_net, &query));
        }
    }
    //search(&petri_net, &query);
}
