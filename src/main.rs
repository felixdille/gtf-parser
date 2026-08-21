use std::time;

mod gene_extractor;

fn main() {
    let start = time::Instant::now();
    let file_path = "/home/felix/Downloads/GCF_000001405.40_GRCh38.p14_genomic.gtf";

    println!("{}", gene_extractor::extract_genes(file_path).len());
    println!("{:?}", time::Instant::now() - start);
}