use std::{
    cmp::min,
    collections::{HashSet},
    io::{BufReader, Read},
    println,
    sync::{Arc, Mutex},
    thread::{self},
};

use pyo3::{prelude::*, types::PyFrozenSet};

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

#[pyfunction]
pub fn extract_genes(file_path: &str) -> HashSet<String> {
    let time_start = std::time::Instant::now();

    let file_handle = std::fs::File::open(file_path).unwrap();

    let metadata = file_handle.metadata().unwrap();

    let gtf_size = metadata.len() as usize;

    let num_cpus = std::thread::available_parallelism().unwrap().get();

    let chunk_size = gtf_size / num_cpus;

    let mut gtf_reader = BufReader::new(file_handle);

    let mut join_handles = Vec::with_capacity(num_cpus);

    let genes = Arc::new(Mutex::new(HashSet::<String>::new()));

    let mut file_contents = String::with_capacity(gtf_size);
    _ = gtf_reader.read_to_string(&mut file_contents);

    let file_contents = Arc::new(file_contents);

    let mut start = 0;
    let mut last = chunk_size;

    for _i in 0..num_cpus {
        let cut = if last >= gtf_size {
            gtf_size
        } else {
            file_contents.get(start..last).and_then(|s| s.rfind('\n')).unwrap() + start +1
        };

        let thread_genes = Arc::clone(&genes);

        let file_contents = file_contents.clone();
        let join_handle = thread::spawn(move || {
            
            let chunk = file_contents.get(start..cut).unwrap();
  
            let mut found_genes = HashSet::new();
            for line in chunk.split('\n') {
                if let Some(gene) = parse_gtf_line(&line) {
                    found_genes.insert(gene);
                }
            }

            let mut thread_genes = thread_genes.lock().unwrap();
            thread_genes.extend(found_genes);
        });

        start = cut;
        last += chunk_size;

        join_handles.push(join_handle);
    }

    join_handles
        .into_iter()
        .for_each(|join_handle| join_handle.join().unwrap());
    
    println!("{}", time_start.elapsed().as_secs());

    genes.lock().unwrap().clone()
}

#[pyfunction]
fn extract_genes_detached(py: Python<'_>, file_path: &str) -> HashSet<String> {
    py.detach(|| extract_genes(file_path))
}

#[pymodule]
fn gene_extractor(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(extract_genes_detached, m)?)?;

    Ok(())
}