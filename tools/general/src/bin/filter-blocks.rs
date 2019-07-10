use std::char::REPLACEMENT_CHARACTER;
use std::io::{BufRead, BufWriter, Write};

use clap::{App, AppSettings, Arg, ArgMatches};
use stdinout::{Input, OrExit, Output};

static INPUT: &'static str = "INPUT";
static OUTPUT: &'static str = "OUTPUT";
static REPLACEMENT: &'static str = "REPLACEMENT";

fn parse_args() -> ArgMatches<'static> {
    App::new("filter-blocks")
        .settings(DEFAULT_CLAP_SETTINGS)
        .arg(
            Arg::with_name(REPLACEMENT)
                .long("replacement")
                .help("Remove blocks with unicode replacement characters"),
        )
        .arg(Arg::with_name(INPUT).help("Input data").index(1))
        .arg(Arg::with_name(OUTPUT).help("Output data").index(1))
        .get_matches()
}

struct Config {
    input: Option<String>,
    output: Option<String>,
    replacement: bool,
}

fn config_from_matches<'a>(matches: &ArgMatches<'a>) -> Config {
    let input = matches.value_of(INPUT).map(ToOwned::to_owned);
    let output = matches.value_of(OUTPUT).map(ToOwned::to_owned);
    let replacement = matches.is_present(REPLACEMENT);

    Config {
        input,
        output,
        replacement,
    }
}

static DEFAULT_CLAP_SETTINGS: &[AppSettings] = &[
    AppSettings::DontCollapseArgsInUsage,
    AppSettings::UnifiedHelpMessage,
];

fn main() {
    let matches = parse_args();
    let config = config_from_matches(&matches);

    let input = Input::from(config.input);
    let output = Output::from(config.output);

    let reader = input.buf_read().or_exit("Cannot open input for reading", 1);
    let mut writer = BufWriter::new(output.write().or_exit("Cannot open output for writing", 1));

    for line in reader.lines() {
        let line = line.or_exit("Cannot read line", 1);

        if config.replacement && line.contains(REPLACEMENT_CHARACTER) {
            continue;
        }

        writeln!(&mut writer, "{}", line).or_exit("Cannot write block", 1);
    }
}
