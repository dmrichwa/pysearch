pub mod gcd;

use crate::gcd::gcd;

use ndarray::{Array, Ix1};

use std::collections::HashMap;
use std::fmt::Display;

use std::sync::RwLock;

type Num = i32;
type Vec = Array<Num, Ix1>;

struct Input {
    name: &'static str,
    vec: [Num; 4],
}

const INPUTS: [Input; 1] = [Input {
    name: "x",
    vec: ['N' as Num, 'E' as Num, 'S' as Num, 'W' as Num],
}];
const GOAL: [Num; 4] = [-2, 1, 2, -1];

const MAX_LENGTH: usize = 14;
const LITERALS: [Num; 12] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];

const USE_OR: bool = false;
const USE_LT: bool = true;
const USE_LEQ: bool = true;
const USE_BIT_OR: bool = true;
const USE_BIT_XOR: bool = true;
const USE_BIT_AND: bool = true;
const USE_BIT_SHL: bool = true;
const USE_BIT_SHR: bool = true;
const USE_BIT_NEG: bool = true;
const USE_ADD: bool = true;
const USE_SUB: bool = true;
const USE_MUL: bool = true;
const USE_MOD: bool = true;
const USE_DIV1: bool = false; /* / */
const USE_DIV2: bool = true; /* // */
const USE_GCD: bool = false;
const USE_NEG: bool = true;
const USE_EXP: bool = true;

const C_STYLE_MOD: bool = false;
const REUSE_VARS: bool = true;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Operator {
    Or = 0x300,
    SpaceOr = 0x301,
    OrSpace = 0x302,
    // SpaceOrSpace = 0x303,
    Lt = 0x500,
    Leq = 0x501,
    // Gt = 0x502,
    // Geq = 0x503,
    // Eq = 0x504,
    // Neq = 0x505,
    BitOr = 0x600,
    BitXor = 0x700,
    BitAnd = 0x800,
    BitShl = 0x900,
    BitShr = 0x901,
    Add = 0xA00,
    Sub = 0xA01,
    Mul = 0xB00,
    Mod = 0xB01,
    Div1 = 0xB02,
    Div2 = 0xB03,
    Gcd = 0xB04,
    Neg = 0xC00,
    BitNeg = 0xC01,
    Exp = 0xD00,
    Parens = 0xFE00,
    Literal = 0xFF00,
}

#[derive(Clone, Copy, Debug)]
struct Expr {
    left: Option<*const Expr>,
    right: Option<*const Expr>,
    op: Operator,
    literal: Num,
    var_mask: usize,
}

impl Expr {
    fn prec(&self) -> usize {
        self.op as usize >> 8
    }
}

impl Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operator::Or => write!(f, "or"),
            Operator::SpaceOr => write!(f, " or"),
            Operator::OrSpace => write!(f, "or "),
            // Operator::SpaceOrSpace => write!(f, " or "),
            Operator::Lt => write!(f, "<"),
            Operator::Leq => write!(f, "<="),
            // Operator::Gt => write!(f, ">"),
            // Operator::Geq => write!(f, ">="),
            // Operator::Eq => write!(f, "=="),
            // Operator::Neq => write!(f, "!="),
            Operator::BitOr => write!(f, "|"),
            Operator::BitXor => write!(f, "^"),
            Operator::BitAnd => write!(f, "&"),
            Operator::BitShl => write!(f, "<<"),
            Operator::BitShr => write!(f, ">>"),
            Operator::Add => write!(f, "+"),
            Operator::Sub => write!(f, "-"),
            Operator::Mul => write!(f, "*"),
            Operator::Mod => write!(f, "%"),
            Operator::Div1 => write!(f, "/"),
            Operator::Div2 => write!(f, "//"),
            Operator::Gcd => write!(f, "∨"),
            Operator::Neg => write!(f, "-"),
            Operator::BitNeg => write!(f, "~"),
            Operator::Exp => write!(f, "**"),
            Operator::Parens => write!(f, "("),
            Operator::Literal => write!(f, ""),
        }
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(left) = self.left {
            Self::fmt(unsafe { &*left }, f)?;
        }
        Display::fmt(&self.op, f)?;
        if let Some(right) = self.right {
            Self::fmt(unsafe { &*right }, f)?;
            if self.op == Operator::Parens {
                write!(f, ")")?;
            }
        } else if self.literal < 0 {
            write!(f, "{}", INPUTS[(!self.literal) as usize].name)?;
        } else {
            write!(f, "{}", self.literal)?;
        }
        Ok(())
    }
}

// cache[length][output] = highest-prec expression of that length yielding that output
type CacheLevel = HashMap<Vec, Expr>;
type Cache = HashMap<usize, RwLock<CacheLevel>>;

// "3or" and ")or" are valid, but "nor" isn't.
unsafe fn ok_before_keyword(e: &Expr) -> bool {
    match e.right {
        None => e.literal >= 0,
        Some(right) => e.op == Operator::Parens || ok_before_keyword(&*right),
    }
}

// "or3", "orn" are invalid. Need a unary op or parens.
unsafe fn ok_after_keyword(e: &Expr) -> bool {
    match e.left {
        None => e.op != Operator::Literal,
        Some(left) => ok_after_keyword(&*left),
    }
}

