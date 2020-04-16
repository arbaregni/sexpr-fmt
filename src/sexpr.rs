use std::fmt::Formatter;
use std::{fmt};
use crate::sexpr::SexprKind::{Compound, Atom};
use std::iter::repeat;

#[derive(Debug)]
pub struct Sexpr<'a> {
    kind: SexprKind<'a>,
    complexity: u32,
}
#[derive(Debug)]
enum SexprKind<'a> {
    Atom(&'a str),
    Compound(Box<Sexpr<'a>>, Vec<Sexpr<'a>>),
}
pub type ParseError = &'static str;

impl Sexpr<'_> {
    /// Attempt to create an s expression from the given input
    pub fn parse(input: &str) -> Result<Sexpr<'_>, ParseError> {
        let (sexpr, tail) = Sexpr::parse_helper(input)?;
        if !tail.is_empty() {
            return Err("unclosed sexpr");
        }
        Ok(sexpr)
    }
    fn parse_helper(input: &str) -> Result<(Sexpr<'_>, &'_ str), ParseError> {
        let input = input.trim();
        if input.is_empty() {
            return Ok((Sexpr::blank(), ""))
        }
        let (head, remaining) = input.split_at(1);
        let (kind, complexity, remaining) = if head == "(" {
            // a compound expression
            // get the first expr, which is at the same depth as us
            let (first, mut remaining) = Sexpr::parse_helper(remaining)?;
            // get the remaining exprs, which are one level below
            let mut args = Vec::new(); // collect args here
            let mut complexity = first.complexity; // find maximum complexity
            while !remaining.is_empty() {
       //         println!("in loop, remaining = \"{}\"", remaining);
                let (sexpr, tail) = Sexpr::parse_helper(remaining)?;
                if sexpr.is_blank() { break; }
                complexity = std::cmp::max(complexity, sexpr.complexity);
                remaining = tail;
                args.push(sexpr);
            }
            // println!("finished reading args, remaining = `{}`", remaining);
            remaining = remaining.trim();
            if remaining.is_empty() {
                return Err("malformed sexpr: expected `)`, found EOI");
            }
            let (end_paren, remaining) = remaining.split_at(1);
            if end_paren != ")" {
                return Err("malformed sexpr: expected `)`, found something else");
            }
            // println!("finished compound, sloughed off `{}`, remaining = `{}`", end_paren, remaining);
            (Compound(Box::new(first), args), complexity + 1, remaining)
        } else if head.is_empty() {
            return Err("unexpected end of input");
        } else {
            // parse an atomic expression by going through the input
            // until we hit a whitespace
            let mut idx= 0;
            while idx < input.len() && is_ident(&input[idx..idx+1]) {
                idx += 1;
            }
            let (item, remaining) = input.split_at(idx);
            let complexity = 0; // the complexity of an atom is zero
            (Atom(item), complexity, remaining)
        };
        let sexpr = Sexpr { kind, complexity };
        // println!("parsed: {:?}, remaining: \"{}\"", sexpr, remaining);
        Ok((sexpr, remaining))
    }
    pub fn blank() -> Sexpr<'static> {
        let kind = Atom("");
        let complexity = 0;
        Sexpr{ kind, complexity }
    }
    pub fn is_blank(&self) -> bool {
        if let Atom(text) = self.kind {
            text.is_empty()
        } else {
            false
        }
    }
    /// Helper function for impl of `fmt::Display`
    /// writes the display to `f`, each line having at least
    /// `depth` spaces before it
    fn format(&self, f: &mut Formatter<'_>, args: FormatArgs) -> fmt::Result {
        let tab = args.tab();
        match self.kind {
            Atom(text) => write!(f, "{}{}", tab, text)?,
            Compound(ref head, ref subformulas) => {
                let (new_depth, sep) =
                    if self.complexity <= args.complexity_threshold {
                        // inlined: do print any tabs and separate with ' '
                        (0, ' ')
                    } else {
                        // multiline: increment the depth, and separate with '\n'
                        (args.depth + 4, '\n')
                    };
                write!(f, "{tab}({head}", tab = tab, head = head)?;
                for sexpr in subformulas {
                    write!(f, "{}", sep)?;
                    sexpr.format(f, args.with_depth(new_depth))?;
                }
                // we put the closing `)` on a new line only if we're in multiline mode
                if self.complexity > args.complexity_threshold {
                    write!(f, "\n{}", tab)?;
                }
                write!(f, ")")?; // finish with the closing paren
            }
        }
        Ok(())
    }
}
/// Contains all of the arguments needed in the calculations
/// of `Sexpr::format`
struct FormatArgs {
    depth: usize, // the current nesting depth of the printing
    complexity_threshold: u32, // the maximum complexity to print a sexpr on a single line
}
impl FormatArgs {
    fn from(depth: usize, complexity_threshold: u32) -> FormatArgs {
        FormatArgs { depth, complexity_threshold }
    }
    fn with_depth(&self, new_depth: usize) -> FormatArgs {
        FormatArgs { depth: new_depth, complexity_threshold: self.complexity_threshold }
    }
    fn tab(&self) -> String {
        repeat(' ').take(self.depth).collect()
    }
}

fn is_ident(s: &str) -> bool {
    s.chars().all(|ch| ch != '(' && ch != ')' && !ch.is_whitespace())
}


impl <'a> fmt::Display for Sexpr<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.format(f, FormatArgs::from(0, 1))
    }
}

