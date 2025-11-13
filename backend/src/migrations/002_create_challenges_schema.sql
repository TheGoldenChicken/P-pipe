CREATE TABLE IF NOT EXISTS challenges (
    -- Bookkeeping fields
    id SERIAL PRIMARY KEY,
    challenge_name TEXT NOT NULL,
    created_at BIGINT DEFAULT EXTRACT(EPOCH FROM now()),
    init_dataset_location TEXT NOT NULL,
    init_dataset_rows INTEGER NOT NULL,
    init_dataset_name TEXT,
    init_dataset_description TEXT,

    -- Option fields
    dispatches_to dispatch_target[] NOT NULL,
    time_of_first_release BIGINT NOT NULL,
    release_proportions DOUBLE PRECISION[] NOT NULL,
    time_between_releases BIGINT NOT NULL, -- Should be given in millis (utc format)

    access_bindings jsonB
);    

CREATE TABLE IF NOT EXISTS test_table (
    id SERIAL PRIMARY KEY,
    dispatches_to dispatch_target[] NOT NULL
);    

