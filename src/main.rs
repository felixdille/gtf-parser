use itertools::{self, Itertools};
use pprof::ProfilerGuard;
use std::{
    cmp::min,
    collections::{HashMap, HashSet},
    fs::File,
    io::{self, BufRead},
    println,
    str::FromStr,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

use crate::Strand::{MINUS, PLUS};
use tracing::{span, Level};

#[derive(Debug)]
enum Strand {
    PLUS,
    MINUS,
}

#[derive(Debug)]
struct GTFEntry {
    seq_id: String,
    source: String,
    feature: String,
    start: usize,
    end: usize,
    score: Option<f64>,
    strand: Strand,
    frame: Option<u8>,
    attribute: HashMap<String, String>,
}

fn get_strand(value: &str) -> Strand {
    if value == "+" { PLUS } else { MINUS }
}

fn parse_attributes(value: &str) -> String {
    for attribute in value.split(";") {
        let attribute = attribute.trim();
        let pair: Vec<&str> = attribute.splitn(2, " ").collect();

        if pair.len() < 2 {
            continue;
        }

        let attribute_value: Vec<&str> = pair[1].split("\"").collect();
        let key = pair[0];

        if key == "gene_id" {
            return String::from(attribute_value[min(1, attribute_value.len()-1)]);
        }
    }
    "".to_string()
}

fn get_value<T>(value: &str) -> Option<T>
where
    T: FromStr,
{
    if value == "." {
        None
    } else {
        let result = value.parse();
        if let Ok(result) = result {
            Some(result)
        } else {
            None
        }
    }
}

fn parse_gtf_line(value: &str) -> Option<String> {
    let line = String::from(value);
    if line.starts_with("#") {
        return None;
    }

    let parts: Vec<&str> = line.split("\t").collect();

    let gene = parse_attributes(parts[8]);

    if gene != "".to_string() {
        return Some(gene);
    } else {
        return None;
    }
}

fn main() {
    // let guard = ProfilerGuard::new(100).unwrap();

    let span = span!(Level::TRACE, "my_span");

    let _enter = span.enter();

    let file_path = "/home/felixd/Downloads/GCF_000001405.40_GRCh38.p14_genomic.gtf";
    let file_handle = std::fs::File::open(file_path).unwrap();
    let buf = io::BufReader::with_capacity(50*1024*1024, file_handle);

    let genes = Arc::new(Mutex::new(Vec::<String>::new()));

    let join_handles: Vec<_> = buf
        .lines()
        .chunks(300000)
        .into_iter()
        .map(|chunk| {
            let thread_genes = Arc::clone(&genes);
            let lines = chunk.collect_vec();

            let join_handle = thread::spawn(move || {
                let mut found_genes = Vec::new();
                for line in lines {
                    if let Ok(line) = line {
                        if let Some(gene) = parse_gtf_line(&line) {
                            found_genes.push(gene);
                        }
                    }
                }

                let mut thread_genes = thread_genes.lock().unwrap();
                thread_genes.append(&mut found_genes);
            });
            join_handle
        })
        .collect();

    join_handles
        .into_iter()
        .for_each(|join_handle| join_handle.join().unwrap());

    println!(
        "{}",
        HashSet::<&String>::from_iter(genes.lock().unwrap().iter()).len()
    );

    // if let Ok(report) = guard.report().build() {
    //     let file = File::create("flamegraph.svg").unwrap();
    //     report.flamegraph(file).unwrap();
    // };
}
