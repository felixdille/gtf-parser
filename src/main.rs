use gene_extractor::extract_genes;

fn main() {
    let file_path = "/home/felixd/Downloads/GCF_000001405.40_GRCh38.p14_genomic.gtf";

    println!("{}", extract_genes(file_path));
}