use std::char::REPLACEMENT_CHARACTER;
use std::io::{BufRead, BufWriter, Write};

use clap::{App, AppSettings, Arg, ArgMatches};
use lazy_static::lazy_static;
use regex::{Regex, RegexSet};
use stdinout::{Input, OrExit, Output};

static INPUT: &str = "INPUT";
static OUTPUT: &str = "OUTPUT";
static HTML: &str = "HTML";
static REPLACEMENT: &str = "REPLACEMENT";

lazy_static! {
    static ref BR_RE: Regex = Regex::new("<br */?>").unwrap();
    static ref HTML_TAG_RE: RegexSet = RegexSet::new(&[
        "<[^>]+/>",
        "<[[:alnum:]:]+>",
        "<[[:alnum:]:]+ [^>]+=[^>]+>",
        "</[[:alnum:]]"
    ])
    .unwrap();
}

fn parse_args() -> ArgMatches<'static> {
    App::new("filter-blocks")
        .settings(DEFAULT_CLAP_SETTINGS)
        .arg(
            Arg::with_name(HTML)
                .long("html")
                .help("Remove br tags, remove blocks with HTML"),
        )
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
    html: bool,
    input: Option<String>,
    output: Option<String>,
    replacement: bool,
}

fn config_from_matches<'a>(matches: &ArgMatches<'a>) -> Config {
    let html = matches.is_present(HTML);
    let input = matches.value_of(INPUT).map(ToOwned::to_owned);
    let output = matches.value_of(OUTPUT).map(ToOwned::to_owned);
    let replacement = matches.is_present(REPLACEMENT);

    Config {
        html,
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
        let mut line = line.or_exit("Cannot read line", 1);

        if config.replacement && line.contains(REPLACEMENT_CHARACTER) {
            continue;
        }

        if config.html {
            line = BR_RE.replace_all(&line, " ").into_owned();

            if HTML_TAG_RE.is_match(&line) {
                continue;
            }
        }

        writeln!(&mut writer, "{}", line).or_exit("Cannot write block", 1);
    }
}
