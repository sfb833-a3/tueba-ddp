extern crate clap;
extern crate flate2;
extern crate siphasher;
extern crate stdinout;

use std::collections::HashMap;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{self, stdin, BufRead, BufWriter, Write};
use std::path::{Path, PathBuf};

use clap::{App, AppSettings, Arg};
use flate2::write::GzEncoder;
use flate2::Compression;
use siphasher::sip::SipHasher;
use stdinout::OrExit;

static DEFAULT_CLAP_SETTINGS: &[AppSettings] = &[
    AppSettings::DontCollapseArgsInUsage,
    AppSettings::UnifiedHelpMessage,
];

fn main() {
    let matches = App::new("shard-corpus")
        .settings(DEFAULT_CLAP_SETTINGS)
        .arg(
            Arg::with_name("bits")
                .short("b")
                .long("bits")
                .takes_value(true)
                .value_name("BITS")
                .help("Number of hash bits to use in sharding"),
        )
        .arg(
            Arg::with_name("gzip")
                .short("g")
                .long("gzip")
                .help("gzip shards"),
        )
        .arg(Arg::with_name("DIR").help("Output directory").index(1))
        .get_matches();

    let output_dir = if let Some(dir) = matches.value_of("DIR") {
        Path::new(dir)
    } else {
        Path::new(".")
    };

    let n_bits = matches
        .value_of("bits")
        .unwrap_or("8")
        .parse::<u64>()
        .or_exit("Cannot parse number of bits", 1);
    let mask = if n_bits > 63 { !0 } else { (1 << n_bits) - 1 };

    let stdin = stdin();
    let reader = stdin.lock();

    let mut writers = HashMap::new();

    for line in reader.lines() {
        let line = line.or_exit("Cannot read line", 1);
        let mut hasher = SipHasher::new();
        line.hash(&mut hasher);
        let trunc_hash = hasher.finish() & mask;

        let filename = format!("{:x}", trunc_hash);

        let mut writer = writers.entry(filename.clone()).or_insert_with(|| {
            BufWriter::new(
                create_shard_writer(&output_dir, &filename, matches.is_present("gzip"))
                    .or_exit("Cannot create shard file", 1),
            )
        });
        writeln!(&mut writer, "{}", line).or_exit("Could not write line to shard", 1);
    }
}

fn create_shard_writer(output_dir: &Path, shard: &str, gzip: bool) -> io::Result<Box<Write>> {
    let mut path = PathBuf::new();
    path.push(output_dir);
    path.push(shard);

    let writer: Box<Write> = if gzip {
        path.set_extension("gz");
        let f = File::create(path)?;
        Box::new(GzEncoder::new(f, Compression::fast()))
    } else {
        Box::new(File::create(path)?)
    };

    Ok(writer)
}
