use std::io::BufWriter;

use clap::{App, AppSettings, Arg};
use conllx::io::{ReadSentence, Reader, WriteSentence, Writer};
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
    let reader = Reader::new(input.buf_read().or_exit("Cannot open input", 1));

    let output = Output::from(matches.value_of("OUTPUT"));
    let mut writer = Writer::new(BufWriter::new(
        output.write().or_exit("Cannot create output", 1),
    ));

    for sentence in reader.sentences() {
        let mut sentence = sentence.or_exit("Cannot read sentence", 1);

        for node in sentence.iter_mut() {
            if let Some(token) = node.token_mut() {
                if let Some(pos) = token.pos() {
                    let sep_idx = pos.find('-').or_exit("Tag without separator", 1);
                    let ut = pos[..sep_idx].to_string();
                    let stts = pos[sep_idx + 1..].to_string();

                    token.set_cpos(Some(ut));
                    token.set_pos(Some(stts));
                };
            }
        }

        writer
            .write_sentence(&sentence)
            .or_exit("Cannot write sentence", 1);
    }
}
