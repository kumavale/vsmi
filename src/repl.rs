use std::io::{Write};

use super::token::Tokens;
use super::lexer::tokenize;
use super::parser::parse;

pub fn run() {
    let mut tokens: Tokens = Tokens::new();
    //tokens.push(TokenKind::EOL, 0);

    let mut number_of_lines: u32 = 1;

    let mut registers: [i32; 32] = [0; 32];
    let mut hi: u32 = 0;
    let mut lo: u32 = 0;
    let mut data:  Vec<u8> = Vec::new();
    let mut stack: Vec<u8> = vec![0];

    println!("Welcome mipsi REPL!");
    println!("Type `exit` or ^C to exit");

    loop {
        print!("> ");
        std::io::stdout().flush().unwrap();

        let input = {
           let mut s = String::new();
           std::io::stdin().read_line(&mut s).unwrap();
           s.trim_end().to_owned()
        };

        match &*input {
            "exit" => break,
            "" => continue,
            _ => (),
        }

        tokenize(number_of_lines, &input, &mut tokens);
        number_of_lines += 1;

        if 0 < tokens.len() {
            parse(&mut tokens, &mut registers, &mut hi, &mut lo,
                &mut data, &mut stack);
        }

        println!();
    }
}

