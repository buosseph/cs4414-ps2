//
// gash.rs
//
// Starting code for PS2
// Running on Rust 0.9
//
// University of Virginia - cs4414 Spring 2014
// Weilin Xu, David Evans
// Version 0.4
//

extern mod extra;

use std::{io, run, os};
use std::io::buffered::BufferedReader;
use std::io::stdin;
use extra::getopts;
use std::vec::OwnedVector;

struct Shell {
    cmd_prompt: ~str,
    cwd: Path
}

impl Shell {
    fn new(prompt_str: &str) -> Shell {
        Shell {
            cmd_prompt: prompt_str.to_owned(),
            cwd:        os::getcwd()
        }
    }
    
    fn run(&mut self) {
        let mut stdin = BufferedReader::new(stdin());
        let mut history: ~[~str] = ~[];
        
        loop {
            print!("{} {}", self.cwd.filename_display() ,self.cmd_prompt);
            io::stdio::flush();
            
            let line = stdin.read_line().unwrap();
            let cmd_line = line.trim().to_owned();
            let program = cmd_line.splitn(' ', 1).nth(0).expect("no program");

            // Internal commands here 
            match program {
                ""          =>  { continue; }
                "history"   =>  { for i in range(0, history.len()) {
                                     println! ("{}: {}", i+1, history[i]);
                                    }
                                }
                "help"      =>  { println!("Why you askin' me?");}
                "cd"        =>  { self.run_cd(cmd_line)}
                "exit"      =>  { return; }
                _           =>  { self.run_cmdline(cmd_line); } // External commands match here
            }

            history.push(cmd_line.to_owned());
        }
    }

    
    fn run_cmdline(&mut self, cmd_line: &str) {
        let mut argv: ~[~str] =
            cmd_line.split(' ').filter_map(|x| if x != "" { Some(x.to_owned()) } else { None }).to_owned_vec();
        
        if argv.len() > 0 {
            let program: ~str = argv.remove(0);
            self.run_cmd(program, argv);
        }
    }

    fn run_cmd(&mut self, program: &str, argv: &[~str]) {
        if self.cmd_exists(program) {
            println!("Running: {}", program);
            run::process_status(program, argv);

        } else {
            println!("{:s}: command not found", program);
        }
    }



    /* Works for basic cases except when folder/file names have spaces
        e.g cd ../"Spring 2014"/"CS 4414" */
    fn run_cd(&mut self, cmd_line: &str){
        let argv: ~[~str] =
            cmd_line.split(' ').filter_map(|x| if x != "" { Some(x.to_owned()) } else { None }).to_owned_vec();

        if argv.len() > 1 {
            let path_str: ~str = argv[1];
            let path_dirs: ~[&str] = path_str.split_str("/").collect();
            for i in range(0, path_dirs.len()) {
                if (path_dirs[i]=="..") {
                    self.cwd.pop();
                }
                else {
                    self.cwd.push(path_dirs[i]);
                }
            }
        }

    }
    
    fn cmd_exists(&mut self, cmd_path: &str) -> bool {
        let ret = run::process_output("which", [cmd_path.to_owned()]);
        return ret.expect("exit code error.").status.success();
    }
}

fn get_cmdline_from_args() -> Option<~str> {
    /* Begin processing program arguments and initiate the parameters. */
    let args = os::args();
    
    let opts = ~[
        getopts::optopt("c")
    ];
    
    let matches = match getopts::getopts(args.tail(), opts) {
        Ok(m) => { m }
        Err(f) => { fail!(f.to_err_msg()) }
    };
    
    if matches.opt_present("c") {
        let cmd_str = match matches.opt_str("c") {
                                                Some(cmd_str) => {cmd_str.to_owned()}, 
                                                None => {~""}
                                              };
        return Some(cmd_str);
    } else {
        return None;
    }
}

fn main() {
    let opt_cmd_line = get_cmdline_from_args();
    
    match opt_cmd_line {
        Some(cmd_line) => Shell::new("").run_cmdline(cmd_line),
        None           => Shell::new("gash > ").run()
    }
}
