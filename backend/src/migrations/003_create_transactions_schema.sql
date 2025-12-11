-- For transactions, we really want something to automatically generate names for different data parts like a uuid or something?
CREATE TABLE IF NOT EXISTS transactions (
    -- Bookkeeping fields
    id SERIAL PRIMARY KEY,
    challenge_id INTEGER NOT NULL REFERENCES challenges(id) ON DELETE CASCADE ON UPDATE CASCADE,
    created_at BIGINT DEFAULT (EXTRACT(EPOCH FROM now()) * 1000)::BIGINT,

    -- Transaction info fields
    scheduled_time BIGINT NOT NULL,
    source_data_location TEXT,
    dispatch_location dispatch_target, -- If null, assume it is an access binding update
    data_intended_location TEXT NOT NULL, -- Folder (or similar structure) that we'll push the data to, always relevant no matter if we're simply updating access rights, creating folders (initialization), or if we're doing actual data uploads
    data_intended_name TEXT, -- Name of the single data slice that'll be in the folder
    rows_to_push INTEGER[],

    access_bindings jsonB,
    challenge_options jsonB NOT NULL
);

CREATE TABLE IF NOT EXISTS completed_transactions (
    -- Bookkeeping fields
    id SERIAL PRIMARY KEY,
    challenge_id INTEGER NOT NULL REFERENCES challenges(id) ON DELETE CASCADE ON UPDATE CASCADE,
    created_at BIGINT DEFAULT (EXTRACT(EPOCH FROM now()) * 1000)::BIGINT,
    
    -- Transaction info fields
    scheduled_time BIGINT NOT NULL,
    source_data_location TEXT,
    dispatch_location dispatch_target, -- If null, assume it is an access binding update
    data_intended_location TEXT, 
    data_intended_name TEXT,
    rows_to_push INTEGER[],

    access_bindings jsonB,
    challenge_options jsonB NOT NULL, 

    -- Status fields
    attempted_at BIGINT,
    transaction_status transaction_status_enum NOT NULL,
    stdout TEXT,
    stderr TEXT
);
