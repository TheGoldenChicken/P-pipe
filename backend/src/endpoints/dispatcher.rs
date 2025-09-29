use rocket::{delete, fairing, get, post, routes, Build, Rocket}; // Have to do this as long as src/lib.rs contains `pub mod endpoints;`, as it breaks #[macro_use]
use rocket::{fairing::AdHoc, figment::Figment, response::status::Custom, http::Status};
use rocket::serde::{Serialize, Deserialize, json::Json};

use rocket_db_pools::{Database, Connection};
use sqlx::Arguments;

// TODO: Consider if it makes sense to use a static migrator to avoid re-parsing the migrations each time
// static MIGRATOR: Migrator = sqlx::migrate!("backend/migrations");

// TODO: Consider if we should do away with the whole Custom<String> Type and instead just use tokio_postgres::Error?
// TODO: Consider if we should look into a way of having custom return messages... what if we want to show both challenges affected, AND transactions?

// TODO: Consider if it even makes sense to have this here, we use it in more places, but it is also so little code, sooooooo?
#[derive(Database)]
#[database("postgres_db")]
// #[database("postgres_tes")]
pub struct Db(sqlx::PgPool);

type Result<T, E = rocket::response::Debug<sqlx::Error>> = std::result::Result<T, E>;


#[derive(Serialize, Deserialize, Clone, sqlx::FromRow, Debug, PartialEq)]
pub struct Challenge {
    // "Bookkeeping fields"
    pub id: Option<i32>,
    pub name: String,
    pub created_at: Option<i64>,
    pub init_dataset_location: String,
    pub init_dataset_rows: i32,
    pub init_dataset_name: Option<String>,
    pub init_dataset_description: Option<String>,

    // Option fields
    pub time_of_first_release: i64,
    pub release_proportions: Vec<f64>,
    pub time_between_releases: i64,
}
#[derive(Serialize, Deserialize, Clone, Debug, sqlx::FromRow)]

pub struct Transaction {
    pub id: Option<i32>,
    pub challenge_id: i32,
    pub created_at: Option<i64>,
    pub scheduled_time: i64,
    pub source_data_location: String,
    pub data_intended_location: String,
    pub rows_to_push: Vec<i32>,
}
// Custom partialEq function so we can test if transactions from the db (with id and created_at) match those we expect to create (from transactions_from_challenge)
impl PartialEq for Transaction {
    fn eq(&self, other: &Self) -> bool {
        self.challenge_id == other.challenge_id &&
        self.scheduled_time == other.scheduled_time &&
        self.source_data_location == other.source_data_location &&
        self.data_intended_location == other.data_intended_location &&
        self.rows_to_push == other.rows_to_push
    }
}




#[derive(Serialize, Deserialize, Clone, Debug, sqlx::FromRow)]
struct CompletedTransaction {
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
async fn add_challenge(
    mut db: Connection<Db>,
    challenge: Json<Challenge> 
) -> Result<Json<Vec<Challenge>>, Custom<String>>  {
    
    // TODO: Add check here to see if release options makes sense:
    // - do release proportions sum to 1?
    // - Are any release proportions above 1?
    // - Is time of first release already passed?
    // - Is time between releases a fair number?

    // TODO; Check if we can do this with execute_query?
    let challenge = sqlx::query_as!(
        Challenge,
        r#"
        INSERT INTO challenges
        (name, init_dataset_location, init_dataset_rows, init_dataset_name, init_dataset_description, time_of_first_release, release_proportions, time_between_releases)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING *
        "#,
        challenge.name,
        challenge.init_dataset_location,
        challenge.init_dataset_rows,
        challenge.init_dataset_name,
        challenge.init_dataset_description,
        challenge.time_of_first_release,
        &challenge.release_proportions,
        challenge.time_between_releases
    )
    .fetch_one(&mut **db)
    .await.map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;
    
    // Generate transactions and add them to the DB
    let generated_transactions = transactions_from_challenge(challenge);
    add_transactions_into_db(&mut db, &generated_transactions).await?;

    get_challenges(db).await                            
}


async fn add_transactions_into_db(
    // db: &sqlx::PgPool,
    db: &mut Connection<Db>,
    transactions: &[Transaction],
) -> Result<u64, Custom<String>> {
    if transactions.is_empty() {
        return Ok(0);
    }

    // Build VALUES clause and parameter list
    let mut query = String::from("INSERT INTO transactions (challenge_id, scheduled_time, source_data_location, data_intended_location, rows_to_push) VALUES ");
    let mut args = sqlx::postgres::PgArguments::default();

    for (i, tx) in transactions.iter().enumerate() {
        if i > 0 {
            query.push_str(", ");
        }
        query.push_str(&format!(
            "(${}, ${}, ${}, ${}, ${})",
            i * 5 + 1,
            i * 5 + 2,
            i * 5 + 3,
            i * 5 + 4,
            i * 5 + 5
        ));

        args.add(tx.challenge_id);
        args.add(tx.scheduled_time);
        args.add(&tx.source_data_location);
        args.add(&tx.data_intended_location);
        args.add(&tx.rows_to_push);
    }

    // A shame here we can't use compile-time checked sql, since bulk insert and whatnot. A fucking shame
    // Also a fucking shame getting caught up on a single fucking deref * as well, fucking shoot me
    // There is no moral dilemmad, do you hop in front of the moving trolley? Yes!
    let affected = sqlx::query_with(&query, args)
        .execute(&mut ***db)
        .await
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?
        .rows_affected();

    Ok(affected)
}


// TODO: Make unit-test testing this for correct behavior!
pub fn transactions_from_challenge(challenge: Challenge) -> Vec<Transaction> {
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
        };
        transactions.push(transaction);
    }

    transactions
}


