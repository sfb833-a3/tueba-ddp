use std::collections::btree_map::Entry;
use std::io::BufWriter;

use clap::{App, AppSettings, Arg};
use conllu::io::{ReadSentence, Reader, WriteSentence, Writer};
use stdinout::{Input, OrExit, Output};

static DEFAULT_CLAP_SETTINGS: &[AppSettings] = &[
    AppSettings::DontCollapseArgsInUsage,
    AppSettings::UnifiedHelpMessage,
];

fn main() {
    let matches = App::new("tdz-ud-ne-iob")
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

        let mut last_id = None;

        for node in sentence.iter_mut() {
            if let Some(token) = node.token_mut() {
                // Temporarily work around the borrows checker
                let token_err = token.clone();

                match token.misc_mut().entry("NE".to_owned()) {
                    Entry::Vacant(entry) => {
                        entry.insert(Some("O".to_owned()));
                        last_id = None;
                    }
                    Entry::Occupied(mut entry) => {
                        let entity = entry
                            .get_mut()
                            .as_mut()
                            .ok_or_else(|| {
                                format!("Named entity feature with missing entity: {:?}", token_err)
                            })
                            .or_exit("Could not process token", 1);
                        fixup_entity(entity, &mut last_id);
                    }
                }
            }
        }

        writer
            .write_sentence(&sentence)
            .or_exit("Cannot write sentence", 1);
    }
}

fn fixup_entity(ud_entity: &mut String, last_id: &mut Option<String>) {
    // Entities can consist of a chain of labels, such as:
    //
    // NE:ORG_1627129-LOC_1627129a
    //
    // Get the first part.
    let entity = ud_entity.split('-').next().expect("Empty entity");

    // The entity and identifier are separated by an underscore. E.g.:
    let sep_idx = entity.find('_').expect("Entity without separator");
    let id = Some(entity[sep_idx + 1..].to_string());
    let entity = &entity[..sep_idx];

    if id == *last_id {
        *ud_entity = format!("I-{}", entity);
    } else {
        *ud_entity = format!("B-{}", entity);
        *last_id = id
    }
}
