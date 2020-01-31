extern crate gers_core as core;
extern crate pyo3;

use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

#[pyfunction]
fn create_window() -> PyResult<()> {
    core::create_window();

    Ok(())
}

#[pyfunction]
fn run_loop() -> PyResult<()> {
    Ok(())
}

/// Formats the sum of two numbers as string
#[pyfunction]
fn say_hello() -> PyResult<()> {
    core::say_hello();

    Ok(())
}

/// This module is a python module implemented in Rust.
#[pymodule]
fn gers(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(say_hello))?;
    m.add_wrapped(wrap_pyfunction!(create_window))?;

    Ok(())
}
