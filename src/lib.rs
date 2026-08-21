mod gene_extractor;

use pyo3::{prelude::*};

#[pyfunction]
fn extract_genes_detached(py: Python<'_>, file_path: &str) -> Vec<String> {
    py.detach(|| gene_extractor::extract_genes(file_path))
}

#[pymodule]
fn get_genes(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(extract_genes_detached, m)?)?;

    Ok(())
}