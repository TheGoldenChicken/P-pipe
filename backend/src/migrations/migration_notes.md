-- TODO: Add some kind of validation check to see if the completed_transactions table follows transactions
-- ... so is completed_transactions a subset of transactions?

-- locations decided as:
-- challenges:
    -- dispatches_to: enum (s3, drive)
    -- if drive:
        -- create folder with name id_challenge_name
    -- if s3
        -- create bucket with name id_challenge_name
        -- create folder (index or whatever?) with id_challenge_name

-- Transactions:
    -- Each transaction will have data_intended_location as:
        -- s3_case: rclone_remote_name:id_challenge_name/id_challenge/name/release_name
        -- drive_case: rclone_remote_name:id_challenge_name/release_name

    -- If has access_bindings, will try to update access to the specific folder or s3 bucket in question
    -- If it has no source_data_location or rows_to_push, we assume it is only a folder / bucket creation / configuration
    -- ...transaction without any data upload attached

-- Later, will have to add option to manually create individual transactions, or
-- ... otherwise decide what transactions goes into what dispatch locations...

-- TODO: Add check or test to access_bindings jsonb or something to ensure is not an empty array. NOT NULL only ensures it is something, even if empty
