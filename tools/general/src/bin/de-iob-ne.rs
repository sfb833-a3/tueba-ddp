use std::cmp::min;
use std::collections::BTreeMap;
use std::io::BufWriter;

use clap::{App, AppSettings, Arg};
use conllx::io::{ReadSentence, Reader, WriteSentence, Writer};
use conllx::token::{Features, Token};
use failure::{bail, format_err, Fallible};
use stdinout::{Input, OrExit, Output};

static DEFAULT_CLAP_SETTINGS: &[AppSettings] = &[
    AppSettings::DontCollapseArgsInUsage,
    AppSettings::UnifiedHelpMessage,
];

fn main() {
    let matches = App::new("de-iob-ner")
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
                de_iob_token(token).or_exit("Could not de-IOB token", 1);
            }
        }

        writer
            .write_sentence(&sentence)
            .or_exit("Cannot write sentence", 1);
    }
}

fn de_iob_token(token: &mut Token) -> Fallible<()> {
    if let Some(old_features) = token.features().map(Features::as_map) {
        let new_features = rewrite_features(token, old_features)?;

        if new_features.is_empty() {
            token.set_features(None);
        } else {
            token.set_features(Some(Features::from_iter(new_features)));
        }
    }

    Ok(())
}

fn rewrite_features(
    token: &Token,
    old_features: &BTreeMap<String, Option<String>>,
) -> Fallible<BTreeMap<String, Option<String>>> {
    let mut new_features = old_features.clone();
    new_features.remove("NE");

    if let Some(tag) = old_features.get("NE") {
        let tag = tag
            .as_ref()
            .ok_or_else(|| format_err!("Named entity without a tag: {:?}", token))?;

        // Remove 'O' classifications, strip 'B-/O' prefix.
        match &tag[..min(tag.len(), 2)] {
            "O" => (),
            "B-" => {
                new_features.insert("ne".to_string(), Some(tag[2..].to_string()));
                new_features.insert("ne_chunk".to_string(), Some("B".to_string()));
            }
            "I-" => {
                new_features.insert("ne".to_string(), Some(tag[2..].to_string()));
                new_features.insert("ne_chunk".to_string(), Some("I".to_string()));
            }
            other => {
                bail!("Unknown NE prefix {} in {:?}", other, token);
            }
        }
    };

    Ok(new_features)
}
