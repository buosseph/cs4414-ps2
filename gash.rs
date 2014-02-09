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
use std::io::{stdin, File};
use extra::getopts;
use std::run::{Process, ProcessOptions};
use std::io::signal::{Listener, Interrupt};

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
            print!("[{}] {}", self.cwd.filename_display() ,self.cmd_prompt);
            io::stdio::flush();

            // Catches interrupt signal to prevent closing gash,
            // placing the listener outside of the loop broke the external command functionality
            let mut listener = Listener::new();
            listener.register(Interrupt);

            let line = stdin.read_line().unwrap();
            let cmd_line = line.trim().to_owned();
            let program = cmd_line.splitn(' ', 1).nth(0).expect("no program");
            
            match program {
                ""      =>  { continue; }
                "history"   =>  { for i in range(0, history.len()) {
                                     println! ("{}: {}", i+1, history[i]);
                                    }
                                }
                "exit"  =>  { return; }
                "help"      =>  { println!("Why you askin' me?");}
                "cd"        =>  { self.run_cd(cmd_line)}
                _       =>  { self.run_cmdline(cmd_line); }
            }

            history.push(cmd_line.to_owned());
        }
    }


    /* Works for basic cases except when folder/file names have spaces
        e.g cd ../"Spring 2014"/"CS 4414" */
    fn run_cd(&mut self, cmd_line: &str){
        let argv: ~[~str] =
            cmd_line.split(' ').filter_map(|x| if x != "" { Some(x.to_owned()) } else { None }).to_owned_vec();

        if argv.len() > 1 {
            let path_str: ~str = argv[1];
            let path = Path::new(path_str);
            if os::change_dir(&path) {
                self.cwd.push(path);
            }
        }
    }
    

    fn run_cmdline(&mut self, cmd_line: &str) {
        let mut argv: ~[~str] =
            cmd_line.split(' ').filter_map(|x| if x != "" { Some(x.to_owned()) } else { None }).to_owned_vec();
    
        if argv.len() > 0 {
        let program: ~str = argv.remove(0);
                self.run_cmd(program, argv);
        }


        // for i in range(0, argv.len()){
        //     if (argv[i] == ~">"){
        //         let mut cmd1 = argv.slice(0, i);
        //         let mut cmd2 = argv.slice(i+1, argv.len());

        //         // let process1 = Process::new(cmd1[0], cmd1.slice(1,cmd1.len()), ProcessOptions {
        //         //     env: None,
        //         //     dir: None,
        //         //     in_fd: None,
        //         //     out_fd: None,
        //         //     err_fd: None
        //         // });
        //         let process2 = Process::new(cmd2[0], cmd2.slice(1,cmd2.len()), ProcessOptions {
        //             env: None,
        //             dir: None,
        //             in_fd: None,
        //             out_fd: None,
        //             err_fd: None
        //         });

        //         let p_opt = run::process_output(cmd1[0], cmd1.slice(1,cmd1.len()));
        //         let p_out = p_opt.unwrap();

        //         // Breaks external commands as well...
        //         spawn(proc(){
        //             match process2 {
        //                 Some(mut process2)      => {
        //                     let writer = process2.input();
        //                     writer.write(p_out.output);
        //                 }
        //                 None                    => {}
        //             }
        //         });

        //     }
        // }


    }
    

    fn run_cmd(&mut self, program: &str, argv: &[~str]) {
        if self.cmd_exists(program) {

            // Redirects are processed from left to right
            if argv.contains(&~">") {
                println!("Found right redirect");
                for i in range(0, argv.len()) {
                    if argv[i] == ~">" {
                        self.redirect_right(program, argv.slice(0, i), argv[i+1]);
                    }
                }
            }

            else if argv.contains(&~"<") {
                println!("Found left redirect");
                for i in range(0, argv.len()) {
                    if argv[i] == ~"<" {
                        self.redirect_left(program, argv.slice(0,i), argv[i+1]);
                    }
                }
            }

            else if argv.contains(&~"|") {
                println!("Found pipe");
                for i in range(0, argv.len()) {
                    if argv[i] == ~"|" {
                        self.pipe(program, argv.slice(0, i), argv[i+1], argv.slice(i+2, argv.len()));
                    }
                }
            }

            else if argv.contains(&~"&") {
                // for i in range(0, argv.len()){
                //     if argv[i] = ~"&" {
                //         argv.remove(i);
                //     }
                // }
                let process = Process::new(program, argv, ProcessOptions {
                    env: None,
                    dir: None,
                    in_fd: Some(0),
                    out_fd: Some(1),
                    err_fd: Some(2)
                });
                spawn(proc () {
                    match process {
                        Some(mut process) => {
                            println!("\tRunning process in the background" );
                            process.finish();
                        }
                        None       => { }
                    }
                });
            } else {
                run::process_status(program, argv);
            }
        } 
        else {
            println!("{:s}: command not found", program);
        }
    }
    

    fn cmd_exists(&mut self, cmd_path: &str) -> bool {
        let ret = run::process_output("which", [cmd_path.to_owned()]);
        return ret.expect("exit code error.").status.success();
    }


    // Functional, but not correctly
    fn redirect_right(&mut self, program: &str, argv: &[~str], write_to: &str){
        let path = &Path::new(write_to.clone());
        let mut output_file = File::create(path);
        println!("Created output file {}", write_to);

        // Works now, but requires interrupt signal to 
        // be sent twice to return to gash
        match Process::new(program, argv, ProcessOptions::new()) {
            Some(mut p) => {
                {
                    let process = &mut p;
                    let rdr = process.output();
                    let out = rdr.read_to_str();

                    // Write to output file
                    output_file.write_str(out);
                }
                p.close_input();
                p.close_outputs();
                p.finish();
            },
            None => {println!("Redirect error!");}
        }
    }


    // Tested with cat command
    // Works, but prints that there's not directory/file for cat and <
    // Try to fix that later 
    fn redirect_left(&mut self, program: &str, argv: &[~str], read_from: &str){
        let path = &Path::new(read_from.clone());

        // println!("Prog1: {}", program );
        // for i in range(0, argv.len()){
        //     print!("\t{}", argv[i]);
        // }

        if !path.is_file() {
            println!("There doesn't seem to be any file called {}", read_from);
        }
        else {
            let input = File::open(path).read_to_end();
            println!("Starting process");
            match Process::new(program, argv, ProcessOptions::new()) {
                Some(mut p) => {
                    {
                        let process = &mut p;
                        let wtr = process.input();

                        // Read input file
                        wtr.write(input);
                    }
                    p.close_input();
                    p.close_outputs();
                    p.finish();
                },
                None => {println!("Redirect error!");}
            }
        }
    }


    fn pipe(&mut self, prog1: &str, prog1_args: &[~str], prog2: &str, prog2_args: &[~str]){
        let mut output: ~str = ~"";

        println!("Prog1: {}", prog1 );
        for i in range(0, prog1_args.len()){
            print!("\t{}", prog1_args[i]);
        }
        println!("\n");

        println!("Prog2: {}", prog2 );
        for i in range(0, prog2_args.len()){
            print!("\t{}", prog2_args[i]);
        }
        println!("\n");

        match Process::new(prog2, prog2_args, ProcessOptions::new()){
            Some(mut p2)    => {
                // match Process::new(prog1, prog1_args, ProcessOptions::new()){
                //     Some(mut p1)    => {
                //         {
                //             let process1 = &mut p1;
                //             output = process1.output().read_to_str();
                //         }
                //         p1.close_input();
                //         p1.close_outputs();
                //         p1.finish();
                //     },
                //     None            => { println!("Something went wrong with {}", prog1);}
                // }

                {
                    // let process2 = &mut p2;
                    // let writer = process2.input();
                    // writer.write_str(output);
                }
                p2.close_input();
                p2.close_outputs();
                p2.finish();
            },
            None            => { println!("Something went wrong with {}", prog2);}
        }

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
