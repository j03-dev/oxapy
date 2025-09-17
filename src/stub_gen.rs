use pyo3_stub_gen::Result;

fn main() -> Result<()> {
    let stub = oxapy::stub_info()?;
    // stub.python_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    stub.generate()?;
    Ok(())
}
