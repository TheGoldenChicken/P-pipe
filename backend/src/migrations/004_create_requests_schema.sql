CREATE TYPE request_status as ENUM ('pending', 'correct', 'partial_correct', 'incorrect', 'syntax_error', 'deadline_exceeded');

CREATE TABLE IF NOT EXISTS requests (
    id SERIAL PRIMARY KEY,
    challenge_id INTEGER NOT NULL REFERENCES challenges(id) ON DELETE CASCADE ON UPDATE CASCADE,
    created_at BIGINT DEFAULT (EXTRACT(EPOCH FROM now()) * 1000)::BIGINT,

    type_of_request jsonB NOT NULL,
    -- type_of_request request_type NOT NULL,
    -- request_payload jsonB NOT NULL,
    expected_response jsonB NOT NULL,
    deadline BIGINT
);

CREATE TABLE IF NOT EXISTS completed_requests (
    id SERIAL PRIMARY KEY,
    -- UUID GOES HERE, right now, ID is set dynamically, not necessarily reflecting the ID of the request a completed_request was made from...
    challenge_id INTEGER NOT NULL REFERENCES challenges(id) ON DELETE CASCADE ON UPDATE CASCADE,
    created_at BIGINT DEFAULT (EXTRACT(EPOCH FROM now()) * 1000)::BIGINT,
    
    type_of_request jsonB NOT NULL,
    expected_response jsonB NOT NULL,
    deadline BIGINT,

    request_status request_status NOT NULL,
    submitted_at BIGINT,
    submitted_response jsonB,
    judgement_message jsonB
);

-- Only used for requests, hence why they are not in 001_create_enums.sql
-- CREATE TYPE request_type as ENUM ('data_validation', 'batch_prediction', 'calculated_feature');
