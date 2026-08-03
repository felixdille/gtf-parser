use std::{cmp::min, collections::HashMap, io::{self, BufRead}, str::FromStr, thread};

use crate::Strand::{MINUS, PLUS};

#[derive(Debug)]
enum Strand {
    PLUS,
    MINUS
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
    attribute: HashMap<String, String>
}

fn get_strand(value: &str) -> Strand {
    if value == "+" {
        PLUS
    } else {
        MINUS
    }
}

fn parse_attributes(value: &str) -> HashMap<String, String> {
    let mut key_to_value: HashMap<String, String> = HashMap::new();
    value.split(";").for_each(|attribute| {
        let attribute = attribute.trim();
        let pair: Vec<&str> = attribute.splitn(2," ").collect();

        if pair.len() < 2 {
            return;
        } 

        let attribute_value: Vec<&str> = pair[1].split("\"").collect();

        key_to_value.insert(String::from(pair[0]), String::from(attribute_value[min(1, attribute_value.len()-1)]));
    });
    key_to_value
}

fn get_value<T>(value: &str) -> Option<T>
where T: FromStr {
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

fn main() {
    let file_path = "/home/felixd/Downloads/GCF_000001405.40_GRCh38.p14_genomic.gtf";
    let file_handle = std::fs::File::open(file_path).unwrap();
    let buf = io::BufReader::new(file_handle);

    let mut counter = 0;

    let entries: Vec<Option<GTFEntry>> = buf.lines().map(move |line| {
        counter+=1;
        if let Ok(line) = line {
            let line = String::from(line);
            if line.starts_with("#"){
                return None;
            }
            
            let parts: Vec<&str> = line.split("\t").collect();
          
            let entry = GTFEntry {
                seq_id: String::from(parts[0]),
                source: String::from(parts[1]),
                feature: String::from(parts[2]),
                start: parts[3].parse().unwrap(),
                end: parts[4].parse().unwrap(),
                score: get_value(parts[5]),
                strand: get_strand(parts[6]),
                frame: get_value(parts[7]),
                attribute: parse_attributes(parts[8])
            };
            Some(entry)
        } else {
            panic!("Not a valid GTF file");
        }
    }).collect();

    println!("Hello, world!");
}
