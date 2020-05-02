extern crate structopt;
use crate::structopt::StructOpt;

mod sexpr;
use sexpr::*;

use std::io;
use std::error::Error;

#[derive(StructOpt)]
pub struct CmdArgs {
    // activate silent mode
    #[structopt(short, long)]
    silent: bool,
    // expect multiple lines of user input from stdin
    #[structopt(short, long)]
    multiline: bool,
    // activate debug mode
    #[structopt(short, long)]
    debug: bool,
    // the nesting depth of a s-expression to display on a single line
    #[structopt(short, long, default_value = "1")]
    complexity_threshold: u32,
    // squish the arguments of quantifiers onto the same line
    #[structopt(short = "q", long)]
    short_quantifiers: bool,
}
impl CmdArgs {
    pub fn noisy(&self) -> bool { !self.silent }
    pub fn multiline(&self) -> bool { self.multiline }
    pub fn debug(&self) -> bool { self.debug }
    pub fn complexity_threshold(&self) -> u32 { self.complexity_threshold }
    pub fn short_quantifiers(&self) -> bool { self.short_quantifiers }
}

fn read_input(args: &CmdArgs) -> Result<String, io::Error> {
    if args.noisy() {
        println!("Input s-expression to format: ");
    }
    let mut input = String::new();
    let mut buf = String::new();
    if args.multiline() {
        loop {
            io::stdin().read_line(&mut buf)?;
            if buf.trim().is_empty() { break; }
            input.push_str(&buf);
            buf.clear();
        }
    } else {
        io::stdin().read_line(&mut input)?;
    }
    Ok(input)
}

fn main() -> Result<(), Box<dyn Error>> {
    let cmd_args = CmdArgs::from_args();
    let input = read_input(&cmd_args)?;
    let sexpr = Sexpr::parse(&input)?;
    if cmd_args.debug() {
        println!("final result: {:#?}", sexpr);
    }
    sexpr.pretty_print(&cmd_args)?;
    Ok(())
}
