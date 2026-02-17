use crate::schemas::challenge::{Challenge, ChallengeOptions};
use crate::schemas::common::{AccessBinding, DispatchTarget, DriveBinding, S3Binding};
use crate::schemas::request::{BatchPredictionPayload, RequestType};
use crate::schemas::transaction::Transaction;
use sqlx::types::Json;

pub fn accessbindings_instance() -> Vec<AccessBinding> {
    vec![
        AccessBinding::S3(S3Binding {
            identity: "ec2userstuff".to_string(),
            bucket: "somebucket".to_string(),
        }),
        AccessBinding::Drive(DriveBinding {
            identity: "dderpson99@gmail.com".to_string(),
            folder_id: Some("abcd123".to_string()),
            user_permissions: "Read".to_string(),
        }),
    ]
}

pub fn challenge_instance() -> Challenge {
    let access_bindings = accessbindings_instance();
    Challenge {
        id: Some(42),
        challenge_name: "testingchallenge1".into(),
        created_at: None,
        init_dataset_location: "py_modules/tests/test_data/iris.csv".into(),
        init_dataset_rows: 150,
        init_dataset_name: Some("dataset".into()),
        init_dataset_description: Some("desc".into()),
        dispatches_to: vec![DispatchTarget::S3],
        time_of_first_release: 1000,
        release_proportions: vec![0.3, 0.4, 0.3],
        time_between_releases: 60,
        access_bindings: Some(Json(access_bindings)),
        challenge_options: Json(ChallengeOptions::default()),
    }
}

pub fn minimal_challenge_instance() -> Challenge {
    Challenge {
        id: None,
        challenge_name: "testingchallenge1".into(),
        created_at: None,
        init_dataset_location: "py_modules/tests/test_data/iris.csv".into(),
        init_dataset_rows: 150,
        init_dataset_name: None,
        init_dataset_description: None,
        dispatches_to: vec![DispatchTarget::S3],
        time_of_first_release: 1000,
        release_proportions: vec![0.3, 0.4, 0.3],
        time_between_releases: 60,
        access_bindings: None,
        challenge_options: Json(ChallengeOptions::default()),
    }
}

pub fn challenge_instance_multiple_dispatch() -> Challenge {
    let access_bindings = accessbindings_instance();
    Challenge {
        id: Some(42),
        challenge_name: "testingchallenge1".into(),
        created_at: None,
        init_dataset_location: "py_modules/tests/test_data/iris.csv".into(),
        init_dataset_rows: 150,
        init_dataset_name: Some("dataset".into()),
        init_dataset_description: Some("desc".into()),
        dispatches_to: vec![DispatchTarget::S3],
        time_of_first_release: 1000,
        release_proportions: vec![0.3, 0.4, 0.3],
        time_between_releases: 60,
        access_bindings: Some(Json(access_bindings)),
        challenge_options: Json(ChallengeOptions::default()),
    }
}

pub fn transactions_expected_from_challenge_instance() -> Vec<Transaction> {
    vec![
        Transaction {
            id: None,
            challenge_id: 42,
            created_at: None,
            scheduled_time: 1000,
            source_data_location: Some("py_modules/tests/test_data/iris.csv".into()),
            dispatch_location: Some(DispatchTarget::S3),
            data_intended_location: "challenge_42_testingchallenge1".into(),
            data_intended_name: Some("release_0".into()),
            rows_to_push: Some(vec![0, 45]),
            access_bindings: Some(sqlx::types::Json(accessbindings_instance())),
            challenge_options: Json(ChallengeOptions::default()),
        },
        Transaction {
            id: None,
            challenge_id: 42,
            created_at: None,
            scheduled_time: 1060,
            source_data_location: Some("py_modules/tests/test_data/iris.csv".into()),
            dispatch_location: Some(DispatchTarget::S3),
            data_intended_location: "challenge_42_testingchallenge1".into(),
            data_intended_name: Some("release_1".into()),
            rows_to_push: Some(vec![45, 105]),
            access_bindings: Some(sqlx::types::Json(accessbindings_instance())),
            challenge_options: Json(ChallengeOptions::default()),
        },
        Transaction {
            id: None,
            challenge_id: 42,
            created_at: None,
            scheduled_time: 1120,
            source_data_location: Some("py_modules/tests/test_data/iris.csv".into()),
            dispatch_location: Some(DispatchTarget::S3),
            data_intended_location: "challenge_42_testingchallenge1".into(),
            data_intended_name: Some("release_2".into()),
            rows_to_push: Some(vec![105, 150]),
            access_bindings: Some(sqlx::types::Json(accessbindings_instance())),
            challenge_options: Json(ChallengeOptions::default()),
        },
    ]
}

pub fn transaction_instance() -> Transaction {
    Transaction {
        id: None,
        challenge_id: 42,
        created_at: None,
        scheduled_time: 1120,
        source_data_location: Some("py_modules/tests/test_data/iris.csv".into()),
        dispatch_location: Some(DispatchTarget::S3),
        data_intended_location: "challenge_42_testingchallenge1".into(),
        data_intended_name: Some("release_2".into()),
        rows_to_push: Some(vec![105, 150]),
        access_bindings: Some(sqlx::types::Json(accessbindings_instance())),
        challenge_options: Json(ChallengeOptions::default()),
    }
}

// TODO: This is a rather ugly instance, and I'd prefer not to use it
use serde_json::json;
use std::collections::HashMap;

pub fn batch_prediction_instance() -> RequestType {
    // Helper to build each row
    fn row(
        row: i32,
        sepal_length: f64,
        sepal_width: f64,
        petal_length: f64,
        petal_width: f64,
        variety: &str,
    ) -> HashMap<String, serde_json::Value> {
        let mut map = HashMap::new();
        map.insert("row".into(), json!(row));
        map.insert("sepal.length".into(), json!(sepal_length));
        map.insert("sepal.width".into(), json!(sepal_width));
        map.insert("petal.length".into(), json!(petal_length));
        map.insert("petal.width".into(), json!(petal_width));
        map.insert("variety".into(), json!(variety));
        map
    }

    let items = vec![
        row(5, 5.4, 3.9, 1.7, 0.4, "Setosa"),
        row(6, 4.6, 3.4, 1.4, 0.3, "Setosa"),
        row(7, 5.0, 3.4, 1.5, 0.2, "Setosa"),
    ];

    RequestType::BatchPrediction(BatchPredictionPayload {
        items,
        count: Some(3),
    })
}
