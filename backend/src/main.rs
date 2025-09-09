use pyo3::prelude::*;
use pyo3::ffi::c_str;

fn main() -> PyResult<()> {

    // Python::with_gil(|py| {
    //     // Add current working directory to sys.path
    //     let sys = py.import("sys")?;
    //     let cwd = std::env::current_dir()?.to_str().unwrap().to_string();
    //     sys.getattr("path")?.call_method1("insert", (0, cwd))?;

    //     // Import your script as a module
    //     let module = PyModule::import(py, "src.python_in_rust")?;

    //     // Call the main() function
    //     let result: String = module.getattr("main")?.call0()?.extract()?;
    //     println!("Python returned: {}", result);

    //     Ok(())
    // })
        Python::with_gil(|py| {
        let builtins = PyModule::import(py, "pandas").expect("error");
        let total: i32 = builtins
            .getattr("sum").expect("error")
            .call1((vec![1, 2, 3],)).expect("error")
            .extract().expect("error");
        assert_eq!(total, 6);
    });
    Python::with_gil(|py| {
    let sys = py.import("sys").expect("error");
    let executable: String = sys.getattr("executable").expect("error").extract().expect("error");
    println!("Using Python interpreter at: {}", executable);
});


    Python::with_gil(|py| {
        let activators = PyModule::from_code(
            py,
            c_str!(r#"
def main():
    import os
    import sys
    import pickle
    print(pickle.dumps({'test': 123}))
    sys.path.insert(0, os.getcwd())
    from dispatcher_module.dispatcher import simple_dispatcher
    from dispatcher_module.file_splitter import split_csv_by_proportions
    import click
    import pandas
    print(pandas.__file__)

    return "derp" # Right now, need to return a PyString, should change that...
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
