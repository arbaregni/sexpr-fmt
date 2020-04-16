mod sexpr;
use sexpr::*;
use std::io;

fn read_input() -> Result<String, io::Error> {
    println!("Input s-expression to format: ");
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    Ok(buf)
}

fn main() {
    let input = match read_input() {
        Ok(line) => line,
        Err(err) => {
            eprintln!("Could not read input: {}", err);
            return;
        },
    };
    let sexpr = match Sexpr::parse(&input) {
        Ok(sexpr) => sexpr,
        Err(err) => {
            eprintln!("Could not parse: {}", err);
            return;
        },
    };
    // println!("{:?}", sexpr);
    println!("{}", sexpr);
}
