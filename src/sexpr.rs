use std::{fmt, io};
use crate::sexpr::SexprKind::{Compound, Atom};
use std::iter::repeat;
use std::fmt::Formatter;
use crate::CmdArgs;

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
    pub fn is_named(&self, text: &str) -> bool {
        match self.kind {
            Atom(name) if name == text => true,
            _ => false
        }
    }
    pub fn is_blank(&self) -> bool {
        if let Atom(text) = self.kind {
            text.is_empty()
        } else {
            false
        }
    }
    pub fn pretty_print(&self, cmd_args: &CmdArgs) -> fmt::Result {
        let fmt_args = FormatArgs::from(cmd_args);
        let mut f = ToWriteFmt(io::stdout());
        self.write_helper(&mut f, fmt_args)
    }
    /// Writes this sexpr to `f`, using the specified FormatArgs
    /// prints the head of this sexpr immediately, but each subsequent newline
    /// has `depth` spaces preceding any text
    fn write_helper<W>(&self, f: &mut W, args: FormatArgs) -> fmt::Result
        where W: fmt::Write
    {
        let tab = args.tab();
        match self.kind {
            Atom(text) => write!(f, "{}", text)?,
            Compound(ref head, ref subformulas) => {
                let (new_depth, sep, line_prefix) =
                    if self.complexity <= args.complexity_threshold {
                        // inlined: do print any tabs on subsequent lines and separate with ' ', followed by no spaces
                        (0, " ", "")
                    } else {
                        // multiline: increment the depth,
                        //     and separate with a newline and a tab, (this indents them relative to us)
                        //     followed by the proper number of spaces (this preserves our indentation relative to our caller)
                        (args.depth + 4, "\n    ", tab.as_str())
                    };
                write!(f, "({}", head)?;
                let mut subformula_iter = subformulas.iter();
                if args.short_quantifiers && head.is_named("forall") || head.is_named("exists") {
                    if let Some(sexpr) = subformula_iter.next() {
                        // if the command line option is set, and our head is an atom `forall` or `exists`,
                        // then the first subformula is written on the same line
                        write!(f, " ")?;
                        sexpr.write_helper(f, args)?;
                    }
                }
                for sexpr in subformula_iter {
                    write!(f, "{}{}", sep, line_prefix)?;
                    sexpr.write_helper(f, args.with_depth(new_depth))?;
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
/// Contains all of the arguments needed in the calculations of `Sexpr::write_helper`
#[derive(Copy, Clone, Debug)]
struct FormatArgs {
    depth: usize, // the current nesting depth of the printing
    complexity_threshold: u32, // the maximum complexity to print a sexpr on a single line
    short_quantifiers: bool,
}
impl FormatArgs {
    /// create the default formatting arguments
    fn new() -> FormatArgs {
        FormatArgs {
            depth: 0,
            complexity_threshold: 1,
            short_quantifiers: false,
        }
    }
    fn from(cmd_args: &CmdArgs) -> FormatArgs {
        FormatArgs {
            depth: 0,
            complexity_threshold: cmd_args.complexity_threshold(),
            short_quantifiers: cmd_args.short_quantifiers(),
        }
    }
    fn with_depth(&self, new_depth: usize) -> FormatArgs {
        FormatArgs {
            depth: new_depth,
            complexity_threshold: self.complexity_threshold,
            short_quantifiers: self.short_quantifiers,
        }
    }
    fn tab(&self) -> String {
        repeat(' ').take(self.depth).collect()
    }
}

fn is_ident(s: &str) -> bool {
    s.chars().all(|ch| ch != '(' && ch != ')' && !ch.is_whitespace())
}

// a wrapper struct to enable things that implement io::Write to be passed to write_helper
struct ToWriteFmt<T>(T);

impl<'a, T> fmt::Write for ToWriteFmt<T> where T: io::Write
{
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.0.write_all(s.as_bytes()).map_err(|_| fmt::Error)
    }
}

impl <'a> fmt::Display for Sexpr<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let fmt_args = FormatArgs::new();
        self.write_helper(f, fmt_args)?;
        Ok(())
    }
}

