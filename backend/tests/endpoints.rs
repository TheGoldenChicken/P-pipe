use rocket::figment::{map, value::{Map, Value}};
use rocket::local::asynchronous::Client;
use rocket::http::Status;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use backend::endpoints::dispatcher::{rocket_from_config, Challenge, Transaction, transactions_from_challenge};


// TODO: Consider if there isn't a way to use Rocket::local::blocking::Client for testing endpoints...

// TODO: Move this async_client_From_pg_connect_options to something else... maybe helper functions or something? Probably a common.rs script or smth...
pub async fn async_client_from_pg_connect_options(
    pg_connect_options: PgConnectOptions,
) -> Client {
    // TODO: Consider if it is important to have such a 'comprehensive' format! here, or if we can do something like format!"postgres://postgres:postgres@localhost:5432/{}"
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

    // rocket_from_config is just helper function that returns rocket<Build> https://github.com/madoke/configmonkey/blob/main/src/app.rs
    let client = Client::tracked(rocket_from_config(figment))
        .await
        .expect("valid rocket instance");

    return client;
}

// TODO: perhaps remove this, we already test for it in the other functions
#[sqlx::test]
async fn challenge_post_sucess(
    _: PgPoolOptions,
    pg_connect_options: PgConnectOptions,
) -> sqlx::Result<()> {

    let client = async_client_from_pg_connect_options(pg_connect_options).await;
    let base_challenges = "/api/challenges";
    // TODO: Perhaps move this challenge to a function that returns it...
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

    let response = client.post(base_challenges)
    .json(&post)
    .dispatch()
    .await;

    assert_eq!(response.status(), Status::Ok, "Expected success status, got {:?}", response.status());

    Ok(())
}


#[sqlx::test]
async fn challenge_post_posts_something(
    _: PgPoolOptions,
    pg_connect_options: PgConnectOptions,
) -> sqlx::Result<()> {

    let pool = PgPoolOptions::new()
    .connect_with(pg_connect_options.clone())
    .await?;

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

    let response = client.post(base_challenges)
    .json(&post)
    .dispatch()
    .await;

    assert_eq!(response.status(), Status::Ok, "Expected success status, got {:?}", response.status());

    let response = response.into_json::<Vec<Challenge>>()
    .await
    .expect("Failed to deserialize Challenge response");
    
    let db_challenges: Vec<Challenge> = sqlx::query_as::<_, Challenge>("SELECT * FROM challenges")
    .fetch_all(&pool)
    .await?;
    
    if db_challenges.is_empty() {
        panic!("No challenges found in db after POST")
    }

    assert_eq!(db_challenges, response, "Expected challenge POST response to be the same as what was posted to db!");

    Ok(())
}


#[sqlx::test]
async fn challenge_post_posts_to_transactions(
    _: PgPoolOptions,
    pg_connect_options: PgConnectOptions,
) -> sqlx::Result<()> {

    let pool = PgPoolOptions::new()
    .connect_with(pg_connect_options.clone())
    .await?;

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


    let response = client.post(base_challenges)
    .json(&post)
    .dispatch()
    .await;

    assert_eq!(response.status(), Status::Ok, "Expected success status, got {:?}", response.status());

    let response = response.into_json::<Vec<Challenge>>()
    .await
    .expect("Failed to deserialize Challenge response")
    .get(0)
    .cloned()
    .expect("No challenge returned from POST!");

    let expected_transactions = transactions_from_challenge(response);

    let db_transactions: Vec<Transaction> = sqlx::query_as::<_, Transaction>("SELECT * FROM transactions")
    .fetch_all(&pool)
    .await?;
    
    if db_transactions.is_empty() {
        panic!("No transactions found in db after POST api/challenges")
    }

    assert_eq!(db_transactions, expected_transactions, "Expected db transactions after POST api/challenges to match expected transactions from transactions_from_challenge!");

    Ok(())
}