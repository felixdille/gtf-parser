use itertools;
use log::debug;
use pprof::ProfilerGuard;
use std::{
    cmp::min,
    collections::{HashMap, HashSet},
    fs::{self, File},
    io::{self, BufRead, BufReader, Read},
    ops::Add,
    println,
    str::FromStr,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

use flate2::read::GzDecoder;

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
            return String::from(attribute_value[min(1, attribute_value.len() - 1)]);
        }
    }
    "".to_string()
}

fn parse_gtf_line(value: &str) -> Option<String> {
    let line = String::from(value);
    if line.starts_with("#") {
        return None;
    }

    let parts: Vec<&str> = line.split("\t").collect();

    if parts.len() < 9 {
        return None;
    }

    let gene = parse_attributes(parts[8]);

    if gene != "".to_string() {
        return Some(gene);
    } else {
        return None;
    }
}

fn main() {
    let file_path = "/home/felixd/Downloads/GCF_000001405.40_GRCh38.p14_genomic.gtf";
    let file_handle = std::fs::File::open(file_path).unwrap();

    let metadata = file_handle.metadata().unwrap();

    let gtf_size = metadata.len() as usize;

    let num_cpus = std::thread::available_parallelism().unwrap().get();

    let chunk_size = gtf_size / num_cpus;

    let mut gtf_reader = BufReader::new(file_handle);

    let mut missing_part = String::new();

    let mut join_handles = Vec::new();

    let genes = Arc::new(Mutex::new(HashSet::<String>::new()));

    

    for _i in 0..num_cpus {
        let mut chunk = vec![0; chunk_size];
        gtf_reader.read_exact(&mut chunk).unwrap();

        let cut_off = chunk.iter().rposition(|x| *x == b'\n').unwrap();

        let mut chunk = chunk.to_vec();

        let new_missing_part = String::from_utf8(chunk.split_off(cut_off)).unwrap();

        let mut chunk = String::from_utf8(chunk).unwrap();

        chunk.insert_str(0, &missing_part);

        let thread_genes = Arc::clone(&genes);

        let join_handle = thread::spawn(move || {
            let mut found_genes = HashSet::new();
            for line in chunk.split('\n') {
                if let Some(gene) = parse_gtf_line(&line) {
                    found_genes.insert(gene);
                }
            }

            let mut thread_genes = thread_genes.lock().unwrap();
            thread_genes.extend(found_genes);
        });
        join_handles.push(join_handle);

        missing_part = new_missing_part;
    }

    join_handles
        .into_iter()
        .for_each(|join_handle| join_handle.join().unwrap());

    println!(
        "{:?}",
        genes.lock().unwrap()
    );
}
