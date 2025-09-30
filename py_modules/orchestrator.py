import click
import json

from dispatchers.drive_dispatcher.drive_utils import build_drive_service
from dispatchers.drive_dispatcher.drive_dispatcher import combined_drive_dispatch_workflow
from fetchers.misc_fetchers import local_csv_data_fetcher

# TODO: IMPORTANT: Add "dispatcher" as part to each transaction (should simplify how we do dispatches significantly, since we don't need to infer from location)

# TODO: Either:
    # - Some "initial dispatch check", to ensure whether the needed initialization of the directory/thingy has already been performed
    # - OR At least a checker for share_information for drive like "ya donkey, ya forgot the share information!"


REQUIRED_FIELDS = [
    "dispatcher",
    "challenge_id",
    "scheduled_time",
    "source_data_location",
    "data_intended_location",
    "rows_to_push"
]

OPTIONAL_FIELDS = {
    "should_overwrite": False,  # default value
    "share_information": None,
    "notify_user": None
}

def unpack_transaction_json(transaction: str):
    try:
        parsed = json.loads(transaction)
    except json.JSONDecodeError as e:
        raise ValueError(f"Invalid JSON: {e}")

    missing = [field for field in REQUIRED_FIELDS if field not in parsed]
    if missing:
        raise ValueError(f"Missing required fields: {', '.join(missing)}")

    # Unpack required fields
    dispatcher = parsed["dispatcher"]
    challenge_id = parsed["challenge_id"]
    scheduled_time = parsed["scheduled_time"]
    source_data_location = parsed["source_data_location"]
    data_intended_location = parsed["data_intended_location"]
    rows_to_push = parsed["rows_to_push"]

    # Unpack optional fields with defaults
    should_overwrite = parsed.get("should_overwrite", OPTIONAL_FIELDS["should_overwrite"])

    return dispatcher, {
        "challenge_id": challenge_id,
        "scheduled_time": scheduled_time,
        "source_data_location": source_data_location,
        "data_intended_location": data_intended_location,
        "rows_to_push": rows_to_push,
        "should_overwrite": should_overwrite
    }

def unpack_transaction_json_args(transaction: str):
    try:
        parsed = json.loads(transaction)
    except json.JSONDecodeError as e:
        raise ValueError(f"Invalid JSON: {e}")

    missing = [field for field in REQUIRED_FIELDS if field not in parsed]
    if missing:
        raise ValueError(f"Missing required fields: {', '.join(missing)}")

    # Unpack required fields
    transaction_id = parsed["id"]
    dispatcher = parsed["dispatcher"]
    challenge_id = parsed["challenge_id"]
    scheduled_time = parsed["scheduled_time"]
    source_data_location = parsed["source_data_location"]
    data_intended_location = parsed["data_intended_location"]
    rows_to_push = parsed["rows_to_push"]

    # Unpack optional fields with defaults
    should_overwrite = parsed.get("should_overwrite", OPTIONAL_FIELDS["should_overwrite"])
    share_information = parsed.get("share_information", OPTIONAL_FIELDS["share_information"])
    notify_user = parsed.get("notify_user", OPTIONAL_FIELDS["notify_user"])

    return dispatcher, (
        transaction_id,
        challenge_id,
        scheduled_time,
        source_data_location,
        data_intended_location,
        rows_to_push,
        should_overwrite,
        share_information,
        notify_user
    )

# TODO IMPORTANT: This looks fucking ugly, see if there isn't a better way to do this without going 100% **kwargs...
def orchestrator(dispatcher: str, transaction_id, challenge_id, scheduled_time, source_data_location, data_intended_location, rows_to_push, should_overwrite, share_information, notify_user):
    # TODO: Ok, so now we fetch data before actually checking if they even got the right dispatcher? Maybe find a way to reverse that? Not use match statements perhaps???
    
    # TODO: Missing case statement based on where the source data is...
    data_to_dispatch = local_csv_data_fetcher(file_path=source_data_location, rows_to_get=rows_to_push)

    match dispatcher.lower():
        case "drive":
            drive_service = build_drive_service()
            data_part_name = f"challenge_{challenge_id}_transaction_{transaction_id}"
            combined_drive_dispatch_workflow(drive_service=drive_service, drive_folder_name=data_intended_location, drive_folder_description=None,
                                                share_information=share_information, notify_user=notify_user, data_to_dispatch=data_to_dispatch,
                                                data_part_name=data_part_name)
        case _:
            # TODO: Find fitting exception for this
            raise(f"{dispatcher} is not a known dispatcher type!")


@click.command()
@click.option("--transaction", required=True, help="transaction as JSON string")
def orchestrator_cli(transaction):
    dispatcher, args = unpack_transaction_json_args(transaction)
    orchestrator(dispatcher, *args)

if __name__ == "__main__":
    orchestrator_cli()


    {
        "id": 11,
        "challenge_id": 4,
        "created_at": 1758633913,
        "scheduled_time": 5000,
        "source_data_location": "/home/cicero/ppipe/py_modules/tests/test_data/iris.csv",
        "data_intended_location": "test_release",
        "rows_to_push": [
            0,
            150
        ],
        "dispatcher": "drive"
    }