fn positive_integer_length(mut k: Num) -> usize {
    let mut l = 1;
    while k >= 10 {
        k /= 10;
        l += 1;
    }
    l
}

fn divmod(left: &Vec, right: &Vec) -> Option<(Vec, Vec)> {
    if left
        .iter()
        .zip(right)
        .any(|(&x, &y)| y == 0 || (x, y) == (Num::MIN, -1))
    {
        None
    } else if C_STYLE_MOD {
        Some((left / right, left % right))
    } else {
        let modulo = (left % right + right) % right;
        let div = (left - modulo.clone()) / right;
        Some((div, modulo))
    }
}

fn cache_if_better(level: &RwLock<CacheLevel>, output: Vec, expr: Expr) {
    let all_mask = (1 << INPUTS.len()) - 1;
    if !REUSE_VARS && expr.var_mask == all_mask {
        let mut mp: HashMap<Num, Num> = HashMap::new();
        for (i, o) in output.iter().enumerate() {
            if let Some(old) = mp.insert(*o, GOAL[i]) {
                if old != GOAL[i] {
                    return;
                }
            }
        }
    }

    let old_prec = level.read().unwrap().get(&output).map(|x| x.prec());
    match old_prec {
        None => {
            level.try_write().unwrap().insert(output, expr);
        }
        Some(old_prec) => {
            if expr.prec() > old_prec {
                level.try_write().unwrap().insert(output, expr);
            }
        }
    }
}

fn variable_expr(index: usize) -> Expr {
    Expr {
        left: None,
        right: None,
        op: Operator::Literal,
        literal: !(index as Num),
        var_mask: 1 << index,
    }
}

fn literal_expr(value: Num) -> Expr {
    Expr {
        left: None,
        right: None,
        op: Operator::Literal,
        literal: value,
        var_mask: 0,
    }
}

fn bin_expr(el: *const Expr, er: *const Expr, op: Operator, var_mask: usize) -> Expr {
    Expr {
        left: Some(el),
        right: Some(er),
        op,
        literal: 0,
        var_mask,
    }
}

unsafe fn unary_expr(er: *const Expr, op: Operator) -> Expr {
    Expr {
        left: None,
        right: Some(er),
        op,
        literal: 0,
        var_mask: (*er).var_mask,
    }
}

