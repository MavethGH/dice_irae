use rand::{rngs::StdRng, SeedableRng};

use super::*;

fn rng() -> StdRng {
    StdRng::seed_from_u64(42)
}

#[test]
fn test_parsing() {
    parser().parse("1").unwrap();
    parser().parse("1d20").unwrap();
    parser().parse("1d(20)").unwrap();
    parser().parse("1d(2 + 2)").unwrap();
    parser().parse("(2 + 2)d(2 + 2)").unwrap();
}

#[test]
fn test_parens() {
    let a = parser().parse("1d20").unwrap();
    let b = parser().parse("1d(20)").unwrap();
    assert_eq!(a, b);
}

#[test]
fn test_eval() {
    let ast = parser().parse("2 + 2").unwrap();
    let result = eval(&ast, &mut rng()).unwrap();
    assert_eq!(result, 4);
}

#[test]
fn test_1d20() {
    let expected = roll_inner(&mut rng(), 1, 20);
    let ast = parser().parse("1d20").unwrap();
    let actual = eval(&ast, &mut rng()).unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn test_negation() {
    let minus1 = roll_str("-1").unwrap();
    assert_eq!(minus1, -1);

    let minus1d20 = roll_str_with_rng("-1d20", &mut rng()).unwrap();
    let plus1d20 = roll_str_with_rng("1d20", &mut rng()).unwrap();
    assert_eq!(-plus1d20, minus1d20);
}
