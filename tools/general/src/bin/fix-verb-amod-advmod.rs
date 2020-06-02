use std::io::BufWriter;

use clap::{App, AppSettings, Arg};
use conllu::graph::{DepTriple, Node, Sentence};
use conllu::io::{ReadSentence, Reader, WriteSentence, Writer};
use failure::Fallible;
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

        replace_verb_amod_advmod(&mut sentence).or_exit("Could not fix advmod relation", 1);

        writer
            .write_sentence(&sentence)
            .or_exit("Cannot write sentence", 1);
    }
}

fn replace_verb_amod_advmod(sent: &mut Sentence) -> Fallible<()> {
    for idx in 1..sent.len() {
        if let Some(head_idx) = qualifying_relation(sent, idx) {
            sent.dep_graph_mut().add_deprel(DepTriple::new(
                head_idx,
                Some("advmod".to_owned()),
                idx,
            ));
        }
    }

    Ok(())
}

fn qualifying_relation(sent: &Sentence, idx: usize) -> Option<usize> {
    let triple = sent
        .dep_graph()
        .head(idx)
        .ok_or_else(|| format!("Token {} does not have a head", idx))
        .or_exit("Cannot process sentence", 1);

    let rel = triple
        .relation()
        .ok_or_else(|| format!("Token {} does not have a head relation", idx))
        .or_exit("Cannot process sentence", 1);

    let head_pos = match &sent[triple.head()] {
        Node::Token(token) => token
            .upos()
            .ok_or_else(|| format!("Token {} does not have a universal POS tag", idx))
            .or_exit("Cannot process sentence", 1),
        Node::Root => return None,
    };

    if rel == "amod" && head_pos == "VERB" {
        Some(triple.head())
    } else {
        None
    }
}
