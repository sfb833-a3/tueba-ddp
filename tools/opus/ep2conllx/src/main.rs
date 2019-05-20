extern crate conllx;
#[macro_use]
extern crate error_chain;
extern crate flate2;
extern crate getopts;
extern crate stdinout;
extern crate xml;

use std::collections::BTreeMap;
use std::env::args;
use std::io::{BufWriter, Read};

use conllx::{Features, Sentence, TokenBuilder, WriteSentence};
use getopts::Options;
use stdinout::{Input, OrExit, Output};
use xml::attribute::OwnedAttribute;
use xml::reader::{EventReader, XmlEvent};

mod error;
use error::*;

fn print_usage(program: &str, opts: Options) {
    let brief = format!(
        "Usage: {} [options] CORPUS_ID [INPUT_FILE] [OUTPUT_FILE]",
        program
    );
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    let matches = opts.parse(&args[1..]).or_exit("Could not parse options", 1);

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    if matches.free.is_empty() || matches.free.len() > 3 {
        print_usage(&program, opts);
        return;
    }

    let input = Input::from(matches.free.get(1));
    let reader = EuroParlReader::new(
        input
            .buf_read()
            .or_exit("Could not open input for reading", 1),
        matches.free[0].clone(),
    );

    let output = Output::from(matches.free.get(2));
    let mut writer = conllx::Writer::new(BufWriter::new(
        output
            .write()
            .or_exit("Could not open output for writing", 1),
    ));

    for sentence in reader {
        let sentence = sentence.or_exit("Cannot read sentence", 1);
        writer
            .write_sentence(&sentence)
            .or_exit("Cannot write sentence", 1);
    }
}

struct EuroParlReader<R>
where
    R: Read,
{
    event_reader: EventReader<R>,
    corpus_name: String,
    chap_id: Option<String>,
    speaker_id: Option<String>,
    p_id: Option<String>,
}

impl<R> EuroParlReader<R>
where
    R: Read,
{
    fn new<S>(read: R, corpus_name: S) -> Self
    where
        S: Into<String>,
    {
        EuroParlReader {
            event_reader: EventReader::new(read),
            corpus_name: corpus_name.into(),
            speaker_id: None,
            chap_id: None,
            p_id: None,
        }
    }

    fn read_sentence(&mut self) -> Result<Option<Sentence>> {
        let mut tokens = Vec::new();
        let mut in_token = false;
        let mut form = String::new();

        loop {
            let event = self.event_reader.next()?;

            use XmlEvent::*;
            match event {
                EndDocument => break,
                StartElement {
                    name, attributes, ..
                } => match name.local_name.as_str() {
                    "P" => self.p_id = get_attribute(&attributes, "id").map(str::to_owned),
                    "w" => in_token = true,
                    "CHAPTER" => self.chap_id = get_attribute(&attributes, "ID").map(str::to_owned),
                    "SPEAKER" => {
                        self.speaker_id = get_attribute(&attributes, "ID").map(str::to_owned)
                    }
                    _ => (),
                },
                EndElement { name } => match name.local_name.as_str() {
                    "CHAPTER" => self.chap_id = None,
                    "SPEAKER" => self.speaker_id = None,
                    "P" => self.p_id = None,
                    "w" => {
                        if form.is_empty() {
                            return Err(ErrorKind::EmptyTokenError.into());
                        }

                        let mut features = BTreeMap::new();
                        features.insert("subcorpus".to_owned(), Some(self.corpus_name.clone()));
                        if self.chap_id.is_some() {
                            features.insert("chapter".to_owned(), self.chap_id.clone());
                        }
                        if self.speaker_id.is_some() {
                            features.insert("speaker".to_owned(), self.speaker_id.clone());
                        }
                        if self.p_id.is_some() {
                            features.insert("para".to_owned(), self.p_id.clone());
                        }

                        tokens.push(
                            TokenBuilder::new(form.as_str())
                                .features(Features::from_iter(features))
                                .token(),
                        );
                        in_token = false;
                        form.clear();
                    }
                    "s" => break,
                    _ => (),
                },
                Characters(chars) => {
                    if in_token {
                        form.push_str(&chars);
                    }
                }
                _ => (),
            }
        }

        if tokens.is_empty() {
            Ok(None)
        } else {
            Ok(Some(Sentence::new(tokens)))
        }
    }
}

impl<R: Read> IntoIterator for EuroParlReader<R> {
    type Item = Result<Sentence>;
    type IntoIter = Sentences<R>;

    fn into_iter(self) -> Self::IntoIter {
        Sentences { reader: self }
    }
}

pub struct Sentences<R>
where
    R: Read,
{
    reader: EuroParlReader<R>,
}

impl<R> Iterator for Sentences<R>
where
    R: Read,
{
    type Item = Result<Sentence>;

    fn next(&mut self) -> Option<Result<Sentence>> {
        match self.reader.read_sentence() {
            Ok(None) => None,
            Ok(Some(sent)) => Some(Ok(sent)),
            Err(e) => Some(Err(e)),
        }
    }
}

fn get_attribute<'a>(attrs: &'a [OwnedAttribute], attr: &str) -> Option<&'a str> {
    attrs
        .iter()
        .find(|a| a.name.local_name == attr)
        .map(|a| a.value.as_str())
}
