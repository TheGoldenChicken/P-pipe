use rand::seq::IndexedRandom;
use rocket::{delete, fairing, get, post, routes, Build, Rocket}; // Have to do this as long as src/lib.rs contains `pub mod endpoints;`, as it breaks #[macro_use]
use rocket::{fairing::AdHoc, figment::Figment, response::status::Custom, http::Status};
use rocket::serde::{Serialize, Deserialize, json::Json};

use rocket_db_pools::{Database, Connection};
use sqlx::Arguments; // Even though arguments appears unused, it is used in the background (macros perhaps?)
use sqlx::types::Json as DbJson;

use std::os::unix::process;
use std::process::Command;
use std::path::Path;
use std::env;
use tokio::time::{interval, Duration};
use chrono::Utc;

    use rand::Rng;


// TODO: Add check to challenges; check if no two identical dispatches_to locations

// TODO: Add some sort of checking to ensure if transactions.rows_to_push it is actually a 'range'? - Not enforced in any other way, I mean...
// ... can be done through making a custom constructor transactions::new() and having the function there...
// ... not good, since most creating those structs logic is done through implicit means
// ... transactions::new would never be called explicitly

// TODO: Consider if it makes sense to use a static migrator to avoid re-parsing the migrations each time
// static MIGRATOR: Migrator = sqlx::migrate!("backend/migrations");

// TODO: Consider if we should do away with the whole Custom<String> Type and instead just use tokio_postgres::Error?
// TODO: Consider if we should look into a way of having custom return messages... what if we want to show both challenges affected, AND transactions?

// TODO: Add checks to add_challenge to see if release options makes sense:
// - do release proportions sum to 1?
// - Are any release proportions above 1?
// - Is time of first release already passed?
// - Is time between releases a fair number?

// TODO: Consider if add_challenge should even return all challenges when added, might be kinda bad...

// TODO: Consider better ways of making add_transactions_into_db. It is VERY fragile right now...
// ... and doesn't respond at all well to changes in the db schema...

// TODO: Make unit-test testing transactions_from_challenge for correct behavior!

// TODO: Possibly make a wrapper struct around Challenges which is PostgresChallenge. This is to make AccessBindings be 
// For now, we can make due with writing this, whenever we need to use a Vec<AccessBinding>  access_bindings: c.access_bindings.map(|json| json.0)
// *Might* not make sense, tho. Since we have to pass AccessBindings back and forth to and from postgres, and at each point, it must be wrapped in sqlx::types::Json<>

// TODO: Consider if it even makes sense to have this here, we use it in more places, but it is also so little code, sooooooo?
#[derive(Database)]
#[database("postgres_db")]
pub struct Db(pub sqlx::PgPool);

type Result<T, E = rocket::response::Debug<sqlx::Error>> = std::result::Result<T, E>;

#[derive(sqlx::Type, Serialize, Deserialize, Debug, Clone, PartialEq)]
#[sqlx(type_name = "dispatch_target", rename_all = "snake_case")] // TODO: CHANGE TO SNAKE CASE, NOT LOWERCASE
pub enum DispatchTarget {
    S3,
    Drive,
}

use sqlx::postgres::{PgTypeInfo, PgHasArrayType};


