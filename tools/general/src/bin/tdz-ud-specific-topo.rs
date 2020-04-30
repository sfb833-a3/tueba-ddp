use std::io::BufWriter;

use clap::{App, AppSettings, Arg};
use conllu::io::{ReadSentence, Reader, WriteSentence, Writer};
use stdinout::{Input, OrExit, Output};

static DEFAULT_CLAP_SETTINGS: &[AppSettings] = &[
    AppSettings::DontCollapseArgsInUsage,
    AppSettings::UnifiedHelpMessage,
];

fn main() {
    let matches = App::new("tdz-ud-specific-topo")
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
                if let Some(topo) = token.misc_mut().get_mut("TopoField") {
                    let topo = topo
                        .as_mut()
                        .or_exit("Topological field feature without value: {}", 1);
                    let most_specific_topo = topo.split('-').last().or_exit(
                        format!("Incomplete topological field annotation: {}", topo),
                        1,
                    );

                    *topo = most_specific_topo.to_owned();
                }
            }
        }

        writer
            .write_sentence(&sentence)
            .or_exit("Cannot write sentence", 1);
    }
}
