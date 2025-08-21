// fn main() {
//     println!("Hello, world!");
// }

use pyo3::prelude::*;
use std::ffi::CString;
use pyo3::ffi::c_str;

fn main() -> PyResult<()> {
    
    Python::with_gil(|py| {
        let activators = PyModule::from_code(
            py,
            c_str!(r#"
def main():
    print("hello world from Python!")
    return "herpderp"
            "#),
            c_str!("hello_world.py"),
            c_str!("hello_world")
        ).expect("error");
        
        let result: String = activators.getattr("main").expect("erorr").call1(()).expect("erorr").extract().expect("erorr");

        println!("{}", result)
    });
    

    Python::with_gil(|py| {
        let builtins = PyModule::import(py, "builtins")?;
        let total: i32 = builtins
            .getattr("sum")?
            .call1((vec![1, 2, 3],))?
            .extract()?;
        assert_eq!(total, 6);
        Ok(())
    })
}



// use pyo3::prelude::*;
// use pyo3::types::PyModule;

// fn main() -> PyResult<()> {
//     Python::with_gil(|py| {
//         let sys = PyModule::import(py, "sys")?;
//         let version: String = sys.get("version")?.extract()?;
//         println!("Python version from Rust: {}", version);

//         let math = PyModule::import(py, "math")?;
//         let result: f64 = math.get("sqrt")?.call1((64.0,))?.extract()?;
//         println!("Square root of 64 is: {}", result);

//         Ok(())
//     })
// }
