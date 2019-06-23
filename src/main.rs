use clap::{App, Arg, SubCommand};

fn main() {
    let mut app = App::new("opentracker exporter")
        .version(env!("CARGO_PKG_VERSION")) // load version from cargo
        .author("Finn Behrens <me@kloenk.de>")
        .about("Exporter for opentracker stats to a prometheus stats endpoint")
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .multiple(true)
                .help("sets the level of verbosity"),
        )
        .arg(
            Arg::with_name("url")
                .short("u")
                .long("url")
                .help("set opentracker stats host")
                .value_name("URL"),
        )
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .help("set the port for the local webserver")
                .value_name("PORT"),
        )
        .subcommand(
            SubCommand::with_name("completion")
                .about("create completions")
                .version("0.1.0")
                .author("Finn Behrens <me@kloenk.de>")
                .arg(
                    Arg::with_name("shell")
                        .help("set the shell to create for. Tries to identify with env variable")
                        .index(1)
                        .required(false)
                        .value_name("SHELL")
                        .possible_value("fish")
                        .possible_value("bash")
                        .possible_value("zsh")
                        .possible_value("powershell")
                        .possible_value("elvish"),
                )
                .arg(
                    Arg::with_name("out")
                        .help("sets output file")
                        .value_name("FILE")
                        .short("o")
                        .long("output"),
                )
                .setting(clap::AppSettings::ColorAuto)
                .setting(clap::AppSettings::ColoredHelp),
        )
        .setting(clap::AppSettings::ColorAuto)
        .setting(clap::AppSettings::ColoredHelp);

    let matches = app.clone().get_matches();

    // run subcommands
    if let Some(matches) = matches.subcommand_matches("completion") {
        completion(&matches, &mut app);
        std::process::exit(0);
    }
    drop(app);

    let mut conf = opentracker_exporter::Config::new();

    // read verbose value
    conf.verbose = matches.occurrences_of("verbose") as u8;

    if let Some(url) = &matches.value_of("url") {
        conf.url = url.to_string();
    }

    if let Some(port) = &matches.value_of("port") {
        conf.port = port.parse().unwrap_or_else(|_| conf.port);
    }

    if conf.verbose >= 1 {
        println!("Debug{}: enabled", conf.verbose);
    }

    conf.run().unwrap();
}

// create completion
fn completion(args: &clap::ArgMatches, app: &mut App) {
    let shell: String = match args.value_of("shell") {
        Some(shell) => shell.to_string(),
        None => {
            let shell = match std::env::var("SHELL") {
                Ok(shell) => shell,
                Err(_) => "/bin/bash".to_string(),
            };
            let shell = std::path::Path::new(&shell);
            match shell.file_name() {
                Some(shell) => shell.to_os_string().to_string_lossy().to_string(),
                None => "bash".to_string(),
            }
        }
    };

    use clap::Shell;
    let shell_l = shell.to_lowercase();
    let shell: Shell;
    if shell_l == "fish".to_string() {
        shell = Shell::Fish;
    } else if shell_l == "zsh".to_string() {
        shell = Shell::Zsh;
    } else if shell_l == "powershell".to_string() {
        shell = Shell::PowerShell;
    } else if shell_l == "elvish".to_string() {
        shell = Shell::Elvish;
    } else {
        shell = Shell::Bash;
    }

    use std::fs::File;
    use std::io::BufWriter;
    use std::io::Write;

    let mut path = BufWriter::new(match args.value_of("out") {
        Some(x) => Box::new(
            File::create(&std::path::Path::new(x)).unwrap_or_else(|err| {
                eprintln!("Error opening file: {}", err);
                std::process::exit(1);
            }),
        ) as Box<Write>,
        None => Box::new(std::io::stdout()) as Box<Write>,
    });

    app.gen_completions_to("raspi_firmware", shell, &mut path);
}
