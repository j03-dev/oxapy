use std::process::Command;

use pyo3_stub_gen::Result;

fn main() -> Result<()> {
    let stub = oxapy::stub_info()?;
    stub.generate()?;
    Command::new("find")
        .args([
            ".",
            "-name",
            "*.pyi",
            "-type",
            "f",
            "-exec",
            "sed",
            "-E",
            "-i",
            "s/-> tuple\\[([A-Za-z0-9_]+), *[A-Za-z0-9_.]+]/-> \\1/",
            "{}",
            "+",
        ])
        .output()?;
    Ok(())
}
