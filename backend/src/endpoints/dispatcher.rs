use rocket::serde::{ Deserialize, Serialize, json::Json };
use rocket::{ State, response::status::Custom, http::Status };
use tokio_postgres::Client;

// TODO: Consider if we should do away with the whole Custom<String> Type and instead just use tokio_postgres::Error?
// TODO: Consider if we should look into a way of having custom return messages... what if we want to show both challenges affected, AND transactions?


// TODO: To it possible to have a more a general "get n from database" function, consider implementing a FromRow trait for Challenge and all other relevant classes
#[derive(Serialize, Deserialize, Clone)]
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
    time_between_releases:i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Transaction {
    // "Bookkeeping fields"
    id: Option<i32>,
    challenge_id: i32,
    created_at: Option<i64>,

    // Transaction info fields
    scheduled_time: i64,
    source_data_location: String,
    data_intended_location: String,
    // TODO: Add some sort of checking to ensure it is actually a 'range'? - Not enforced in any other way, I mean...
    rows_to_push: Vec<i32>,
    
    // Status fields - for completed transactions
    attempted_at: Option<i64>,
    status: Option<String>,
    stdout: Option<String>,
    stderr: Option<String>
}

// TODO: Consider if this function should even return all challenges when added, might be kinda bad...
#[post("/api/challenges", data = "<challenge>")]
pub async fn add_challenge(
    conn: &State<Client>,
    challenge: Json<Challenge> 
) -> Result<Json<Vec<Challenge>>, Custom<String>>  {
    
    // TODO: Add check here to see if release options makes sense:
    // - do release proportions sum to 1?
    // - Are any release proportions above 1?
    // - Is time of first release already passed?
    // - Is time between releases a fair number?

    // TODO; Check if we can do this with execute_query?
    let challenge = conn.query_one(
    "INSERT INTO challenges
    (name, init_dataset_location, init_dataset_rows, init_dataset_name, init_dataset_description, time_of_first_release, release_proportions, time_between_releases)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
    RETURNING *",
    &[&challenge.name, &challenge.init_dataset_location, &challenge.init_dataset_rows, &challenge.init_dataset_name, &challenge.init_dataset_description,
        &challenge.time_of_first_release, &challenge.release_proportions, &challenge.time_between_releases]
    ).await.map_err(|e| Custom(Status::InternalServerError, e.to_string()))
    .map(|row| Challenge { id: Some(row.get("id")),
                                name: row.get("name"),
                                created_at: row.get("created_at"),
                                init_dataset_location: row.get("init_dataset_location"),
                                init_dataset_rows: row.get("init_dataset_rows"),
                                init_dataset_name: row.get("init_dataset_name"),
                                init_dataset_description: row.get("init_dataset_description"),
                                time_of_first_release: row.get("time_of_first_release"),
                                release_proportions: row.get("release_proportions"),
                                time_between_releases: row.get("time_between_releases"),
                                })?;
    
    // Generate transactions and add them to the DB
    let generated_transactions = transactions_from_challenge(challenge);
    add_transactions_into_db(conn, generated_transactions).await?;

    get_challenges(conn).await                            
}

pub async fn add_transactions_into_db(
    conn: &Client,
    transactions: Vec<Transaction>,
) -> Result<u64, Custom<String>> {
    if transactions.is_empty() {
        return Ok(0);
    }

    // Build the VALUES clause with placeholders
    let mut query = String::from("INSERT INTO transactions (challenge_id, scheduled_time, source_data_location, data_intended_location, rows_to_push) VALUES ");
    let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();

    for (i, tx) in transactions.iter().enumerate() {
        let base = i * 5;
        query.push_str(&format!(
            "(${}, ${}, ${}, ${}, ${})",
            base + 1,
            base + 2,
            base + 3,
            base + 4,
            base + 5
        ));
        if i < transactions.len() - 1 {
            query.push_str(", ");
        }

        params.push(&tx.challenge_id);
        params.push(&tx.scheduled_time);
        params.push(&tx.source_data_location);
        params.push(&tx.data_intended_location);
        params.push(&tx.rows_to_push);
    }

    execute_query(conn, &query, &params).await
}


