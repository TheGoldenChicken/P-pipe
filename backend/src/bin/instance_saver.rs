use backend::testing_common::instances::{challenge_instance, transaction_instance, minimal_challenge_instance, challenge_instance_multiple_dispatch, transactions_expected_from_challenge_instance};
use backend::testing_common::save_instance::save_instance;


// TODO: Minor changes to this so it correctly saves to py_modules/tests/json_instances
fn main() -> std::io::Result<()> {
    save_instance(&challenge_instance(), "challenge_instance")?;
    save_instance(&minimal_challenge_instance(), "minimal_challenge_instance")?;
    save_instance(&challenge_instance_multiple_dispatch(), "challenge_instance_multiple_dispatch")?;
    save_instance(&transaction_instance(), "transaction_instance")?;
    save_instance(&transactions_expected_from_challenge_instance(), "transactions_expected_from_challenge_instance")?;

    Ok(())
}
