use std::env;
use std::process::ExitCode;

static VERSION: &str = "0.1";

#[derive(Debug)]
struct Args {
    //hidden flags
    help_msg: String,
    
    //standard flags
    help: bool,
    version: bool,
}

fn main() -> ExitCode {
    let args = parse_args(env::args().skip(1).collect());
    if args.help {
        return ExitCode::from(show_help(args));
    } else if args.version {
        return ExitCode::from(show_version());
    }
    ExitCode::from(0)
}

fn parse_args(arg_strings: Vec<String>) -> Args {
    let mut a = Args {
        //invisible flags
        help_msg: "".to_string(),
        
        //standard flags
        help: false,
        version: false,
    };
    for s in arg_strings {
        match s.as_str() {
            "-h" | "--help" => a.help = true,
            "-v" | "--version" => a.version = true,
            _ => {a.help = true; a.help_msg = s},
        }
        if a.help {
            break;
        }
    }
    a
}

fn show_help(args: Args) -> u8 {
    let returnval = if !args.help_msg.is_empty() {
        println!("{} is an unrecognized argument.\n", args.help_msg);
        println!("Here's the help statement.");
        1
    } else { 0 };
    
    let help_msg = concat!("TasknetRS: A tool for preparing tasks\n",
    "\n",
    "  -h, --help\n",
    "          Show this help message\n",
    "  -v, --version\n",
    "          Show the current version\n",
    "",
    );
    println!("{}", help_msg);
    returnval
}

fn show_version() -> u8 {
    println!("TasknetRS Version {}", VERSION);
    0
}


