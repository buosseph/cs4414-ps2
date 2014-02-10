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
    }
    

    fn run_cmd(&mut self, program: &str, argv: &[~str]) {
        if self.cmd_exists(program) {
            if argv.contains(&~"&") {
                let background = true;
                let argv_bkgd = argv.slice(0, argv.len()-1);
                if argv_bkgd.contains(&~">"){
                    for i in range(0, argv_bkgd.len()){
                        if argv_bkgd[i] == ~">" {
                            self.redirect_right(program, argv_bkgd.slice(0, i), argv_bkgd[i+1], background);
                            break;
                        }
                    }
                }
                else if argv_bkgd.contains(&~"<"){
                    for i in range(0, argv_bkgd.len()){
                        if argv_bkgd[i] == ~"<" {
                            self.redirect_left(program, argv_bkgd.slice(0, i), argv_bkgd[i+1], background);
                            break;
                        }
                    }
                }
                else if argv_bkgd.contains(&~"|"){
                    let mut args = argv.to_owned();
                    args.insert(0, program.to_owned());
                    self.check_for_pipes(args);
                }
                else {
                    let process = Process::new(program, argv_bkgd, ProcessOptions{
                        env: None,
                        dir: None,
                        in_fd: Some(0),
                        out_fd: Some(1),
                        err_fd: Some(2)
                    });
                    spawn(proc () {
                        match process {
                            Some(mut process) => {
                                process.finish();
                            },
                            None       => { }
                        }
                    });
                }

            } else {





                let background = false;
                if argv.contains(&~">"){
                    for i in range(0, argv.len()){
                        if argv[i] == ~">" {
                            let mut file = argv[i+1].to_owned();
                            let last_char = file.pop_char();
                            if last_char == ';' {
                                self.redirect_right(program, argv.slice(0, i), file, background);
                                let program = argv[i+2].to_owned();
                                let args = argv.slice(i+3, argv.len());
                                self.run_cmd(program, args);
                                break;
                            }
                            argv[i+1].to_owned().push_char(last_char);
                            self.redirect_right(program, argv.slice(0, i), argv[i+1], background);
                            break;
                        }
                    }
                }
                else if argv.contains(&~"<"){
                    for i in range(0, argv.len()){
                        if argv[i] == ~"<" {
                            self.redirect_left(program, argv.slice(0, i), argv[i+1], background);
                            break;
                        }
                    }
                }
                else if argv.contains(&~"|"){
                    let mut args = argv.to_owned();
                    args.insert(0, program.to_owned());
                    self.check_for_pipes(args);
                }
                else {
                    run::process_status(program, argv);
                }
                
            }

        } else {
            println!("{:s}: command not found", program);
        }
    }
    

    fn cmd_exists(&mut self, cmd_path: &str) -> bool {
        let ret = run::process_output("which", [cmd_path.to_owned()]);
        return ret.expect("exit code error.").status.success();
    }


    fn check_for_pipes(&mut self, cmd_argv: &[~str]){
        for i in range(0, cmd_argv.len()){
            if cmd_argv[i] == ~"|" {
                self.pipe(cmd_argv[0], cmd_argv.slice(1, i), cmd_argv[i+1], cmd_argv.slice(i+2, cmd_argv.len()));
            }
        }      
    }
    
    // fn check_for_pipes(&mut self, cmd_argv: &[~str]){

    //     // let mut channels: ~[std::os::Pipe] = ~[];
    //     // for _ in range(0, progs.len()) {
    //     //   channels.push(std::os::pipe());
    //     // }
    //     // let mut prog: [&[~str]];
    //     // let mut last = 0;
    //     // for i in range(0, cmd_argv.len()){
    //     //     if cmd_argv[i] == ~"|" {
    //     //         prog.pus(cmd_argv.slice(last, i));
    //     //         last = i+1;
    //     //         //self.pipe(cmd_argv[0], cmd_argv.slice(1, i), cmd_argv[i+1], cmd_argv.slice(i+2, cmd_argv.len()));
    //     //     }
    //     // }

    //     // let mut out;
    //     // let mut into;
    //     // for i in range(0, prog.len()) {
    //     //     if i == 0 {
    //     //         out = 
    //     //     }

    //     //     if i == prog.len()-1 {
    //     //         program = prog[i].remove(0);
    //     //         args = prog[i];
    //     //         let fp = Process::new(program, args, ProcessOptions{
    //     //             env: None,
    //     //             dir: None,
    //     //             in_fd: Some(prog.len()-1),
    //     //             out_fd: Some(prog.len()),
    //     //             err_fd: None
    //     //         });
    //     //         fp.unwrap().finish();
    //     //     }
    //     //     else {
    //     //         spwan(proc() {
    //     //             program = prog[i].remove(0);
    //     //             args = prog[i];
    //     //             let fp = Process::new(program, args, ProcessOptions{
    //     //                 env: None,
    //     //                 dir: None,
    //     //                 in_fd: Some(0),
    //     //                 out_fd: Some(1),
    //     //                 err_fd: Some(2)
    //     //             });
    //     //             fp.unwrap().finish();
    //     //         });
    //     //     }
    //     // }


    // }

    // Functional, but not correctly
    fn redirect_right(&mut self, program: &str, argv: &[~str], write_to_file: &str, background: bool){


        if background {
            let path = &Path::new(write_to_file.clone());
            let mut output_file = File::create(path).unwrap();
            let process = Process::new(program, argv, ProcessOptions::new());

            let (port, chan): (Port<~str>, Chan<~str>) = Chan::new();
            spawn(proc() {
                match process {
                    Some(mut p)     => {
                        {
                            let process = &mut p;
                            let reader = process.output();
                            let output = reader.read_to_str();  //~str
                            

                            // Write to output file
                            chan.send(output);
                        }
                        p.close_input();
                        p.close_outputs();
                        p.finish();
                    },
                    None            => {println!("Redirect error!");}
                }
            });
            let output = port.recv();
            output_file.write_str(output);
        } else {
            let path = &Path::new(write_to_file.clone());
            let mut output_file = File::create(path).unwrap();   

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
    }

    fn redirect_left(&mut self, program: &str, argv: &[~str], read_from_file: &str, background: bool){
        if background {

        }
        else {
            let path = &Path::new(read_from_file.clone());
            if !path.is_file() {
                println!("There doesn't seem to be any file called {}", read_from_file);
            }
            else {
                let mut input_file = File::open(path);
                let input = input_file.read_to_end();
                match Process::new(program, argv, ProcessOptions{
                    env: None,
                    dir: None,
                    in_fd: None,
                    out_fd: Some(1),
                    err_fd: Some(2)
                }) {
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
            
    }


    fn pipe(&mut self, prog1: &str, prog1_args: &[~str], prog2: &str, prog2_args: &[~str]){
        //let mut output: ~str = ~"";

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

     let (port, chan): (Port<~str>, Chan<~str>) = Chan::new();

    //spawn(proc() {
        match Process::new(prog2, prog2_args, ProcessOptions{
            env: None,
            dir: None,
            in_fd: Some(0),
            out_fd: None,
            err_fd: Some(2)
        }){
            Some(mut p2)    => {
        let process = Process::new(prog1, prog1_args, ProcessOptions{
                        env: None,
                    dir: None,
                    in_fd: None,
                        out_fd: Some(1),
                        err_fd: Some(2)
                    });

        spawn(proc() {
                match process{
                    Some(mut p1)    => {
                        {
                            let process1 = &mut p1;
                            let output = process1.output().read_to_str();
                            //let process = &mut p;
                            //let reader = process.output();
                            //let output = reader.read_to_str();  //~str
                            
                            // Write to output file
                            chan.send(output);
                        }
                        p1.close_input();
                        p1.close_outputs();
                        p1.finish();
                    },
                    None            => { ; }  //println!("Something went wrong with {}", prog1);}
                }
        });

                let output = port.recv();

                {
                    let process2 = &mut p2;
                    let writer = process2.input();
                    writer.write_str(output);
                }
                p2.close_input();
                p2.close_outputs();
                p2.finish();
            },
            None            => { println!("Something went wrong with {}", prog2);}
        }
    //});

    }

    // fn pipe(&mut self, prog1: &str, prog1_args: &[~str], prog2: &str, prog2_args: &[~str]){


    //     match Process::new(prog2, prog2_args, ProcessOptions{
    //         env: None,
    //         dir: None,
    //         in_fd: None,
    //         out_fd: Some(1),
    //         err_fd: Some(2)
    //     }){
    //         Some(mut p2)    => {
    //             let process1 = Process::new(prog1, prog1_args, ProcessOptions{
    //                 env: None,
    //                 dir: None,
    //                 in_fd: Some(0),
    //                 out_fd: None,
    //                 err_fd: Some(2)
    //             });
    //             let (port, chan): (Port<~str>, Chan<~str>) = Chan::new();
    //             spawn(proc() {
    //                 match process1 {
    //                     Some(mut p1)    => {
    //                         {
    //                             let process1 = &mut p1;
    //                             let output = process1.output().read_to_str();
    //                             chan.send(output);
    //                         }
    //                         p1.close_input();
    //                         p1.close_outputs();
    //                         p1.finish();
    //                     },
    //                     None            => { println!("Pipe Error!");}
    //                 }
    //             });

    //             {
    //                 let process2 = &mut p2;
    //                 let writer = process2.input();
    //                 let input = port.recv();
    //                 writer.write_str(input);
    //             }
    //             p2.close_input();
    //             p2.close_outputs();
    //             p2.finish();
    //         },
    //         None            => { println!("Something went wrong with {}", prog2);}
    //     }
    // }

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
