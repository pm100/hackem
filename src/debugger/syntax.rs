use clap::{arg, Arg};
use clap::{ArgGroup, Command};

pub fn syntax() -> Command {
    const PARSER_TEMPLATE: &str = "\
        {all-args}
    ";
    const APPLET_TEMPLATE: &str = "\
        {about-with-newline}\n\
        {usage-heading}\n    {usage}\n\
        \n\
        {all-args}{after-help}\
    ";

    Command::new("hackem")
        .multicall(true)
        .arg_required_else_help(true)
        .subcommand_required(true)
        .subcommand_value_name("Command")
        .subcommand_help_heading("Commands")
        .help_template(PARSER_TEMPLATE)
        .subcommand(
            Command::new("load_code")
                .visible_alias("load")
                .about("Load binary file (.hx or raw binary)")
                .arg(Arg::new("file").required(true))
                .arg_required_else_help(true)
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("load_pdb")
                .visible_alias("pdb")
                .about("Load debug info file (JSON)")
                .arg(Arg::new("file").required(true))
                .arg_required_else_help(true)
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("quit")
                .visible_aliases(["exit", "q"])
                .about("Quit")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("next_instruction")
                .visible_alias("ni")
                .about("Step one instruction")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("step_instruction")
                .visible_alias("si")
                .about("Step one instruction (alias of ni)")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("go")
                .visible_alias("g")
                .about("Resume execution")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("break")
                .about("Set breakpoint")
                .visible_alias("b")
                .arg(Arg::new("address").required(true))
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("watch")
                .about("Set watchpoint")
                .visible_alias("w")
                .arg(Arg::new("address").required(true))
                .arg(arg!(-r --read   "watch for reads"))
                .arg(arg!(-w --write  "watch for writes"))
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("list_breakpoints")
                .about("List breakpoints")
                .visible_alias("lbp")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("delete_breakpoint")
                .visible_alias("dbp")
                .arg(Arg::new("address").required(false))
                .about("Delete breakpoint (omit address to delete all)")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("list_watchpoints")
                .about("List watchpoints")
                .visible_alias("lwp")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("delete_watchpoint")
                .visible_alias("dwp")
                .arg(Arg::new("address").required(false))
                .about("Delete watchpoint (omit address to delete all)")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("dis")
                .about("Disassemble instructions")
                .arg(Arg::new("address").help("Start address (default: PC)"))
                .arg(
                    Arg::new("count")
                        .short('n')
                        .long("count")
                        .value_parser(clap::value_parser!(u16))
                        .default_value("16")
                        .help("Number of instructions"),
                )
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("display_memory")
                .visible_aliases(["mem", "m"])
                .about("Display RAM as hex")
                .arg(Arg::new("address").required(true))
                .arg(
                    Arg::new("count")
                        .short('n')
                        .long("count")
                        .value_parser(clap::value_parser!(u16))
                        .default_value("16")
                        .help("Number of words"),
                )
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("print")
                .visible_alias("p")
                .arg(arg!(<address> "address of value to print"))
                .arg(arg!(asint:     -i "integer"))
                .arg(arg!(asstring:  -s "string"))
                .group(ArgGroup::new("format").args(["asint", "asstring"]))
                .about("Formatted display of a memory value")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("reg")
                .about("Display CPU registers")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("write_memory")
                .visible_alias("wm")
                .about("Write a value to RAM")
                .arg(arg!(<address> "address to write to"))
                .arg(arg!(<value>   "value (integer or expression)"))
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("list_symbols")
                .visible_alias("lsy")
                .arg(Arg::new("match").required(false))
                .about("List PDB symbols (optional substring filter)")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("expr")
                .arg(arg!(<expression> "expression to evaluate"))
                .about("Evaluate an address expression")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("cd")
                .about("Change working directory")
                .arg(Arg::new("directory").required(true))
                .arg_required_else_help(true)
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("about")
                .about("Help for commands")
                .arg(arg!([topic]))
                .help_template(APPLET_TEMPLATE),
        )
}
