use crate::{expr::Expr, operator::*};

pub type Num = i32;

pub struct Input {
    pub name: &'static str,
    pub vec: &'static [Num],
    pub min_uses: u8,
    pub max_uses: u8,
}

pub const INPUTS: &[Input] = &[Input {
    name: "n",
    vec: &['E' as i32, 'W' as i32, 'N' as i32, 'S' as i32],
    min_uses: 1,
    max_uses: 255,
}];

pub struct Matcher {}

impl Matcher {
    pub fn new() -> Self {
        Self {}
    }

    pub fn match_one(&mut self, index: usize, output: Num) -> bool {
        output == GOAL[index]
    }

    // Will be called after match_one returns true for all outputs
    pub fn match_final(self, _el: Option<&Expr>, _er: &Expr, _op: OpIndex) -> bool {
        true
    }

    pub fn match_all(expr: &Expr) -> bool {
        let mut matcher = Self::new();
        expr.output
            .iter()
            .enumerate()
            .all(|(i, &o)| matcher.match_one(i, o))
            && matcher.match_final(
                expr.left.map(|e| unsafe { e.as_ref() }),
                unsafe { expr.right.unwrap().as_ref() },
                expr.op_idx,
            )
    }
}

pub const GOAL: &[Num] = &[1, -1, 0, 0];

pub const MAX_LENGTH: usize = 14;
pub const MAX_CACHE_LENGTH: usize = 10;
pub const MIN_MULTITHREAD_LENGTH: usize = MAX_CACHE_LENGTH + 1;
pub const LITERALS: &[Num] = &[
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
    27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40,
];
/// If not 0, include all numbers in 1..=MAX_LITERAL in addition to LITERALS.
pub const MAX_LITERAL: Num = 0;

#[rustfmt::skip]
pub const BINARY_OPERATORS: &[BinaryOp] = &[
    OP_OR_SYMBOL, // a || b
    OP_OR_LOGICAL, // +!!(a || b), ensures output is 0 or 1
    OP_AND_SYMBOL, // a && b
    OP_AND_LOGICAL, // +!!(a && b), ensures output is 0 or 1
    OP_LT, // a < b
    OP_LE, // a <= b
    OP_GT, // a > b
    OP_GE, // a >= b
    OP_EQ, // a == b
    OP_NE, // a != b
    OP_BIT_OR, // a | b
    OP_BIT_XOR, // a ^ b
    OP_BIT_AND, // a & b
    OP_BIT_SHL_WRAP, // a << b
    OP_BIT_SHR_WRAP, // a >> b
    OP_ADD, // a + b
    OP_SUB, // a - b
    OP_MUL, // a * b
    OP_MOD_TRUNC, // a % b, remainder
    OP_DIV_TRUNC, // a / b | 0, truncated (e.g. -21 / 4 = -5.25 => -5, -23 / 4 = -5.75 => -5)
    OP_EXP, // a ** b

    // Not applicable to JavaScript
    // OP_OR, // aorb
    // OP_SPACE_OR, // a orb
    // OP_OR_SPACE, // aor b
    // OP_SPACE_OR_SPACE, // a or b
    // OP_AND, // aandb
    // OP_SPACE_AND, // a andb
    // OP_AND_SPACE, // aand b
    // OP_SPACE_AND_SPACE, // a and b
    // OP_BIT_SHL, // a << b but with checks to ensure b is between [0, bits]
    // OP_BIT_SHR, // a >> b but with checks to ensure b is between [0, bits)
    // OP_MOD_FLOOR, // ((a % b) + b) % b, modulus
    // OP_DIV_FLOOR, // a / b, rounded to negative infinity (e.g. -21 / 4 = -5.25 => -6, -23 / 4 = -5.75 => -6)
    // OP_GCD, // greatest common divisor(a, b)
];

#[rustfmt::skip]
pub const UNARY_OPERATORS: &[UnaryOp] = &[
    OP_BIT_NEG, // ~a
    OP_NEG, // -a
    OP_NOT, // !a
];

/// Match leaf expressions 1 output at a time to avoid unnecessary precalculations
pub const MATCH_1BY1: bool = true;

/// If set, e.g. to `Some(-159236)`, this arbitrary number is chosen to represent errors.
/// That is, pysearch will pretend 1/0 = -159236, and -159236 * 2 = -159236, and so on.
pub const ERROR_VALUE: Option<Num> = None;
