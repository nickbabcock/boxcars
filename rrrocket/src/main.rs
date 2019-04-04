#[cfg(test)]
extern crate assert_cli;
extern crate boxcars;
extern crate failure;
extern crate globset;
extern crate rayon;
extern crate serde_json;
extern crate structopt;

use boxcars::{CrcCheck, NetworkParse, ParserBuilder, Replay};
use failure::{err_msg, Error, ResultExt};
use globset::Glob;
use rayon::prelude::*;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::io::{self, BufWriter};
use std::path::Path;
use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone, PartialEq)]
#[structopt(
    name = "rrrocket",
    about = "Parses Rocket League replay files and outputs JSON with decoded information"
)]
struct Opt {
    #[structopt(
        short = "c",
        long = "crc-check",
        help = "forces a crc check for corruption even when replay was successfully parsed"
    )]
    crc: bool,

    #[structopt(
        short = "n",
        long = "network-parse",
        help = "parses the network data of a replay instead of skipping it"
    )]
    body: bool,

    #[structopt(
        short = "m",
        long = "multiple",
        help = "parse multiple replays, instead of writing JSON to stdout, write to a sibling JSON file"
    )]
    multiple: bool,

    #[structopt(
        long = "dry-run",
        help = "parses but does not write JSON output"
    )]
    dry_run: bool,

    #[structopt(help = "Rocket League replay files")]
    input: Vec<String>,
}

fn read_file(input: &str) -> Result<Vec<u8>, Error> {
    let mut f = File::open(input)
        .with_context(|_e| format!("Could not open rocket league file: {}", input))?;
    let mut buffer = vec![];
    f.read_to_end(&mut buffer)
        .with_context(|_e| format!("Could not read rocket league file: {}", input))?;
    Ok(buffer)
}

fn parse_replay<'a>(opt: &Opt, data: &'a [u8]) -> Result<Replay<'a>, Error> {
    ParserBuilder::new(&data[..])
        .with_crc_check(if opt.crc {
            CrcCheck::Always
        } else {
            CrcCheck::OnError
        }).with_network_parse(if opt.body {
            NetworkParse::Always
        } else {
            NetworkParse::Never
        }).parse()
}

/// Each file argument that we get could be a directory so we need to expand that directory and
/// find all the *.replay files. File arguments turn into single element vectors.
fn expand_paths(files: &[String]) -> Result<Vec<Vec<String>>, Error> {
    let glob = Glob::new("*.replay")?.compile_matcher();

    files
        .iter()
        .map(|x| {
            let p = Path::new(x);
            if p.is_dir() {
                // If the commandline argument is a directory we look for all files that match
                // *.replay. A file that does not match the pattern because of an error reading the
                // directory / file will not be filtered and will cause the error to bubble up. In
                // the future, we could get smart and ignore directories / files we don't have
                // permission that wouldn't match the pattern anyways
                let files: Result<Vec<_>, _> = p
                    .read_dir()?
                    .filter_map(|entry| {
                        match entry {
                            Ok(y) => {
                                if glob.is_match(y.path()) {
                                    // Force UTF-8. There is a special place in the fourth circle
                                    // of inferno for people who rename their rocket league replays
                                    // to not contain UTF-8. We won't panic, but will cause an
                                    // error when the file is attempted to be read.
                                    Some(Ok(y.path().to_string_lossy().into_owned()))
                                } else {
                                    None
                                }
                            }
                            Err(e) => Some(Err(e)),
                        }
                    }).collect();
                Ok(files?)
            } else {
                Ok(vec![x.clone()])
            }
        }).collect()
}

fn parse_multiple_replays(opt: &Opt) -> Result<(), Error> {
    let res: Result<Vec<()>, Error> = expand_paths(&opt.input)?
        .into_iter()
        .flat_map(|x| x)
        .collect::<Vec<_>>()
        .par_iter()
        .map(|file| {
            let data = read_file(file)?;
            let replay = parse_replay(opt, &data[..])
                .with_context(|_e| format!("Could not parse: {}", file))?;

            if !opt.dry_run {
                let outfile = format!("{}.json", file);
                let fout = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(&outfile)
                    .with_context(|_e| format!("Could not open json output file: {}", outfile))?;

                let mut writer = BufWriter::new(fout);
                serde_json::to_writer(&mut writer, &replay)
                    .with_context(|_e| format!("Could not serialize replay {}", file))?;
            }
            Ok(())
        }).collect();
    res?;
    Ok(())
}

fn run() -> Result<(), Error> {
    let opt = Opt::from_args();
    if opt.multiple {
        parse_multiple_replays(&opt)
    } else if opt.input.len() != 1 {
        Err(err_msg(
            "Expected one input file if --multiple is not specified",
        ))
    } else {
        let file = &opt.input[0];
        let data = read_file(file)?;
        let replay = parse_replay(&opt, &data[..]).context("Could not parse replay")?;
        if !opt.dry_run {
            serde_json::to_writer(&mut io::stdout(), &replay)
                .context("Could not serialize replay")?;
        }
        Ok(())
    }
}

fn main() {
    if let Err(ref e) = run() {
        let mut stderr = io::stderr();
        for fail in e.iter_chain() {
            let _ = writeln!(stderr, "{}", fail);
        }

        ::std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use assert_cli::Assert;
    use std::io::Write;

    #[test]
    fn test_error_output() {
        let mut w = Vec::new();
        writeln!(
            &mut w,
            "Could not open rocket league file: assets/fuzz-string-too-long.replay"
        ).unwrap();

        Assert::cargo_binary("rrrocket")
            .with_args(&[
                "-n",
                "-c",
                "--dry-run",
                "assets/fuzz-string-too-long.replay",
            ]).fails_with(1)
            .stderr()
            .contains(String::from_utf8(w).unwrap())
            .unwrap();
    }

    #[test]
    fn test_error_output2() {
        let mut w = Vec::new();
        writeln!(
            &mut w,
            "Could not parse: ../assets/fuzz-string-too-long.replay"
        ).unwrap();
        writeln!(
            &mut w,
            "Crc mismatch. Expected 3765941959 but received 1825689991"
        ).unwrap();

        Assert::cargo_binary("rrrocket")
            .with_args(&[
                "-n",
                "-c",
                "--dry-run",
                "-m",
                "../assets/fuzz-string-too-long.replay",
            ]).fails_with(1)
            .stderr()
            .contains(String::from_utf8(w).unwrap())
            .unwrap();
    }
}
