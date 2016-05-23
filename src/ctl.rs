use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum Value {
    Const(i32),
    Ref(String),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Value::Const(ref v) => write!(f, "{}", v),
            &Value::Ref(ref v) => write!(f, "{}", v),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum CTLFormula {
    //True,
    //False,
    LT(Value, Value),
    LE(Value, Value),
    GT(Value, Value),
    GE(Value, Value),
    Not(Box<CTLFormula>),
    EX(Box<CTLFormula>),
    AX(Box<CTLFormula>),
    EF(Box<CTLFormula>),
    AF(Box<CTLFormula>),
    EG(Box<CTLFormula>),
    AG(Box<CTLFormula>),
    And(Box<CTLFormula>, Box<CTLFormula>),
    Or(Box<CTLFormula>, Box<CTLFormula>),
    EU(Box<CTLFormula>, Box<CTLFormula>),
    AU(Box<CTLFormula>, Box<CTLFormula>),
}

impl fmt::Display for CTLFormula {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            //&CTLFormula::False => write!(f, "false"),
            //&CTLFormula::True => write!(f, "true"),
            &CTLFormula::LT(ref left, ref right) => write!(f, "({} < {})", left, right),
            &CTLFormula::GT(ref left, ref right) => write!(f, "({} > {})", left, right),
            &CTLFormula::LE(ref left, ref right) => write!(f, "({} <= {})", left, right),
            &CTLFormula::GE(ref left, ref right) => write!(f, "({} >= {})", left, right),
            &CTLFormula::Not(ref inner) => write!(f, "!({})", inner),
            &CTLFormula::EX(ref inner) => write!(f, "EX({})", inner),
            &CTLFormula::AX(ref inner) => write!(f, "AX({})", inner),
            &CTLFormula::EF(ref inner) => write!(f, "EF({})", inner),
            &CTLFormula::AF(ref inner) => write!(f, "AF({})", inner),
            &CTLFormula::EG(ref inner) => write!(f, "EG({})", inner),
            &CTLFormula::AG(ref inner) => write!(f, "AG({})", inner),
            &CTLFormula::And(ref left, ref right) => write!(f, "({} && {})", left, right),
            &CTLFormula::Or(ref left, ref right) => write!(f, "({} || {})", left, right),
            &CTLFormula::EU(ref left, ref right) => write!(f, "(E {} U {})", left, right),
            &CTLFormula::AU(ref left, ref right) => write!(f, "(A {} U {})", left, right),
        }
    }
}
