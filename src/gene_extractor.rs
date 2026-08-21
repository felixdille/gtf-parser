use std::{
    cmp::min,
    collections::{HashSet},
    io::{BufReader, Read},
    sync::{Arc, Mutex},
    thread::{self},
};

fn parse_attributes(value: &str) -> &str {
    for attribute in value.split(";") {
        let attribute = attribute.trim();
        let pair: Vec<&str> = attribute.splitn(2, " ").collect();

        if pair.len() < 2 {
            continue;
        }

        let attribute_value: Vec<&str> = pair[1].split("\"").collect();
        let key = pair[0];

        if key == "gene_id" {
            return attribute_value[min(1, attribute_value.len() - 1)];
        }
    }
    ""
}

fn parse_gtf_line(value: &str) -> Option<&str> {
    let line = value;
    
    if line.starts_with("#") {
        return None;
    }

    let parts: Vec<&str> = line.split("\t").collect();

    if parts.len() < 9 {
        return None;
    }

    let gene = parse_attributes(parts[8]);

    if gene != "" {
        return Some(gene);
    } else {
        return None;
    }
}

pub fn extract_genes(file_path: &str) -> Vec<String> {
    let file_handle = std::fs::File::open(file_path).unwrap();

    let metadata = file_handle.metadata().unwrap();

    let gtf_size = metadata.len() as usize;

    let num_cpus = std::thread::available_parallelism().unwrap().get();

    let chunk_size = gtf_size / num_cpus;

    let mut gtf_reader = BufReader::new(file_handle);

    let mut join_handles = Vec::with_capacity(num_cpus);

    let genes = Arc::new(Mutex::new(HashSet::<String>::new()));

    let mut file_contents = Arc::new(String::with_capacity(gtf_size));
    _ = gtf_reader.read_to_string(Arc::make_mut(&mut file_contents));

    let mut start = 0;
    let mut last = chunk_size;

    for _i in 0..num_cpus {
        let cut = if last >= gtf_size {
            gtf_size
        } else {
            file_contents.get(start..last).and_then(|s| s.rfind('\n')).unwrap() + start +1
        };

        let thread_genes = Arc::clone(&genes);
        let thread_file_contents = Arc::clone(&file_contents);

        let join_handle = thread::spawn(move || {
            let chunk: &'_ str =  &thread_file_contents[start..cut];
  
            let mut found_genes = HashSet::new();
            for line in chunk.split('\n') {
                if let Some(gene) = parse_gtf_line(&line) {
                    found_genes.insert(gene);
                }
            }

            let mut thread_genes = thread_genes.lock().unwrap();
            thread_genes.extend(found_genes.iter().map(|x| String::from(x as &str)));
        });

        start = cut;
        last += chunk_size;

        join_handles.push(join_handle);
    }

    join_handles
        .into_iter()
        .for_each(|join_handle| join_handle.join().unwrap());

    genes.lock().unwrap().clone().into_iter().collect()
}