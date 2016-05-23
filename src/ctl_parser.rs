use std::io::Read;
use xml::reader::EventReader;
use ctl::*;
use ctl::CTLFormula::*;
use ctl::Value::*;
use xml_util::*;

//read a list of formulas from parser
pub fn read_set<T : Read>(parser: &mut EventReader<T>) -> Vec<CTLFormula> {
    //read property-set
    collect_inside("property-set", parser, |p, event| {
        expect_start(event, "formula").map(|_| read_formula(p))
    })
}

//formula parsing logic
fn read_formula<T : Read>(parser: &mut EventReader<T>) -> CTLFormula {
    let next_tag = next_tag_open(parser);
    let result = match &*next_tag {
        "integer-le" => atom(LE, parser),
        "integer-ge" => atom(GE, parser),
        "integer-lt" => atom(LT, parser),
        "integer-gt" => atom(GT, parser),
        "negation" => unary_formula(Not, parser),
        "conjunction" => binary_formula(And, parser),
        "disjunction" => binary_formula(Or, parser),
        "all-paths" => match &*next_tag_open(parser) {
                "until" => read_until(AU, parser),
                "globally" => unary_formula(AG, parser),
                "next" => unary_formula(AX, parser),
                "finally" => unary_formula(AF, parser),
                other => {
                    panic!("Unexpected tag {:?}", other);
                }
            },
        "exists-path" => match &*next_tag_open(parser) {
                "until" => read_until(EU, parser),
                "globally" => unary_formula(EG, parser),
                "next" => unary_formula(EX, parser),
                "finally" => unary_formula(EF, parser),
                other => {
                    panic!("Unexpected tag {:?}", other);
                }
            },
        other => {
            panic!("Unexpected tag {:?}", other);
        }
    };
    expect_tag_close(&*next_tag, parser);
    result
}

//value parsing logic
fn read_value<T: Read>(parser: &mut EventReader<T>) -> Value {
    let next_tag = next_tag_open(parser);
    let result = match &*next_tag {
        "integer-constant" => {
            if let Ok(val) = next_text(parser).parse() {
                Const(val)
            } else {
                panic!("Error parsing integer constant")
            }
        }
        "tokens-count" => Ref(next_text(parser)),
        other => {
            panic!("Unexpected tag {:?}", other);
        }
    };
    expect_tag_close(&*next_tag, parser);
    result
}

// helper functions for creating formulas

fn read_until<T, F>(constructor: F, parser: &mut EventReader<T>) -> CTLFormula
    where   T : Read, F : Fn(Box<CTLFormula>, Box<CTLFormula>) -> CTLFormula {
    let before = inside("before", parser, read_formula);
    let reach = inside("reach", parser, read_formula);
    constructor(Box::new(before), Box::new(reach))
}

fn unary_formula<T, F>(constructor: F, parser: &mut EventReader<T>) -> CTLFormula
    where T : Read, F : Fn(Box<CTLFormula>) -> CTLFormula {
    constructor(Box::new(read_formula(parser)))
}

fn binary_formula<T, F>(constructor: F, parser: &mut EventReader<T>) -> CTLFormula
    where   T : Read, F : Fn(Box<CTLFormula>, Box<CTLFormula>) -> CTLFormula {
    constructor(Box::new(read_formula(parser)), Box::new(read_formula(parser)))
}

fn atom<T, F>(constructor: F, parser: &mut EventReader<T>) -> CTLFormula
    where   T : Read, F : Fn(Value, Value) -> CTLFormula {
    constructor(read_value(parser), read_value(parser))
}