#[get("/api/challenges")]
async fn get_challenges(mut db: Connection<Db>) -> Result<Json<Vec<Challenge>>, Custom<String>> {
    let challenges = sqlx::query_as!(
        Challenge,
        "SELECT * FROM challenges"
    )
    .fetch_all(&mut **db)
    .await
    .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;

    Ok(Json(challenges))
}

#[delete("/api/challenges/<id>")]
async fn delete_challenge(mut db: Connection<Db>, id: i32) -> Result<Status, Custom<String>> {
    sqlx::query!(
        "DELETE FROM challenges WHERE id = $1",
        id
    )
    .execute(&mut **db)
    .await
    .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;

    Ok(Status::NoContent)
}

#[get("/api/transactions")]
async fn get_transactions(mut db: Connection<Db>) -> Result<Json<Vec<Transaction>>, Custom<String>> {
    let transactions = sqlx::query_as!(
        Transaction,
        "SELECT id, challenge_id, created_at, scheduled_time, source_data_location, data_intended_location, rows_to_push FROM transactions"
    )
    .fetch_all(&mut **db)
    .await
    .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;

    Ok(Json(transactions))
}

#[delete("/api/transactions/<id>")]
async fn delete_transaction(mut db: Connection<Db>, id: i32) -> Result<Status, Custom<String>> {
    sqlx::query!(
        "DELETE FROM transactions WHERE id = $1",
        id
    )
    .execute(&mut **db)
    .await
    .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;

    Ok(Status::NoContent)
}

#[delete("/api/challenges")]
async fn destroy_challenges(mut db: Connection<Db>) -> Result<()> {
    sqlx::query!("DELETE FROM challenges").execute(&mut **db).await?;

    Ok(())
}

#[delete("/api/transactions")]
async fn destroy_transactions(mut db: Connection<Db>) -> Result<()> {
    sqlx::query!("DELETE FROM transactions").execute(&mut **db).await?;

    Ok(())
}

async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    match Db::fetch(&rocket) {
        Some(db) => match sqlx::migrate!("src/migrations").run(&**db).await {
            Ok(_) => Ok(rocket),
            Err(e) => {
                eprintln!("Failed to initialize SQLx database: {}", e);
                Err(rocket)
            }
        },
        None => Err(rocket),
    }
}

// TODO: Find out if there is a way to return rocket::AdHoc so we can Rocket::build() later?
pub fn rocket_from_config(figment: Figment) -> Rocket<Build> {
    rocket::custom(figment)
        .attach(Db::init())
        .attach(AdHoc::try_on_ignite("SQLx Migrations", run_migrations))
        .mount("/", routes![
            add_challenge,
            get_challenges,
            delete_challenge,
            destroy_challenges,
            get_transactions,
            delete_transaction,
            destroy_transactions
        ])
}


#[cfg(test)]
mod tests {
    use proptest::prelude::{proptest, prop, prop_assert_eq};
    // TODO: Remove here, and only import specifcally what is asked for!
    use super::*;
    
    #[test]
    // TODO: Consider naming convention here, should we really call it basic(), edge_case, invalid_input, etc.?
    fn test_transactions_from_challenge_basic() {
        let challenge = Challenge {
            id: Some(42),
            name: "Test Challenge".into(),
            created_at: None,
            init_dataset_location: "s3://bucket/data.csv".into(),
            init_dataset_rows: 100,
            init_dataset_name: Some("dataset".into()),
            init_dataset_description: Some("desc".into()),
            time_of_first_release: 1000,
            release_proportions: vec![0.3, 0.4, 0.3],
            time_between_releases: 60,
        };

        let transactions = transactions_from_challenge(challenge.clone());

        assert_eq!(transactions.len(), 3, "Expected 3 transactions");

        let expected = vec![
            Transaction {
                id: None,
                challenge_id: 42,
                created_at: None,
                scheduled_time: 1000,
                source_data_location: challenge.init_dataset_location.clone(),
                data_intended_location: "release_0".into(),
                rows_to_push: vec![0, 30],
            },
            Transaction {
                id: None,
                challenge_id: 42,
                created_at: None,
                scheduled_time: 1060,
                source_data_location: challenge.init_dataset_location.clone(),
                data_intended_location: "release_1".into(),
                rows_to_push: vec![30, 70],
            },
            Transaction {
                id: None,
                challenge_id: 42,
                created_at: None,
                scheduled_time: 1120,
                source_data_location: challenge.init_dataset_location.clone(),
                data_intended_location: "release_2".into(),
                rows_to_push: vec![70, 100],
            },
        ];

        assert_eq!(transactions, expected, "Transaction output mismatch");
    }

    proptest! {
        #[test]
        fn total_rows_pushed_is_100(proportions in prop::collection::vec(0.0..1.0, 1..10)) {
            // In case of proportion of only 1, will create vector of normalized proportion of [1.0]!
            let total: f64 = proportions.iter().sum();
            let normalized: Vec<f64> = if total == 0.0 {
                vec![1.0] // fallback to avoid division by zero
            } else {
                proportions.iter().map(|p| p / total).collect()
            };

            let challenge = Challenge {
                id: Some(1),
                name: "test".into(),
                created_at: None,
                init_dataset_location: "s3://bucket/data.csv".into(),
                init_dataset_rows: 100,
                init_dataset_name: None,
                init_dataset_description: None,
                time_of_first_release: 0,
                release_proportions: normalized.clone(),
                time_between_releases: 1,
            };

            let transactions = transactions_from_challenge(challenge);

            let total_rows: i32 = transactions.iter()
                .map(|t| t.rows_to_push[1] - t.rows_to_push[0])
                .sum();
            prop_assert_eq!(total_rows, 100, "Expected total rows to be 100, got {}", total_rows);
        }
    }

}