unsafe fn find_expressions(cache: &Cache, n: usize) {
    let cn = &cache[&n];
    if n == 1 {
        for (i, input) in INPUTS.iter().enumerate() {
            let vec: Vec = Array::from_iter(input.vec);
            cn.try_write().unwrap().insert(vec, variable_expr(i));
        }
    }
    for lit in LITERALS {
        if positive_integer_length(lit) == n {
            let vec: Vec = Array::from_elem(GOAL.len(), lit);
            cn.try_write().unwrap().insert(vec, literal_expr(lit));
        }
    }

    let dim = GOAL.len();

    for k in 1..n {
        for (or, er) in &*cache[&k].read().unwrap() {
            // 1-byte operators
            if n >= k + 2 {
                for (ol, el) in &*cache[&(n - k - 1)].read().unwrap() {
                    if !REUSE_VARS && (el.var_mask & er.var_mask != 0) {
                        continue;
                    }
                    let mask = el.var_mask | er.var_mask;
                    if USE_LT && el.prec() >= 5 && er.prec() > 5 {
                        let z = Array::from_shape_fn(GOAL.len(), |i| (ol[i] < or[i]) as Num);
                        cache_if_better(cn, z, bin_expr(el, er, Operator::Lt, mask));
                    }
                    if USE_BIT_OR && el.prec() >= 6 && er.prec() > 6 {
                        cache_if_better(cn, ol | or, bin_expr(el, er, Operator::BitOr, mask));
                    }
                    if USE_BIT_XOR && el.prec() >= 7 && er.prec() > 7 {
                        cache_if_better(cn, ol ^ or, bin_expr(el, er, Operator::BitXor, mask));
                    }
                    if USE_BIT_AND && el.prec() >= 8 && er.prec() > 8 {
                        cache_if_better(cn, ol & or, bin_expr(el, er, Operator::BitAnd, mask));
                    }
                    if el.prec() >= 10 && er.prec() > 10 {
                        if USE_ADD {
                            cache_if_better(cn, ol + or, bin_expr(el, er, Operator::Add, mask));
                        }
                        if USE_SUB {
                            cache_if_better(cn, ol - or, bin_expr(el, er, Operator::Sub, mask));
                        }
                    }
                    if el.prec() >= 11 && er.prec() > 11 {
                        if USE_MUL {
                            cache_if_better(cn, ol * or, bin_expr(el, er, Operator::Mul, mask));
                        }
                        if let Some((div, modulo)) = divmod(ol, or) {
                            if USE_MOD {
                                cache_if_better(cn, modulo, bin_expr(el, er, Operator::Mod, mask));
                            }
                            if USE_DIV1 {
                                cache_if_better(cn, div, bin_expr(el, er, Operator::Div1, mask));
                            }
                        }
                        if USE_GCD {
                            let z = Array::from_shape_fn(GOAL.len(), |i| gcd(ol[i], or[i]));
                            cache_if_better(cn, z, bin_expr(el, er, Operator::Gcd, mask));
                        }
                    }
                }
            }
            // 2-byte operators
            if n >= k + 3 {
                for (ol, el) in &*cache[&(n - k - 2)].read().unwrap() {
                    if !REUSE_VARS && (el.var_mask & er.var_mask != 0) {
                        continue;
                    }
                    let mask = el.var_mask | er.var_mask;
                    if el.prec() >= 3
                        && er.prec() > 3
                        && USE_OR
                        && ok_before_keyword(el)
                        && ok_after_keyword(er)
                    {
                        let z = Array::from_shape_fn(GOAL.len(), |i| {
                            if ol[i] == 0 {
                                or[i]
                            } else {
                                ol[i]
                            }
                        });
                        cache_if_better(cn, z, bin_expr(el, er, Operator::Or, mask));
                    }
                    if USE_LEQ && el.prec() >= 5 && er.prec() > 5 {
                        let z = Array::from_shape_fn(GOAL.len(), |i| (ol[i] <= or[i]) as Num);
                        cache_if_better(cn, z, bin_expr(el, er, Operator::Leq, mask));
                    }
                    if el.prec() > 9
                        && er.prec() >= 9
                        && (0..dim).all(|i| 0 <= or[i] && or[i] <= 31)
                    {
                        if USE_BIT_SHL {
                            cache_if_better(cn, ol << or, bin_expr(el, er, Operator::BitShl, mask));
                        }
                        if USE_BIT_SHR {
                            cache_if_better(cn, ol >> or, bin_expr(el, er, Operator::BitShr, mask));
                        }
                    }
                    if el.prec() >= 11 && er.prec() > 11 {
                        if let Some((div, _)) = divmod(ol, or) {
                            if USE_DIV2 {
                                cache_if_better(cn, div, bin_expr(el, er, Operator::Div2, mask));
                            }
                        }
                    }
                    if USE_EXP
                        && el.prec() > 13
                        && er.prec() >= 13
                        && (0..dim).all(|i| 0 <= or[i] && or[i] <= 6)
                    {
                        let z = Array::from_shape_fn(GOAL.len(), |i| ol[i].pow(or[i] as u32));
                        cache_if_better(cn, z, bin_expr(el, er, Operator::Exp, mask));
                    }
                }
            }
            // 3-byte operators
            if n >= k + 4 {
                for (ol, el) in &*cache[&(n - k - 3)].read().unwrap() {
                    if !REUSE_VARS && (el.var_mask & er.var_mask != 0) {
                        continue;
                    }
                    let mask = el.var_mask | er.var_mask;
                    if el.prec() >= 3 && er.prec() > 3 {
                        let z = Array::from_shape_fn(GOAL.len(), |i| {
                            if ol[i] == 0 {
                                or[i]
                            } else {
                                ol[i]
                            }
                        });
                        if USE_OR && !ok_before_keyword(el) && ok_after_keyword(er) {
                            cache_if_better(
                                cn,
                                z.clone(),
                                bin_expr(el, er, Operator::SpaceOr, mask),
                            );
                        }
                        if USE_OR && ok_before_keyword(el) && !ok_after_keyword(er) {
                            cache_if_better(cn, z, bin_expr(el, er, Operator::OrSpace, mask));
                        }
                    }
                }
            }
        }
    }
    if n >= 3 {
        for (or, er) in &*cache[&(n - 2)].read().unwrap() {
            if er.op >= Operator::Parens {
                continue;
            }
            cn.try_write().unwrap().insert(
                or.clone(),
                Expr {
                    left: None,
                    right: Some(er),
                    op: Operator::Parens,
                    literal: 0,
                    var_mask: er.var_mask,
                },
            );
        }
    }
    if n >= 2 {
        for (or, er) in &*cache[&(n - 1)].read().unwrap() {
            if er.prec() >= 12 {
                if USE_BIT_NEG {
                    cache_if_better(cn, !or, unary_expr(er, Operator::BitNeg));
                }
                if USE_NEG {
                    cache_if_better(cn, -or, unary_expr(er, Operator::Neg));
                }
            }
        }
    }
}

fn main() {
    let mut cache: Cache = HashMap::new();
    println!("sizeof(Expr) = {}", std::mem::size_of::<Expr>());
    let mut no_results: bool = true;
    for n in 1..=MAX_LENGTH {
        cache.insert(n, Default::default());
        println!("Finding length {n}...");
        unsafe {
            find_expressions(&cache, n);
        }
        println!("Found {} expressions.", cache[&n].read().unwrap().len());
        let mut first: bool = true;
        for (out, expr) in &*cache[&n].read().unwrap() {
            if GOAL.iter().enumerate().all(|(i, x)| *x == out[i]) {
                if first {
                    println!("\n--- Length {n} ---");
                    first = false;
                    no_results = false;
                }
                println!("{}", expr);
            }
        }
    }
    if no_results {
        println!("\nNo results found.");
    }
    println!();
}
