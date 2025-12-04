-- Only used for requests, hence why they are not in 001_create_enums.sql
CREATE TYPE request_type as ENUM ('data_validation', 'batch_prediction', 'calculated_feature')
CREATE TYPE request_status as ENUM ('correct', 'partial_correct', 'incorrect', 'syntax_error', 'deadline_exceeded')

CREATE TABLE IF NOT EXISTS requests (
    id SERIAL PRIMARY KEY,
    challenge_id INTEGER NOT NULL REFERENCES challenges(id) ON DELETE CASCADE ON UPDATE CASCADE,
    created_at BIGINT DEFAULT EXTRACT(EPOCH FROM now()),

    type_of_request TEXT NOT NULL,
    request_payload jsonB NOT NULL,
    expected_response jsonB NOT NULL,
    deadline BIGINT
);

CREATE TABLE IF NOT EXISTS requests (
    id SERIAL PRIMARY KEY,
    challenge_id INTEGER NOT NULL REFERENCES challenges(id) ON DELETE CASCADE ON UPDATE CASCADE,
    created_at BIGINT DEFAULT EXTRACT(EPOCH FROM now()),
    
    type_of_request TEXT NOT NULL,
    request_payload jsonB NOT NULL,
    expected_response jsonB NOT NULL,
    deadline BIGINT,

    request_status ,
    submitted_at BIGINT
    submitted_response jsonB,

);