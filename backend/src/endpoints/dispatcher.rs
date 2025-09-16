// use rocket::serde::{ Deserialize, Serialize, json::Json };
// use rocket::{ State, response::status::Custom, http::Status };
// use tokio_postgres::Client;
// use postgres_types::{ ToSql, FromSql };

use std::process::Command;

// TODO: Find actually logical names for these...
#[post("/api/assignments/temp")] 
pub async  fn call_dispatcher() {

    // let output = Command::new("pytest").output().expect("failed to run pytest!");

    let output = Command::new("python") // or "python" depending on your system
        .args(&[
            "module/dispatcher.py",
            "--input_file", "tests/test_data/rust_splits/split_1.csv",
            "--output_dir", "tests/test_data/rust_dispatched",
            "--merge_files", "False",
        ])
        .output()
        .expect("Failed to execute Python script");

    println!("Status: {}", output.status);
    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
}

#[post("/api/assignments")] 
pub async fn call_file_splitter() {

    // let output = Command::new("pytest").output().expect("failed to run pytest!");

    let output = Command::new("python") // or "python" depending on your system
        .args(&["module/file_splitter.py",
                "--csv_file", "tests/test_data/iris.csv",
                "--proportions", "0.5,0.25,0.25",
                "--output_dir", "tests/test_data/rust_splits"]) // path to your Python script
        .output()
        .expect("Failed to execute Python script");

    println!("Status: {}", output.status);
    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
}
