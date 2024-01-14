use chumsky::prelude::*;
use rand::{thread_rng, Rng};

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub enum DiceError {
    ParseError(Vec<Simple<char>>),
    EvalError(String),
}

pub fn roll_str_with_rng(dice_string: &str, rng: &mut impl Rng) -> Result<i32, DiceError> {
    let ast = parser()
        .parse(dice_string)
        .map_err(|e| DiceError::ParseError(e))?;
    eval(&ast, rng).map_err(|e| DiceError::EvalError(e))
}

pub fn roll_str(dice_string: &str) -> Result<i32, DiceError> {
    roll_str_with_rng(dice_string, &mut thread_rng())
}

#[derive(Debug, PartialEq, Eq)]
enum Expr {
    Constant(i32),
    DieRoll(Box<Expr>, Box<Expr>),
    Neg(Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Pow(Box<Expr>, Box<Expr>),
}

fn parser() -> impl Parser<char, Expr, Error = Simple<char>> {
    recursive(|expr| {
        let int = text::int(10)
            .map(|s: String| Expr::Constant(s.parse().unwrap()))
            .padded();

        let atom = int.or(expr.delimited_by(just('('), just(')'))).padded();

        let operator = |c| just(c).padded();

        let roll = atom
            .clone()
            .then(operator('d').then(atom.clone()))
            .map(|(lhs, (_, rhs))| Expr::DieRoll(Box::new(lhs), Box::new(rhs)))
            .or(atom);

        let unary = operator('-')
            .repeated()
            .then(roll)
            .foldr(|_op, rhs| Expr::Neg(Box::new(rhs)));

        let pow = unary
            .clone()
            .then(operator('^').then(unary.clone()))
            .map(|(lhs, (_, rhs))| Expr::Pow(Box::new(lhs), Box::new(rhs)))
            .or(unary);

        let product = pow
            .clone()
            .then(
                operator('*')
                    .to(Expr::Mul as fn(_, _) -> _)
                    .or(operator('/').to(Expr::Div as fn(_, _) -> _))
                    .then(pow)
                    .repeated(),
            )
            .foldl(|lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)));
        let sum = product
            .clone()
            .then(
                operator('+')
                    .to(Expr::Add as fn(_, _) -> _)
                    .or(operator('-').to(Expr::Sub as fn(_, _) -> _))
                    .then(product)
                    .repeated(),
            )
            .foldl(|lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)));
        sum
    })
    .then_ignore(end())
}

fn eval(expr: &Expr, rng: &mut impl Rng) -> Result<i32, String> {
    match expr {
        Expr::Constant(x) => Ok(*x),
        Expr::DieRoll(count, sides) => {
            let count = eval(count, rng)?;
            let sides = eval(sides, rng)?;
            Ok(roll_inner(rng, count, sides))
        }
        Expr::Neg(a) => Ok(-eval(a, rng)?),
        Expr::Add(a, b) => Ok(eval(a, rng)? + eval(b, rng)?),
        Expr::Sub(a, b) => Ok(eval(a, rng)? - eval(b, rng)?),
        Expr::Mul(a, b) => Ok(eval(a, rng)? * eval(b, rng)?),
        Expr::Div(a, b) => Ok(eval(a, rng)? / eval(b, rng)?),
        Expr::Pow(a, b) => Ok(eval(a, rng)?.pow(
            eval(b, rng)?
                .try_into()
                .map_err(|_| "negative exponents are not allowed".to_string())?,
        )),
    }
}

fn roll_inner(rng: &mut impl Rng, count: i32, sides: i32) -> i32 {
    (0..count)
        .into_iter()
        .map(|_| rng.gen_range(1..=sides))
        .sum()
}