impl PgHasArrayType for DispatchTarget {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("dispatch_target[]")
    }
}


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type")] // This tells serde to use the "type" field to determine the variant
pub enum AccessBinding {
    S3(S3Binding),
    Drive(DriveBinding),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct S3Binding {
    pub identity: String,
    pub bucket: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct DriveBinding {
    pub identity: String,
    pub folder_id: Option<String>,
    pub user_permissions: String, // TODO: Change this to be an enum of all roles in Drive
}

#[derive(sqlx::Type, Serialize, Deserialize, Debug, Clone, PartialEq)]
#[sqlx(type_name = "transaction_status_enum", rename_all = "snake_case")]
pub enum TransactionStatus {
    Success,
    SuccessWithStdout,
    Failed,
}


#[derive(Serialize, Deserialize, Clone, sqlx::FromRow, Debug, PartialEq)]
pub struct Challenge {
    // "Bookkeeping fields"
    pub id: Option<i32>,
    pub challenge_name: String,
    pub created_at: Option<i64>,
    pub init_dataset_location: String,
    pub init_dataset_rows: i32,
    pub init_dataset_name: Option<String>,
    pub init_dataset_description: Option<String>,

    // Option fields\JsonDb
    pub dispatches_to: Vec<DispatchTarget>,
    pub time_of_first_release: i64,
    pub release_proportions: Vec<f64>,
    pub time_between_releases: i64, 
    
    pub access_bindings: Option<DbJson<Vec<AccessBinding>>>,
}


#[derive(Serialize, Deserialize, Clone, Debug, sqlx::FromRow)]
pub struct Transaction {
    // "Bookkeeping fields"
    pub id: Option<i32>,
    pub challenge_id: i32,
    pub created_at: Option<i64>,

    // Transaction info fields
    pub scheduled_time: i64,
    pub source_data_location: Option<String>,
    pub dispatch_location: Option<DispatchTarget>,
    pub data_intended_location: String,
    pub data_intended_name: Option<String>,
    pub rows_to_push: Option<Vec<i32>>,

    pub access_bindings: Option<DbJson<Vec<AccessBinding>>>,
}

// Custom PartialEq function so we can test if transactions from the db (with id and created_at) match those we expect to create (from transactions_from_challenge)
impl PartialEq for Transaction {
    fn eq(&self, other: &Self) -> bool {
        self.challenge_id == other.challenge_id &&
        self.scheduled_time == other.scheduled_time &&
        self.source_data_location == other.source_data_location &&
        self.data_intended_location == other.data_intended_location &&
        self.rows_to_push == other.rows_to_push &&
        self.access_bindings == other.access_bindings
    }
}


#[derive(Serialize, Deserialize, Clone, Debug, sqlx::FromRow)]
pub struct CompletedTransaction {
    id: Option<i32>,
    challenge_id: i32,
    created_at: Option<i64>,

    scheduled_time: i64,
    source_data_location: Option<String>,
    dispatch_location: Option<DispatchTarget>,
    data_intended_location: String,
    data_intended_name: Option<String>,
    rows_to_push: Option<Vec<i32>>,

    access_bindings: Option<DbJson<Vec<AccessBinding>>>,

    // Status fields - for completed transactions
    attempted_at: Option<i64>,
    transaction_status: TransactionStatus,
    stdout: Option<String>,
    stderr: Option<String>,
}

impl CompletedTransaction {
    pub fn from_transaction(
        tx: Transaction,
        attempted_at: Option<i64>,
        transaction_status: TransactionStatus,
        stdout: Option<String>,
        stderr: Option<String>,
    ) -> Self {
        CompletedTransaction {
            id: tx.id,
            challenge_id: tx.challenge_id,
            created_at: tx.created_at,
            scheduled_time: tx.scheduled_time,
            source_data_location: tx.source_data_location,
            dispatch_location: tx.dispatch_location,
            data_intended_location: tx.data_intended_location,
            data_intended_name: tx.data_intended_name,
            rows_to_push: tx.rows_to_push,
            access_bindings: tx.access_bindings,
            attempted_at,
            transaction_status,
            stdout,
            stderr,
        }
    }
}



#[post("/api/challenges", data = "<challenge>")]
async fn add_challenge(
    mut db: Connection<Db>,
    challenge: Json<Challenge> 
) -> Result<Json<Vec<Challenge>>, Custom<String>>  {
    
    // TODO; Check if we can do this with execute_query?
    let challenge = sqlx::query_as!(
        Challenge,
        r#"
        INSERT INTO challenges
        (challenge_name, init_dataset_location, init_dataset_rows, init_dataset_name,
        init_dataset_description, dispatches_to, time_of_first_release, release_proportions, time_between_releases, access_bindings)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING
            id,
            challenge_name,
            created_at,
            init_dataset_location,
            init_dataset_rows,
            init_dataset_name,
            init_dataset_description,
            dispatches_to as "dispatches_to: Vec<DispatchTarget>",
            time_of_first_release,
            release_proportions,
            time_between_releases,
            access_bindings as "access_bindings: DbJson<Vec<AccessBinding>>"
        "#,
        challenge.challenge_name,
        challenge.init_dataset_location,
        challenge.init_dataset_rows,
        challenge.init_dataset_name,
        challenge.init_dataset_description,
        challenge.dispatches_to as _,
        challenge.time_of_first_release,
        &challenge.release_proportions,
        challenge.time_between_releases,
        challenge.access_bindings as _
    )
    .fetch_one(&mut **db)
    .await.map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;
    
    // Generate transactions and add them to the DB
    let generated_transactions = transactions_from_challenge(challenge)?;
    add_transactions_into_db(&mut db, &generated_transactions).await?;

    get_challenges(db).await                            
}


// TODO IMPORTANT: Really have a good dig into this one, will fail regularly if we don't find a better way of structuring it, and we'll have no idea why it fails...
async fn add_transactions_into_db(
    db: &mut Connection<Db>,
    transactions: &[Transaction],
) -> Result<u64, Custom<String>> {
    if transactions.is_empty() {
        return Ok(0);
    }

    let mut query = String::from(
        "INSERT INTO transactions (
            challenge_id,
            scheduled_time,
            source_data_location,
            data_intended_location,
            data_intended_name,
            rows_to_push,
            dispatch_location,
            access_bindings
        ) VALUES ",
    );

    let mut args = sqlx::postgres::PgArguments::default();

    for (i, tx) in transactions.iter().enumerate() {
        if i > 0 {
            query.push_str(", ");
        }
        
        let base = i * 8;
        query.push_str(&format!(
            "(${}, ${}, ${}, ${}, ${}, ${}, ${}, ${})",
            base + 1,
            base + 2,
            base + 3,
            base + 4,
            base + 5,
            base + 6,
            base + 7,
            base + 8
        ));

        args.add(tx.challenge_id);
        args.add(tx.scheduled_time);
        args.add(&tx.source_data_location);
        args.add(&tx.data_intended_location);
        args.add(&tx.data_intended_name);
        args.add(&tx.rows_to_push);
        args.add(&tx.dispatch_location);
        args.add(&tx.access_bindings);
    }

    let affected = sqlx::query_with(&query, args)
        .execute(&mut ***db)
        .await
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?
        .rows_affected();

    Ok(affected)
}


pub fn transactions_from_challenge(challenge: Challenge) -> Result<Vec<Transaction>, Custom<String>> {
    let mut transactions = Vec::new();

    let mut running_proportion: f64 = 0.;

    for (i, release_proportion) in challenge.release_proportions.iter().enumerate() {
        let scheduled_time = challenge.time_of_first_release
            + challenge.time_between_releases * i as i64;
        
        // Old implementation, added all data points that should be included... may be useful still...
        // let rows_to_push_count = (release_proportion * challenge.init_dataset_rows as f64).round() as i32;
        // let rows_to_push = (0..rows_to_push_count).collect::<Vec<i32>>();   

        // TODO: Consider option to have each portion randomly split between dispatch_locations...
        let rows_from = (running_proportion * challenge.init_dataset_rows as f64).round() as i32;
        let rows_to = ((running_proportion + release_proportion) * challenge.init_dataset_rows as f64).round() as i32;
        let rows_to_push = vec![rows_from, rows_to];
        running_proportion += release_proportion;
        
        // Returns error here if challenge id does not exist.
        let challenge_id = challenge.id.ok_or_else(|| {
            Custom(Status::BadRequest, "Missing challenge ID".to_string())
        })?;

        
        // TODO: Consider if this is safe behavior... can it panic?
        // Random slice of Dispatches to
        let mut rng: rand::prelude::ThreadRng = rand::rng();
        let n = rng.random_range(1..=challenge.dispatches_to.len());
        let dispatch_locations = challenge.dispatches_to
            .choose_multiple(&mut rng, n); 

        // TODO: We can avoid unecessary cloning by using shuffling with .drain(..n)
        for item in dispatch_locations.cloned() {
            let transaction = Transaction {
                id: None,
                challenge_id: challenge_id,
                created_at: None,
                scheduled_time,
                source_data_location: Some(challenge.init_dataset_location.clone()),
                dispatch_location: Some(item),
                data_intended_location: format!("challenge_{}_{}", challenge_id, challenge.challenge_name),
                data_intended_name: Some(format!("release_{}", i)),
                rows_to_push: Some(rows_to_push.clone()),
                access_bindings: challenge.access_bindings.clone()
            };
            transactions.push(transaction);
        }
    }

    Ok(transactions)
}


#[get("/api/challenges")]
async fn get_challenges(mut db: Connection<Db>) -> Result<Json<Vec<Challenge>>, Custom<String>> {
    let challenges = sqlx::query_as!(
        Challenge,
        r#"
        SELECT
            id,
            challenge_name,
            created_at,
            init_dataset_location,
            init_dataset_rows,
            init_dataset_name,
            init_dataset_description,
            dispatches_to as "dispatches_to: Vec<DispatchTarget>",
            time_of_first_release,
            release_proportions,
            time_between_releases,
            access_bindings as "access_bindings: DbJson<Vec<AccessBinding>>"
        FROM
            challenges;
        "#
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

#[delete("/api/challenges")]
async fn destroy_challenges(mut db: Connection<Db>) -> Result<()> {
    sqlx::query!("DELETE FROM challenges").execute(&mut **db).await?;

    Ok(())
}

#[get("/api/transactions")]
async fn get_transactions(mut db: Connection<Db>) -> Result<Json<Vec<Transaction>>, Custom<String>> {
    let transactions = sqlx::query_as!(
        Transaction,
        r#"
        SELECT
            id,
            challenge_id,
            created_at,
            scheduled_time,
            source_data_location,
            dispatch_location as "dispatch_location: DispatchTarget",
            data_intended_location,
            data_intended_name,
            rows_to_push,
            access_bindings as "access_bindings: DbJson<Vec<AccessBinding>>"
        FROM 
            transactions
        "#
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

#[delete("/api/transactions")]
async fn destroy_transactions(mut db: Connection<Db>) -> Result<()> {
    sqlx::query!("DELETE FROM transactions").execute(&mut **db).await?;
    Ok(())
}

async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    let should_migrate = env::var("RUN_MIGRATIONS")
        .map(|v| v == "true")
        .unwrap_or(false);

    if !should_migrate {
        eprintln!("🔧 Skipping migrations due to config");
        return Ok(rocket);
    }
    
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

async fn process_transaction(tx: &Transaction) -> Result<std::process::Output, Custom<String>> {
    // TODO: Move python_path and script_path to env variables
    let python_path = Path::new("../.venv/bin/python");
    let script_path = Path::new("../py_modules/orchestrator.py");

    println!("Processing transaction!");
    // TODO: Remove expect here - Should have proper error handling
    let transaction_string = serde_json::to_string(&tx)
    .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;

    let output = Command::new(python_path)
        .arg(script_path)
        .arg("orchestrator-cli")
        .arg("--transaction")
        .arg(transaction_string)
        .output();

    // TODO: Ask Hans if there isn't a better way of doing this...
    match output {
        Ok(result) => {
            Ok(result)
        }
        Err(e) => {
            Err(Custom(Status::InternalServerError, format!("Failed calling Python script with error {:?}", e)))
        }
    }
}

pub async fn transaction_scheduler(pool: sqlx::PgPool) -> Result<(), Custom<String>> {
    let mut ticker = interval(Duration::from_secs(5));
    loop {
        ticker.tick().await;
        let now = Utc::now().timestamp_millis();

        // TODO: Move all transactions that are affected to another table - completed transactions or something.
        let transactions = sqlx::query_as!(
            Transaction,
            r#"
            DELETE FROM
                transactions
            WHERE
                scheduled_Time <= $1
            RETURNING
                id,
                challenge_id,
                created_at,
                scheduled_time,
                source_data_location,
                dispatch_location as "dispatch_location: DispatchTarget",
                data_intended_location,
                data_intended_name,
                rows_to_push,
                access_bindings as "access_bindings: DbJson<Vec<AccessBinding>>"
            "#,
            now
        )
        .fetch_all(&pool)
        .await
        .map_err(|e| Custom(Status::InternalServerError, format!("Scheduler to grab transactions from database, {}", e)))?;

        for tx in transactions {
            let process_output = process_transaction(&tx).await?;
            

            let tx_status = match process_output.status.success() {
                true => TransactionStatus::Success,
                false => TransactionStatus::Failed,
            };

            let completed_tx = CompletedTransaction::from_transaction(tx,
            Some(now),
                tx_status,
                Some(String::from_utf8(process_output.stdout).expect("Could not convert py stdout utf8 string!")),
                Some(String::from_utf8(process_output.stderr).expect("Could not convert py stderr utf8 string!")),
            );

            insert_completed_transaction(&pool, &completed_tx).await
            .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;
        }
    }
}

pub async fn insert_completed_transaction(
    pool: &sqlx::PgPool,
    tx: &CompletedTransaction,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO completed_transactions (
            challenge_id,
            created_at,
            scheduled_time,
            source_data_location,
            dispatch_location,
            data_intended_location,
            data_intended_name,
            rows_to_push,
            access_bindings,
            attempted_at,
            transaction_status,
            stdout,
            stderr
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13
        )
        "#,
        tx.challenge_id,
        tx.created_at,
        tx.scheduled_time,
        tx.source_data_location,
        tx.dispatch_location as _,
        tx.data_intended_location,
        tx.data_intended_name,
        tx.rows_to_push.as_deref(),
        tx.access_bindings as _,
        tx.attempted_at,
        tx.transaction_status.clone() as TransactionStatus,
        tx.stdout,
        tx.stderr
    )
    .execute(pool)
    .await?;

    Ok(())
}


fn scheduler_fairing() -> AdHoc {
    AdHoc::on_ignite("Transaction Scheduler", |rocket| async {
        // We don't use Db<PgPool> here, since connection is only used inside of a request guard
        let db = rocket.state::<Db>().expect("Db not initialized");
        let pool = db.0.clone();
        tokio::spawn(async move {
        if let Err(e) = transaction_scheduler(pool.clone()).await {
            eprintln!("Scheduled task failed: {:?}", e);
        }
    });
        rocket
    })
}

pub fn rocket_from_config(figment: Figment) -> Rocket<Build> {
    let rocket_build = rocket::custom(figment)
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
        ]);

    let attach_scheduler = env::var("ATTACH_SCHEDULER")
        .map(|v| v == "true")
        .unwrap_or(false);

    if attach_scheduler {
        println!("Attaching scheduler fairing");
        rocket_build.attach(scheduler_fairing())
    }

    else {
        eprintln!("ATTACH_SCHEDULER either false or not set, no scheduler fairing attached");
        rocket_build
    }
}


// #[cfg(test)]
// mod tests {
//     use proptest::prelude::{proptest, prop, prop_assert_eq};
//     // TODO: Remove here, and only import specifcally what is asked for!
//     use super::*;
    
//     #[test]
//     // TODO: Consider naming convention here, should we really call it basic(), edge_case, invalid_input, etc.?
//     fn test_transactions_from_challenge_basic() {

//         let access_bindings = vec![
//             AccessBinding::S3(S3Binding { identity: "ec2userstuff".to_string(), bucket: "somebucket".to_string() }),
//             AccessBinding::Drive(DriveBinding { identity: "dderpson99@gmail.com".to_string(), folder_id: Some("abcd123".to_string()), user_permissions: "Read".to_string()})
//         ];

//         let challenge = Challenge {
//             id: Some(42),
//             challenge_name: "testingchallenge1".into(),
//             created_at: None,
//             init_dataset_location: "s3://bucket/data.csv".into(),
//             init_dataset_rows: 300,
//             init_dataset_name: Some("dataset".into()),
//             init_dataset_description: Some("desc".into()),
//             dispatches_to: vec![DispatchTarget::S3, DispatchTarget::Drive],
//             time_of_first_release: 1000,
//             release_proportions: vec![0.3, 0.4, 0.3],
//             time_between_releases: 60,
//             access_bindings: Some(sqlx::types::Json(access_bindings))
//         };

//         let transactions = transactions_from_challenge(challenge.clone()).expect("Could not generate transactions from challenge!");
        
//         assert_eq!(transactions.len(), 3, "Expected 3 transactions");

//         let expected = vec![
//             Transaction {
//                 id: None,
//                 challenge_id: 42,
//                 created_at: None,
//                 scheduled_time: 1000,
//                 source_data_location: challenge.init_dataset_location.clone(),
//                 dispatch_location: 
//                 data_intended_location: "release_0".into(),
//                 rows_to_push: Some(vec![0, 30]),
//                 access_bindings: Some(sqlx::types::Json(access_bindings))
//             },
//             Transaction {
//                 id: None,
//                 challenge_id: 42,
//                 created_at: None,
//                 scheduled_time: 1060,
//                 source_data_location: challenge.init_dataset_location.clone(),
//                 data_intended_location: "release_1".into(),
//                 rows_to_push: Some(vec![30, 70]),
//                 access_bindings: Some(sqlx::types::Json(access_bindings))
//             },
//             Transaction {
//                 id: None,
//                 challenge_id: 42,
//                 created_at: None,
//                 scheduled_time: 1120,
//                 source_data_location: challenge.init_dataset_location.clone(),
//                 data_intended_location: "release_2".into(),
//                 rows_to_push: Some(vec![70, 100]),
//                 access_bindings: Some(sqlx::types::Json(access_bindings))
//             },
//         ];

//         assert_eq!(transactions, expected, "Transaction output mismatch");
//     }

//     proptest! {
//         #[test]
//         fn total_rows_pushed_is_100(proportions in prop::collection::vec(0.0..1.0, 1..10)) {
//             // In case of proportion of only 1, will create vector of normalized proportion of [1.0]!
//             let total: f64 = proportions.iter().sum();
//             let normalized: Vec<f64> = if total == 0.0 {
//                 vec![1.0] // fallback to avoid division by zero
//             } else {
//                 proportions.iter().map(|p| p / total).collect()
//             };

//             let challenge = Challenge {
//                 id: Some(1),
//                 name: "test".into(),
//                 created_at: None,
//                 init_dataset_location: "s3://bucket/data.csv".into(),
//                 init_dataset_rows: 100,
//                 init_dataset_name: None,
//                 init_dataset_description: None,
//                 time_of_first_release: 0,
//                 release_proportions: normalized.clone(),
//                 time_between_releases: 1,
//             };

//             let transactions = transactions_from_challenge(challenge).expect("Could not generate transactions from challenge");

//             let total_rows: i32 = transactions.iter()
//                 .map(|t| t.rows_to_push[1] - t.rows_to_push[0])
//                 .sum();
//             prop_assert_eq!(total_rows, 100, "Expected total rows to be 100, got {}", total_rows);
//         }
//     }

// }
