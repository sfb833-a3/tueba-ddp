use std::fs::File;
use std::io::BufReader;
use std::process;

use clap::{App, AppSettings, Arg};
use conllu::graph::Node;
use conllu::io::{ReadSentence, Reader};
use stdinout::OrExit;

static DEFAULT_CLAP_SETTINGS: &[AppSettings] = &[
    AppSettings::DontCollapseArgsInUsage,
    AppSettings::UnifiedHelpMessage,
];

fn main() {
    let matches = App::new("tdz-ud-ne-iob")
        .settings(DEFAULT_CLAP_SETTINGS)
        .arg(
            Arg::with_name("GOLD")
                .help("Gold annotations file")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("PREDICT")
                .help("Predicted annotations file")
                .required(true)
                .index(2),
        )
        .get_matches();

    let gold_file =
        File::open(matches.value_of("GOLD").unwrap()).or_exit("Cannot open gold annotations", 1);
    let gold_reader = Reader::new(BufReader::new(gold_file));

    let predict_file = File::open(matches.value_of("PREDICT").unwrap())
        .or_exit("Cannot open predicted annotations", 1);
    let predict_reader = Reader::new(BufReader::new(predict_file));

    for (gold_sentence, predict_sentence) in gold_reader.sentences().zip(predict_reader.sentences())
    {
        let gold_sentence = gold_sentence.or_exit("Cannot read gold sentence", 1);
        let predict_sentence = predict_sentence.or_exit("Cannot read predicted sentence", 1);

        if gold_sentence.len() != predict_sentence.len() {
            eprintln!("Length mismatch");
            process::exit(1);
        }

        for (gold_node, predict_node) in gold_sentence.iter().zip(predict_sentence.iter()) {
            if let Some(token) = gold_node.token() {
                let gold_entity = get_entity(gold_node).or_exit("Cannot get gold entity", 1);
                let predict_entity =
                    get_entity(predict_node).or_exit("Cannot get predicted entity", 1);

                println!("{}\t{}\t{}", token.form(), gold_entity, predict_entity);
            }
        }
    }
}

fn get_entity(node: &Node) -> Result<&str, &str> {
    let token = node.token().ok_or("Node is not a token")?;
    token
        .misc()
        .get("NE")
        .ok_or("Token without the 'NE' feature")?
        .as_ref()
        .ok_or("Token without a value for the 'NE' feature")
        .map(String::as_str)
}
