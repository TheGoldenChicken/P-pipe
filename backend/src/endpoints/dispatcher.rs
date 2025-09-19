use rocket::execute;
use rocket::serde::{ Deserialize, Serialize, json::Json };
use rocket::{ State, response::status::Custom, http::Status };
use tokio_postgres::Client;
// use postgres_types::{ ToSql, FromSql };

use std::process::Command;

// TODO: To it possible to have a more a general "get n from database" function, consider implementing a FromRow trait for Challenge and all other relevant classes
#[derive(Serialize, Deserialize, Clone)]
pub struct Challenge {
    // "Bookkeeping fields"
    id: Option<i32>,
    name: String,
    created_at: Option<i64>,
    init_dataset_location: String,
    init_dataset_name: Option<String>,
    init_dataset_description: Option<String>,

    // Option fields
    time_of_first_release: i64,
    release_proportions: Vec<f64>,
    time_between_releases:i64,
}

// TODO: Consider if this function should even return all challenges when added, might be kinda bad...
#[post("/api/challenges", data = "<challenge>")]
pub async fn add_challenge(
    conn: &State<Client>,
    challenge: Json<Challenge> 
) -> Result<Json<Vec<Challenge>>, Custom<String>>  {
    
    execute_query(conn,
    "INSERT INTO challenges
    (name, init_dataset_location, init_dataset_name, init_dataset_description, time_of_first_release, release_proportions, time_between_releases)
    VALUES ($1, $2, $3, $4, $5, $6, $7)",
    &[&challenge.name, &challenge.init_dataset_location, &challenge.init_dataset_name, &challenge.init_dataset_description,
        &challenge.time_of_first_release, &challenge.release_proportions, &challenge.time_between_releases]
    ).await?;
    
    get_challenges(conn).await

    // TODO: Add logic for handling if there is already an existing query? Add logic to create transactions automatically
}

#[get("/api/challenges")]
pub async fn get_challenges(conn: &State<Client>) -> Result<Json<Vec<Challenge>>, Custom<String>> {
    get_challenges_from_db(conn).await.map(Json)
}

async fn get_challenges_from_db(client: &Client) -> Result<Vec<Challenge>, Custom<String>> {
    let challenges = client
    .query("SELECT * FROM challenges", &[]).await
    .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?
    .iter()
    .map(|row| Challenge { id: Some(row.get("id")),
                                name: row.get("name"),
                                created_at: row.get("created_at"),
                                init_dataset_location: row.get("init_dataset_location"),
                                init_dataset_name: Some(row.get("init_dataset_name")),
                                init_dataset_description: Some(row.get("init_dataset_description")),
                                time_of_first_release: row.get("time_of_first_release"),
                                release_proportions: row.get("release_proportions"),
                                time_between_releases: row.get("time_between_releases"),
                                })
    .collect::<Vec<Challenge>>();
    Ok(challenges)
}

#[delete("/api/challenges/<id>")]
pub async fn delete_challenge(conn: &State<Client>, id: i32) -> Result<Status, Custom<String>> {
    execute_query(conn, "DELETE FROM challenges WHERE id = $1", &[&id]).await?;
    Ok(Status::NoContent)
}


// // TODO: Find actually logical names for these...
// #[post("/api/assignments/temp")] 
// pub async  fn call_dispatcher() {

//     // let output = Command::new("pytest").output().expect("failed to run pytest!");

//     let output = Command::new("python") // or "python" depending on your system
//         .args(&[
//             "module/dispatcher.py",
//             "--input_file", "tests/test_data/rust_splits/split_1.csv",
//             "--output_dir", "tests/test_data/rust_dispatched",
//             "--merge_files", "False",
//         ])
//         .output()
//         .expect("Failed to execute Python script");

//     println!("Status: {}", output.status);
//     println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
//     println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
// }

// #[post("/api/assignments")] 
// pub async fn call_file_splitter() {

//     // let output = Command::new("pytest").output().expect("failed to run pytest!");

//     let output = Command::new("python") // or "python" depending on your system
//         .args(&["module/file_splitter.py",
//                 "--csv_file", "tests/test_data/iris.csv",
//                 "--proportions", "0.5,0.25,0.25",
//                 "--output_dir", "tests/test_data/rust_splits"]) // path to your Python script
//         .output()
//         .expect("Failed to execute Python script");

//     println!("Status: {}", output.status);
//     println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
//     println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
// }

async fn execute_query(
    client: &Client,
    query: &str,
    params: &[&(dyn tokio_postgres::types::ToSql + Sync)]
) -> Result<u64, Custom<String>> {
    client
        .execute(query, params).await
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))
}