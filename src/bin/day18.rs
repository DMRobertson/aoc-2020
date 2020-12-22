use itertools::__std_iter::Peekable;
use std::str::CharIndices;

fn main() {
    let (sum1, sum2) = aoc_2020::problem_lines()
        .map(|s| {
            (
                evaluate(&s, read_expression),
                evaluate(&s, read_expression2),
            )
        })
        .fold((0, 0), |(a, b), (c, d)| ((a + c), (b + d)));
    println!("{}\n{}", sum1, sum2);
}

type TokenStream<'a> = Peekable<std::slice::Iter<'a, Token>>;
type ExpressionReader = fn(&mut TokenStream) -> Expression;

fn evaluate(input: &str, read_expression: ExpressionReader) -> usize {
    let the_tokens = tokenize(input);
    let mut tokens = the_tokens.iter().peekable();
    let exp = read_expression(&mut tokens);
    exp.evaluate()
}

#[derive(Debug, PartialEq)]
enum Token {
    OpenBracket,
    CloseBracket,
    Add,
    Multiply,
    Literal(usize),
}

fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.char_indices().peekable();
    while let Some((index, c)) = chars.next() {
        match c {
            c if c.is_ascii_digit() => {
                tokens.push(Token::Literal(parse_number(&input, &mut chars, index)))
            }
            '(' => tokens.push(Token::OpenBracket),
            ')' => tokens.push(Token::CloseBracket),
            '+' => tokens.push(Token::Add),
            '*' => tokens.push(Token::Multiply),
            _ => continue,
        }
    }

    tokens
}

fn parse_number<'a>(input: &str, chars: &mut Peekable<CharIndices>, start_index: usize) -> usize {
    loop {
        if let Some((_, c)) = chars.peek() {
            if c.is_ascii_digit() {
                chars.next();
                continue;
            }
        }
        break;
    }
    let end_index = chars.peek().map(|(i, _)| *i).unwrap_or(input.len());
    let digits = &input[start_index..end_index];
    usize::from_str_radix(digits, 10).unwrap()
}

#[derive(Debug)]
enum Expression {
    Constant(usize),
    Sum(Box<Expression>, Box<Expression>),
    Product(Box<Expression>, Box<Expression>),
}

impl Expression {
    fn evaluate(&self) -> usize {
        match self {
            Expression::Constant(u) => *u,
            Expression::Sum(e1, e2) => e1.evaluate() + e2.evaluate(),
            Expression::Product(e1, e2) => e1.evaluate() * e2.evaluate(),
        }
    }
}

fn read_expression(tokens: &mut TokenStream) -> Expression {
    let first = read_term(tokens, read_expression);
    let mut exp = Box::new(first);

    while let Some(token) = tokens.next() {
        match token {
            Token::Add => {
                exp = Box::new(Expression::Sum(
                    exp,
                    Box::new(read_term(tokens, read_expression)),
                ))
            }
            Token::Multiply => {
                exp = Box::new(Expression::Product(
                    exp,
                    Box::new(read_term(tokens, read_expression)),
                ))
            }
            Token::CloseBracket => break,
            _ => panic!(),
        }
    }
    *exp
}

fn read_expression2(tokens: &mut TokenStream) -> Expression {
    let first = read_summands(tokens, read_expression2);
    let mut exp = Box::new(first);

    while let Some(token) = tokens.next() {
        match token {
            Token::Add => {
                exp = Box::new(Expression::Sum(
                    exp,
                    Box::new(read_summands(tokens, read_expression2)),
                ))
            }
            Token::Multiply => {
                exp = Box::new(Expression::Product(
                    exp,
                    Box::new(read_summands(tokens, read_expression2)),
                ))
            }
            Token::CloseBracket => break,
            _ => panic!(),
        }
    }
    *exp
}

fn read_term(tokens: &mut TokenStream, read_expression: ExpressionReader) -> Expression {
    let term = match tokens.next().unwrap() {
        Token::OpenBracket => {
            let exp = read_expression(tokens);
            exp
        }
        Token::CloseBracket => unreachable!(),
        Token::Add => unreachable!(),
        Token::Multiply => unreachable!(),
        Token::Literal(u) => Expression::Constant(*u),
    };
    term
}

fn read_summands(tokens: &mut TokenStream, read_expression: ExpressionReader) -> Expression {
    let mut read_summand = |tokens: &mut TokenStream| {
        Box::new(match tokens.next().unwrap() {
            Token::OpenBracket => read_expression(tokens),
            Token::CloseBracket => unreachable!(),
            Token::Add => unreachable!(),
            Token::Multiply => unreachable!(),
            Token::Literal(u) => Expression::Constant(*u),
        })
    };
    let mut sum = read_summand(tokens);
    while let Some(Token::Add) = tokens.peek() {
        tokens.next();
        let next_summand = read_summand(tokens);
        sum = Box::new(Expression::Sum(sum, next_summand));
    }
    *sum
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn example_1() {
        assert_eq!(evaluate("1 + 2 * 3 + 4 * 5 + 6", read_expression), 71);
        assert_eq!(evaluate("10 * (20 + 30) * 40", read_expression), 20000);
        assert_eq!(evaluate("1 + (2 * 3) + (4 * (5 + 6))", read_expression), 51);
        assert_eq!(evaluate("2 * 3 + (4 * 5)", read_expression), 26);
        assert_eq!(
            evaluate("5 + (8 * 3 + 9 + 3 * 4 * 3)", read_expression),
            437
        );
        assert_eq!(
            evaluate("5 * 9 * (7 * 3 * 3 + 9 * 3 + (8 + 6 * 4))", read_expression),
            12240
        );
        assert_eq!(
            evaluate(
                "((2 + 4 * 9) * (6 + 9 * 8 + 6) + 6) + 2 + 4 * 2",
                read_expression
            ),
            13632,
        );
    }

    #[test]
    fn example_2() {
        assert_eq!(evaluate("1 + 2 * 3 + 4 * 5 + 6", read_expression2), 231);
        assert_eq!(evaluate("10 * (20 + 30) * 40", read_expression2), 20000);
        assert_eq!(
            evaluate("1 + (2 * 3) + (4 * (5 + 6))", read_expression2),
            51
        );
        assert_eq!(evaluate("2 * 3 + (4 * 5)", read_expression2), 46);
        assert_eq!(
            evaluate("5 + (8 * 3 + 9 + 3 * 4 * 3)", read_expression2),
            1445
        );
        assert_eq!(
            evaluate(
                "5 * 9 * (7 * 3 * 3 + 9 * 3 + (8 + 6 * 4))",
                read_expression2
            ),
            669060
        );
        assert_eq!(
            evaluate(
                "((2 + 4 * 9) * (6 + 9 * 8 + 6) + 6) + 2 + 4 * 2",
                read_expression2
            ),
            23340,
        );
    }
}
