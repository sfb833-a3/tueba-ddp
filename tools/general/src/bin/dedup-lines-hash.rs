use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::io::{BufRead, BufWriter, Write};

use clap::{App, AppSettings, Arg};
use siphasher::sip::SipHasher;
use stdinout::{Input, OrExit, Output};

static DEFAULT_CLAP_SETTINGS: &[AppSettings] = &[
    AppSettings::DontCollapseArgsInUsage,
    AppSettings::UnifiedHelpMessage,
];

fn main() {
    let matches = App::new("dedup-lines-hash")
        .settings(DEFAULT_CLAP_SETTINGS)
        .arg(Arg::with_name("INPUT").help("Input file").index(1))
        .arg(Arg::with_name("OUTPUT").help("Output file").index(2))
        .get_matches();

    let input = Input::from(matches.value_of("INPUT"));
    let reader = input.buf_read().or_exit("Cannot open input", 1);

    let output = Output::from(matches.value_of("OUTPUT"));
    let mut writer = BufWriter::new(output.write().or_exit("Cannot create output", 1));

    let mut seen = HashSet::new();

    for line in reader.lines() {
        let line = line.or_exit("Cannot read line", 1);
        let mut hasher = SipHasher::new();
        line.hash(&mut hasher);
        let hash = hasher.finish();

        if seen.insert(hash) {
            writeln!(writer, "{}", line).or_exit("Cannot write line", 1);
        }
    }
}
