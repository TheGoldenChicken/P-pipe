use rocket::fairing::AdHoc;
// use rocket::local::blocking::Client;
use rocket::local::asynchronous::Client;
use rocket::serde::{Serialize, Deserialize};
use rocket::http::Status;
use sqlx::test;
use sqlx::{PgPool, Row};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use backend::endpoints::dispatcher::{stage, test_stage, rocket_from_config};

#[derive(Serialize, Deserialize, Clone, sqlx::FromRow, Debug)]
pub struct Challenge {
    // "Bookkeeping fields"
    id: Option<i32>,
    name: String,
    created_at: Option<i64>,
    init_dataset_location: String,
    init_dataset_rows: i32,
    init_dataset_name: Option<String>,
    init_dataset_description: Option<String>,

    // Option fields
    time_of_first_release: i64,
    release_proportions: Vec<f64>,
    time_between_releases: i64,
}

    use rocket::{
        figment::{
            map,
            value::{Map, Value},
        },
        http::ContentType,
        local::asynchronous::{LocalResponse},
        serde::{
            json::{from_str, serde_json::json},
        },
        uri,
    };
// Challenges endpoints tests:

// Test create challenges
// Create challenge - then delete afterwards

// Test Get challenges
    // - Should yield empty when no users
    // - Create bunch of users
        // - Get should then return all created
        // - Their values, matching what was meant to be created

// Test delete
// Create challenge - then test
    // - Should remove challenge that was created
    // - Should remove all transactions as well
    // - Should only affect one row in challenges!


// Create users many times - then test
    // - Challenges should be empty
    // - Transactions should be empty
// #[test]
// fn test_delete() {
//     test("/", crate::dispatcher::stage())
// }

// #[test]
// fn test_destroy() {
//     test("/", crate::dispatcher::stage())
// }


// #[sqlx::test]
// async fn basic_test() -> sqlx::Result<()> {

//     // TODO: Creates test for DATABASE_URL being sat here, sqlx::test needs it!

//     let sqlx_stage = stage();
//     let rocket = rocket::build().attach(sqlx_stage);
//     let client = Client::untracked(rocket).await.expect("some error happened");

//     // let client = Client::untracked(rocket::build().attach(sqlx_stage)).unwrap();
    
//     let base_challenges = "/api/challenges";
//     let base_transactions = "/api/transactions";

//     assert_eq!(client.delete(base_challenges).dispatch().await.status(), Status::Ok);
//     // assert_eq!(client.get(base).dispatch().into_json::<Vec<i64>>(), Some(vec![]));
//     Ok(())
// }

pub async fn async_client_from_pg_connect_options(
    pg_connect_options: PgConnectOptions,
) -> Client {
    // let db_url = format!(
    //     "postgres://postgres:postgres@localhost:5432/{}",
    //     pg_connect_options.get_database().unwrap()
    // );
    println!("{:?}", pg_connect_options.get_database().clone().unwrap());

    let db_url = format!(
    "postgres://{}:{}@{}:{}/{}",
    pg_connect_options.get_username(),
    "postgres",
    pg_connect_options.get_host(),
    pg_connect_options.get_port(),
    pg_connect_options.get_database().unwrap()
    );

    let db_config: Map<_, Value> = map! {
        "url" => db_url.into(),
    };

    // Figment here is usually found automatically when calling rocket::build()
    // ... there it finds a figment with Config::figmen(), which finds it in 
    // ... one of three, based on an order, one of these is via DATABASE_URL https://rocket.rs/guide/master/configuration/
    let figment: rocket::figment::Figment = rocket::Config::figment()
        .merge(("databases", map!["postgres_db" => db_config]));

    println!("{:?}", figment);
    // rocket_from_config is just helper function that returns rocket<Build> https://github.com/madoke/configmonkey/blob/main/src/app.rs
    let client = Client::tracked(rocket_from_config(figment))
        .await
        .expect("valid rocket instance");

    println!("{:?}", client);

    return client;
}

#[sqlx::test]
async fn create_config_success(
    _: PgPoolOptions,
    pg_connect_options: PgConnectOptions,
) -> sqlx::Result<()> {
    println!("{:?}", pg_connect_options.clone());

    let client = async_client_from_pg_connect_options(pg_connect_options).await;
    let base_challenges = "/api/challenges";
    let post = Challenge {
        id: None,
        created_at: None,
        name: String::from("testing_challenge"),
        init_dataset_location: String::from("/home/cicero/ppipe/tests/test_data/iris.csv"),
        init_dataset_rows: 300,
        init_dataset_name: Some(String::from("iris")),
        init_dataset_description: Some(String::from("a .csv collection of flowers, classification task")),
        time_of_first_release: 5000,
        release_proportions: vec![0.50, 0.25],
        time_between_releases: 100
    };

    let response = client.post(base_challenges).json(&post).dispatch().await.into_json::<Challenge>();
    println!("{:?}", response.await);

    Ok(())
}



