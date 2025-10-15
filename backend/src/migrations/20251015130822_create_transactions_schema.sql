CREATE TYPE transaction_status_enum AS ENUM ('success', 'failed', 'success_stdout_present');

-- For transactions, we really want something to automatically generate names for different data parts like a uuid or something?
CREATE TABLE IF NOT EXISTS transactions (
    -- Bookkeeping fields
    id SERIAL PRIMARY KEY,
    challenge_id INTEGER NOT NULL REFERENCES challenges(id) ON DELETE CASCADE ON UPDATE CASCADE,
    created_at BIGINT DEFAULT EXTRACT(EPOCH FROM now()),
    
    -- Transaction info fields
    scheduled_time BIGINT NOT NULL,
    source_data_location TEXT,
    data_intended_location TEXT NOT NULL,
    
    rows_to_push INTEGER[],

    access_bindings jsonB
);

CREATE TABLE IF NOT EXISTS completed_transactions (
    -- Bookkeeping fields
    id SERIAL PRIMARY KEY,
    challenge_id INTEGER NOT NULL REFERENCES challenges(id) ON DELETE CASCADE ON UPDATE CASCADE,
    created_at BIGINT DEFAULT EXTRACT(EPOCH FROM now()),

    -- Transaction info fields
    scheduled_time BIGINT NOT NULL,
    source_data_location TEXT,
    data_intended_location TEXT NOT NULL,
    rows_to_push INTEGER[],

    access_bindings jsonB, 

    -- Status fields
    attempted_at BIGINT,
    transaction_status transaction_status_enum NOT NULL,
    stdout TEXT,
    stderr TEXT
);
