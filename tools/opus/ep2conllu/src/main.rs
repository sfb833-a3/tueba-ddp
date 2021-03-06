use std::env::args;
use std::io::{BufWriter, Read};
use std::mem;

use conllu::graph::{Comment, Sentence};
use conllu::io::WriteSentence;
use conllu::token::TokenBuilder;
use getopts::Options;
use stdinout::{Input, OrExit, Output};
use xml::attribute::OwnedAttribute;
use xml::reader::{EventReader, XmlEvent};

mod error;
use crate::error::ParseError;

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

    let input = Input::from(matches.free.get(0));
    let reader = EuroParlReader::new(
        input
            .buf_read()
            .or_exit("Could not open input for reading", 1),
        matches.free[0].clone(),
    );

    let output = Output::from(matches.free.get(2));
    let mut writer = conllu::io::Writer::new(BufWriter::new(
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
    p_id: Option<String>,
    sent_id: Option<String>,
    comments: Vec<Comment>,
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
            chap_id: None,
            p_id: None,
            sent_id: None,
            comments: vec![],
        }
    }

    fn read_sentence(&mut self) -> Result<Option<Sentence>, ParseError> {
        let mut sentence = Sentence::new();
        let mut in_token = false;
        let mut form = String::new();

        loop {
            let event = self.event_reader.next()?;

            use crate::XmlEvent::{Characters, EndDocument, EndElement, StartElement};
            match event {
                EndDocument => break,
                StartElement {
                    name, attributes, ..
                } => match name.local_name.as_str() {
                    "P" => {
                        self.p_id = get_attribute(&attributes, "id").map(str::to_owned);
                        self.comments.push(Comment::AttrVal {
                            attr: "newpar id".to_string(),
                            val: format!(
                                "{}-{}-{}",
                                &self.corpus_name,
                                self.chap_id.as_deref().expect("Unknown chapter"),
                                self.p_id.as_deref().unwrap_or("unk"),
                            ),
                        });
                    }
                    "s" => self.sent_id = get_attribute(&attributes, "id").map(str::to_owned),
                    "w" => in_token = true,
                    "CHAPTER" => self.chap_id = get_attribute(&attributes, "ID").map(str::to_owned),
                    "SPEAKER" => {
                        self.comments.push(Comment::AttrVal {
                            attr: "speaker id".to_string(),
                            val: get_attribute(&attributes, "ID")
                                .expect("Speaker without ID")
                                .to_owned(),
                        });
                        self.comments.push(Comment::AttrVal {
                            attr: "speaker name".to_string(),
                            val: get_attribute(&attributes, "NAME")
                                .expect("Speaker without name")
                                .to_owned(),
                        });
                    }
                    _ => (),
                },
                EndElement { name } => match name.local_name.as_str() {
                    "CHAPTER" => self.chap_id = None,
                    "P" => self.p_id = None,
                    "w" => {
                        if form.is_empty() {
                            return Err(ParseError::EmptyTokenError);
                        }

                        sentence.push(TokenBuilder::new(form.as_str()).into());
                        in_token = false;
                        form.clear();
                    }
                    "s" => {
                        let mut comments = Vec::new();
                        mem::swap(&mut self.comments, &mut comments);

                        comments.push(Comment::AttrVal {
                            attr: "sent_id".to_string(),
                            val: format!(
                                "{}-{}-{}-{}",
                                &self.corpus_name,
                                self.chap_id.as_deref().expect("Unknown chapter"),
                                self.p_id.as_deref().unwrap_or("unk"),
                                self.sent_id.as_deref().expect("Unknown sentence")
                            ),
                        });

                        sentence.set_comments(comments);

                        break;
                    }
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

        if sentence.len() == 1 {
            Ok(None)
        } else {
            Ok(Some(sentence))
        }
    }
}

impl<R: Read> IntoIterator for EuroParlReader<R> {
    type Item = Result<Sentence, ParseError>;
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
    type Item = Result<Sentence, ParseError>;

    fn next(&mut self) -> Option<Result<Sentence, ParseError>> {
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
