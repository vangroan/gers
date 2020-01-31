extern crate gers_core as core;
extern crate pyo3;

use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

#[pyfunction]
/// Formats the sum of two numbers as string
fn say_hello() -> PyResult<()> {
    core::say_hello();

    Ok(())
}

/// This module is a python module implemented in Rust.
#[pymodule]
fn gers(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(say_hello))?;

    Ok(())
}