// use rocket::{delete, get, post, routes, Build, Rocket}; // Have to do this as long as src/lib.rs contains `pub mod endpoints;`, as it breaks #[macro_use]
// use rocket_db_pools::{Database, Connection};
// use sqlx::Arguments;

// #[derive(Database)]
// #[database("postgres_db")]
// // pub struct Db(sqlx::PgPool);

// #[sqlx::test]
// async fn pool_basic_test(pool: PgPool) -> sqlx::Result<()> {
//     // let mut conn = pool.acquire().await?;
//     // TODO: Creates test for DATABASE_URL being sat here, sqlx::test needs it!

//     // let rocket = rocket::build()
//     //     .manage(pool.clone())
//     //     .mount("/", routes![add_challenge, get_challenges, delete_challenge, destroy_challenges, get_transactions, delete_transaction]);

//     let sqlx_stage = test_stage(pool.clone());
//     let rocket = rocket::build().attach(sqlx_stage);
//     let client = Client::untracked(rocket).await.expect("some error happened");

//     // let client = Client::untracked(rocket::build().attach(sqlx_stage)).unwrap();
    
//     let base_challenges = "/api/challenges";
//     let base_transactions = "/api/transactions";

//     // assert_eq!(client.delete(base_challenges).dispatch().await.status(), Status::Ok);
//     let post = Challenge {
//         id: None,
//         created_at: None,
//         name: String::from("testing_challenge"),
//         init_dataset_location: String::from("/home/cicero/ppipe/tests/test_data/iris.csv"),
//         init_dataset_rows: 300,
//         init_dataset_name: Some(String::from("iris")),
//         init_dataset_description: Some(String::from("a .csv collection of flowers, classification task")),
//         time_of_first_release: 5000,
//         release_proportions: vec![0.50, 0.25],
//         time_between_releases: 100
//     };

//     let response = client.post(base_challenges).json(&post).dispatch().await.into_json::<Challenge>();

//     // assert_eq!(client.get(base).dispatch().into_json::<Vec<i64>>(), Some(vec![]));
//     Ok(())
// }


// #[sqlx::test]
// async fn my_test_case(
//   _pg_pool_options: PgPoolOptions,
//   pg_connect_options: PgConnectOptions,
// ) -> sqlx::Result<()> {
//   let client = async_client_from_pg_connect_options(pg_connect_options).await;
//   ...

// }
// fn test(base: &str, stage: AdHoc) {
//     // NOTE: If we had more than one test running concurrently that dispatches
//     // DB-accessing requests, we'd need transactions or to serialize all tests.
//     let client = Client::tracked(rocket::build().attach(stage)).unwrap();

//     // Clear everything from the database.
//     assert_eq!(client.delete(base).dispatch().status(), Status::Ok);
//     assert_eq!(client.get(base).dispatch().into_json::<Vec<i64>>(), Some(vec![]));

//     // // Add some random posts, ensure they're listable and readable.
//     // for i in 1..=N{
//     //     let title = format!("My Post - {}", i);
//     //     let text = format!("Once upon a time, at {}'o clock...", i);
//     //     let post = Post { title: title.clone(), text: text.clone() };

//     //     // Create a new post.
//     //     let response = client.post(base).json(&post).dispatch().into_json::<Post>();
//     //     assert_eq!(response.unwrap(), post);

//     //     // Ensure the index shows one more post.
//     //     let list = client.get(base).dispatch().into_json::<Vec<i64>>().unwrap();
//     //     assert_eq!(list.len(), i);

//     //     // The last in the index is the new one; ensure contents match.
//     //     let last = list.last().unwrap();
//     //     let response = client.get(format!("{}/{}", base, last)).dispatch();
//     //     assert_eq!(response.into_json::<Post>().unwrap(), post);
//     // }

//     // // Now delete all of the posts.
//     // for _ in 1..=N {
//     //     // Get a valid ID from the index.
//     //     let list = client.get(base).dispatch().into_json::<Vec<i64>>().unwrap();
//     //     let id = list.first().expect("have post");

//     //     // Delete that post.
//     //     let response = client.delete(format!("{}/{}", base, id)).dispatch();
//     //     assert_eq!(response.status(), Status::Ok);
//     // }

//     // // Ensure they're all gone.
//     // let list = client.get(base).dispatch().into_json::<Vec<i64>>().unwrap();
//     // assert!(list.is_empty());

//     // // Trying to delete should now 404.
//     // let response = client.delete(format!("{}/{}", base, 1)).dispatch();
//     // assert_eq!(response.status(), Status::NotFound);
// }

// #[test]
// fn test_sqlx() {
//     test("/", crate::dispatcher::stage())
// }

