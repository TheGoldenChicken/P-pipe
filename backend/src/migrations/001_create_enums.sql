CREATE TYPE transaction_status_enum AS ENUM ('success', 'failed', 'success_stdout_present');
CREATE TYPE dispatch_target AS ENUM ('s3', 'drive');
