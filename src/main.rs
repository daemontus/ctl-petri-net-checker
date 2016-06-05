extern crate typed_arena;
extern crate clap;
extern crate pnml;
extern crate ctl;
extern crate twox_hash;

mod petri_net;
mod query;
mod graph;
mod storage;
mod successors;

use ctl::parser::read_formula_list_file;
use pnml::pt_net::parser::read_pt_file;
use clap::{Arg, App};
use typed_arena::Arena;
use query::*;
use graph::*;
use petri_net::*;
use storage::*;
use successors::OTFSuccessors;
use successors::CachedSuccessors;

fn main() {
    let matches = App::new("Explicit CTL checker")
                        .version("0.1")
                        .author("Samuel Pastva <daemontus@gmail.com>")
                        .about("Verification tool for petri nets.")
                        .arg(Arg::with_name("model")
                            .short("m").long("model")
                            .value_name("PNML FILE")
                            .required(true)
                            .takes_value(true))
                        .arg(Arg::with_name("queries")
                            .short("q").long("queries")
                            .value_name("XML QUERY FILE")
                            .required(true)
                            .takes_value(true))
                        .arg(Arg::with_name("number")
                            .short("n").long("number")
                            .value_name("QUERY NUMBER")
                            .takes_value(true))
                        .get_matches();
    let pt_net = read_pt_file(matches.value_of("model").unwrap());
    let petri_net = PetriNet::new(&pt_net);
    let formulas = read_formula_list_file(matches.value_of("queries").unwrap());
    let query_num: isize = matches.value_of("number").unwrap_or("-1").parse().unwrap();
    let arena = Arena::new();
    let mut markings = MarkingSet::new(&arena);
    let mut graph = Graph::new(&petri_net, &mut markings);
    if query_num >= 0 {
        let (query, _) = Query::from_formula(&formulas[query_num as usize], &petri_net, 0);
        println!("Query: {:?}", formulas[query_num as usize]);
        println!("Result: {:?}", graph.search::<OTFSuccessors>(&query));
    } else {
        //batch
        for formula in formulas {
            let (query, _) = Query::from_formula(&formula, &petri_net, 0);
            println!("Query: {:?}", formula);
            println!("Result: {:?}", graph.search::<CachedSuccessors>(&query));
        }
    }
}
