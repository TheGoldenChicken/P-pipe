use tokio_postgres::Client;

pub async fn create_tables(client: &Client) {
    let create_challenges_string = "
        CREATE TABLE IF NOT EXISTS challenges (
        -- Bookkeeping fields
            id SERIAL PRIMARY KEY,
            name TEXT NOT NULL,
            created_at BIGINT DEFAULT EXTRACT(EPOCH FROM now()),
            init_dataset_location TEXT NOT NULL,
            init_dataset_rows INTEGER NOT NULL,
            init_dataset_name TEXT,
            init_dataset_description TEXT,

            -- Option fields
            time_of_first_release BIGINT NOT NULL,
            release_proportions DOUBLE PRECISION[] NOT NULL,
            time_between_releases BIGINT NOT NULL -- Should be given in seconds
        );    
    ";

    let create_transactions_string = "
        CREATE TABLE IF NOT EXISTS transactions (
            -- Bookkeeping fields
            id SERIAL PRIMARY KEY,
            challenge_id INTEGER NOT NULL REFERENCES challenges(id) ON DELETE CASCADE ON UPDATE CASCADE,
            created_at BIGINT DEFAULT EXTRACT(EPOCH FROM now()),
            
            -- Transaction info fields
            -- Possibly add a new one like 'should_overwrite' BOOLEAN DEFAULT FALSE
            scheduled_time BIGINT NOT NULL,
            source_data_location TEXT NOT NULL,
            data_intended_location TEXT NOT NULL,
            -- Really would have wanted this to be of type INT4RANGE - that is more correct, but rust won't serialize easily. So we're stuck with jsonB instead...
            rows_to_push INTEGER[] NOT NULL
        );
    ";

    let create_completed_transactions_string = "
        CREATE TABLE IF NOT EXISTS completed_transactions (
            -- Bookkeeping fields
            id SERIAL PRIMARY KEY,
            challenge_id INTEGER NOT NULL REFERENCES challenges(id) ON DELETE CASCADE ON UPDATE CASCADE,
            created_at BIGINT DEFAULT EXTRACT(EPOCH FROM now()),

            -- Transaction info fields
            scheduled_time BIGINT NOT NULL,
            source_data_location TEXT NOT NULL,
            data_intended_location TEXT NOT NULL,
            rows_to_push INTEGER[] NOT NULL,

            -- Status fields
            attempted_at BIGINT,
            status TEXT, -- TODO: Should be replaced with an enum of 'failed', 'succeeded', 'etc'
            stdout TEXT,
            stderr TEXT
        );
    ";

    if let Err(e) = client.execute(create_challenges_string, &[]).await {
        eprintln!("Error creating challenges table: {}", e);
    }

        if let Err(e) = client.execute(create_transactions_string, &[]).await {
        eprintln!("Error creating transactions table: {}", e);
    }

        if let Err(e) = client.execute(create_completed_transactions_string, &[]).await {
        eprintln!("Error creating completed_transactions table: {}", e);
    }
}