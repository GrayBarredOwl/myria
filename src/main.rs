mod myria_settings;

use std::{
    env,
    io::{self, Write},
};

use myria::{
    debug_print,
    ast::Scope,
    gen::{self, run_myria, run_myria_with_scope},
    obj::Primitive,
};

use myria_settings::MyriaConfig;

fn main() {
    let mut config = MyriaConfig::from_args(env::args());
    config.execute_setup();
    let Some(program) = config.program_string() else {
        repl();
        return;
    };

    debug_print!("{}\n", &program);

    match run_myria(&program) {
        Ok(result) => println!("Result: {result}"),
        Err(err) => gen::print_error(err),
    }
}

fn repl() {
    let mut scope = Scope::default();
    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let resp = {
            let mut s = String::new();
            io::stdin().read_line(&mut s).unwrap();
            s = s.trim().to_string();

            while s.ends_with('\\') {
                s.pop();
                let mut s2 = String::new();
                io::stdin().read_line(&mut s2).unwrap();
                let s2 = s2.trim();
                s = format!("{s}\n{s2}");
            }
            s
        };
        let resp = resp.trim();

        if resp == "exit" || resp == "quit" {
            break;
        };
        match run_myria_with_scope(resp, &mut scope) {
            Ok(result) => {
                if result.primitive != Primitive::Null {
                    println!("{result}");
                    // println!("{result:?}");
                }
            }
            Err(err) => gen::print_error(err),
        }
    }
}