fn transactions_from_challenge(challenge: Challenge) -> Vec<Transaction> {
    let mut transactions = Vec::new();

    let mut running_proportion: f64 = 0.;

    for (i, release_proportion) in challenge.release_proportions.iter().enumerate() {
        let scheduled_time = challenge.time_of_first_release
            + challenge.time_between_releases * i as i64;

        // Old implementation, added all data points that should be included... may be useful still...
        // let rows_to_push_count = (release_proportion * challenge.init_dataset_rows as f64).round() as i32;
        // let rows_to_push = (0..rows_to_push_count).collect::<Vec<i32>>();   

        // TODO: Add check or fix here to ensure all rows are included - or at least, the total number of rows to push does not exceed the number of rows in dataset
        let rows_from = (running_proportion * challenge.init_dataset_rows as f64).round() as i32;
        let rows_to = ((running_proportion + release_proportion) * challenge.init_dataset_rows as f64).round() as i32;
        let rows_to_push = vec![rows_from, rows_to];
        running_proportion += release_proportion;
        

        let transaction = Transaction {
            id: None,
            challenge_id: challenge.id.unwrap_or_default(), // TODO: Here should fail or return error if none is present, or_default simply gets 0, which is an error...
            created_at: None,
            scheduled_time,
            source_data_location: challenge.init_dataset_location.clone(),
            data_intended_location: format!("release_{}", i),
            rows_to_push,
            attempted_at: None,
            status: None,
            stdout: None,
            stderr: None,
        };
        transactions.push(transaction);
    }

    transactions
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
                                init_dataset_rows: row.get("init_dataset_rows"),
                                init_dataset_name: row.get("init_dataset_name"),
                                init_dataset_description: row.get("init_dataset_description"),
                                time_of_first_release: row.get("time_of_first_release"),
                                release_proportions: row.get("release_proportions"),
                                time_between_releases: row.get("time_between_releases"),
                                })
    .collect::<Vec<Challenge>>();
    Ok(challenges)
}

// TODO: delete shouldn't return Status::NoContent, but u32, if we wanna standardize across the entire thing...
#[delete("/api/challenges/<id>")]
pub async fn delete_challenge(conn: &State<Client>, id: i32) -> Result<Status, Custom<String>> {
    execute_query(conn, "DELETE FROM challenges WHERE id = $1", &[&id]).await?;
    Ok(Status::NoContent)
}

#[get("/api/transactions")]
pub async fn get_transactions(conn: &State<Client>) -> Result<Json<Vec<Transaction>>, Custom<String>> {
    get_transactions_from_db(conn).await.map(Json)
}

// TODO: ADD THESE TO THE CORS, AND TEST THEM!

// TODO: Find out specifically why we don't need to call Some on status, attempted_at ,stdout, stderr, and so on...
async fn get_transactions_from_db(client: &Client) -> Result<Vec<Transaction>, Custom<String>> {
    let transactions = client
        .query("SELECT * FROM transactions", &[])
        .await
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?
        .iter()
        .map(|row| Transaction {
            id: Some(row.get("id")),
            challenge_id: row.get("challenge_id"),
            created_at: row.get("created_at"),
            scheduled_time: row.get("scheduled_time"),
            source_data_location: row.get("source_data_location"),
            data_intended_location: row.get("data_intended_location"),
            rows_to_push: row.get("rows_to_push"),
            attempted_at: row.get("attempted_at"),
            status: row.get("status"),
            stdout: row.get("stdout"),
            stderr: row.get("stderr"),
        })
        .collect::<Vec<Transaction>>();
    Ok(transactions)
}

pub async fn add_transation_into_db(
    conn: &State<Client>,
    transaction: Transaction
) -> Result<u64, Custom<String>> {
    execute_query(conn,
    "INSERT INTO transactions
    (challenge_id, scheduled_time, source_data_location, data_intended_location, rows_to_push)
    VALUES ($1, $2, $3, $4, $5)",
    &[&transaction.challenge_id, &transaction.scheduled_time, &transaction.source_data_location,
            &transaction.data_intended_location, &transaction.rows_to_push]
    ).await
}


#[delete("/api/transactions/<id>")]
pub async fn delete_transaction(conn: &State<Client>, id: i32) -> Result<Status, Custom<String>> {
    execute_query(conn, "DELETE FROM transactions WHERE id = $1", &[&id]).await?;
    Ok(Status::NoContent)
}



async fn execute_query(
    client: &Client,
    query: &str,
    params: &[&(dyn tokio_postgres::types::ToSql + Sync)]
) -> Result<u64, Custom<String>> {
    client
        .execute(query, params).await
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))
}