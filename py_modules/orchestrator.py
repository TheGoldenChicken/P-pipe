import click
import json
from py_modules import rclone_functions

# TODO: Currently missing functionality
# 1. Giving other users permission to drive and s3 buckets
    # May be particularly difficult for drive folders, since they require that we know their ID (fixable by adding UUIDs)
    # *will* require us to go outside rclone functionality, RIP
# 2. Initializing folders, s3 buckets and whatnot
    # should be trivial with rclone.mkdir()
# 3. validating that a transaction is formatted correctly, (contains correct values and whatnot)
# 4. Adding UUIDs as part of each challenge and/or transaction
# 5. Adding generalized data readers for multiple file types (fetchers)
# 6. adding generalized data savers, so these read files can also be subdivided
    #   ...and locally saved in a proper manner for multiple file types (savers)


def unpack_transaction_json(transaction: str):
    try:
        parsed = json.loads(transaction)
    except json.JSONDecodeError as e:
        raise ValueError(f"Invalid JSON: {e}")

    # missing = [field for field in REQUIRED_FIELDS if field not in parsed]
    # if missing:
    #     raise ValueError(f"Missing required fields: {', '.join(missing)}")
    
    # Unpack required fields
    id = parsed['id']
    challenge_id = parsed['challenge_id']
    created_at = parsed['created_at']
    scheduled_time = parsed['scheduled_time']
    
    # Unpack optional fields
    source_data_location = parsed.get('source_data_location')
    dispatch_location = parsed.get('dispatch_location')
    data_intended_location = parsed.get('data_intended_location')
    data_intended_name = parsed.get('data_intended_name')
    rows_to_push = parsed.get('rows_to_push')
    access_bindings = parsed.get('access_bindings')

    full_dict_return = {
        'id': id,
        'challenge_id': challenge_id,
        'created_at': created_at,
        'scheduled_time': scheduled_time,
        'source_data_location': source_data_location, # ACTUALLY USED
        'dispatch_location': dispatch_location, # ACTUALLY USED
        'data_intended_location': data_intended_location, # ACTUALLY USED
        'data_intended_name': data_intended_name, # ACTUALLY USED
        'rows_to_push': rows_to_push, # ACTUALLY USED
        'access_bindings': access_bindings
    }    

    return full_dict_return


# def validate_transaction_dict(transaction_dict):
#     """Validates whether a transaction dict is viable for what it needs to do
#     And returns the intended purpose (update of permissions, initilization, or data transaction)

#     Args:
#         transaction_dict (dict): Dictionary of transaction as returned by unpack_transaction_json
#     """


def orchestrator(transaction_dict, local_folder_path=None):
    """TODO: MISSING DOCSTRING

    Args:
        transaction_dict (_type_): _description_

    Raises:
        ValueError: _description_
    """
    dispatch_location = transaction_dict['dispatch_location']
    rclone_remote = rclone_functions.find_rclone_remote(dispatch_location)
    if rclone_remote is None:
        raise ValueError(f"No rclone remote found based on dispatch location: {dispatch_location}")

    # TODO: Replace with variable data loader function
    import pandas as pd
    source_data_location = transaction_dict['source_data_location']
    data_to_from = transaction_dict['rows_to_push']
    # from pathlib import Path
    # source_data_location = Path(source_data_location).resolve()
    data_part: pd.DataFrame = pd.read_csv(source_data_location)[data_to_from[0]:data_to_from[1]]
    
    transaction_dict = sanitizer(transaction_dict)

    data_intended_location = transaction_dict['data_intended_location']
    import os
    if local_folder_path is None:
        local_folder_path = os.path.join("transaction_releases", data_intended_location)
    os.makedirs(local_folder_path, exist_ok=True)

    data_intended_name = transaction_dict['data_intended_name']
    full_data_part_path = os.path.join(local_folder_path, data_intended_name)
    data_part.to_csv(full_data_part_path)

    rclone_functions.rclone_copy_file(full_data_part_path, 
                                      rclone_remote_name=rclone_remote,
                                      folder_name=data_intended_location)

def sanitizer(transaction_dict):
    """TEMP SANITIZER
    TODO: MAKE ACTUALLY BETTER...

    Args:
        transaction_dict (_type_): _description_
    """

    if transaction_dict['dispatch_location'].lower() == "s3":
        transaction_dict['data_intended_location'] = transaction_dict['data_intended_location'].replace("_", "-")
        transaction_dict['data_intended_name'] = transaction_dict['data_intended_name'].replace("_", "-") 

    return transaction_dict

def make_dir(transaction_dict):
    dispatch_location = transaction_dict['dispatch_location']
    rclone_remote = rclone_functions.find_rclone_remote(dispatch_location)
    if rclone_remote is None:
        raise ValueError(f"No rclone remote found based on dispatch location: {dispatch_location}")

    folder_name = transaction_dict["data_intended_location"]

    rclone_functions.rclone_init_dir(folder_name, rclone_remote)

@click.group()
def cli():
    pass

@click.command()
@click.option("--transaction", required=True, help="transaction as JSON string")
def orchestrator_cli(transaction):
    transaction_dict = unpack_transaction_json(transaction)
    orchestrator(transaction_dict)

@click.command()
@click.option("--transaction", required=True, help="transaction as JSON string")
def make_dir_cli(transaction):
    transaction_dict = unpack_transaction_json(transaction)
    make_dir(transaction_dict)

cli.add_command(orchestrator_cli)
cli.add_command(make_dir_cli)

if __name__ == "__main__":
    cli()
    # orchestrator_cli()

    # example_input = {
    #     "id": 1,
    #     "challenge_id": 1,
    #     "created_at": 1761920926,
    #     "scheduled_time": 5000,
    #     "source_data_location": "/home/cicero/ppipe/py_modules/tests/test_data/iris.csv",
    #     "dispatch_location": "S3",
    #     "data_intended_location": "challenge_1_testingchallenge1",
    #     "data_intended_name": "release_0",
    #     "rows_to_push": [
    #         0,
    #         150
    #     ],
    #     "access_bindings": [
    #         {
    #             "type": "S3",
    #             "identity": "arn:aws:iam::123456789012:user/alice",
    #             "bucket": "ml-challenges"
    #         },
    #         {
    #             "type": "Drive",
    #             "identity": "user:bob@example.com",
    #             "folder_id": "abc123",
    #             "user_permissions": "editor"
    #         },
    #         {
    #             "type": "Drive",
    #             "identity": "user:carol@example.com",
    #             "folder_id": None,
    #             "user_permissions": "viewer"
    #         }
    #     ]
    # }


# test with
"""
$ python py_modules/orchestrator.py orchestrator-cli --transaction '{
  "id": 1,
  "challenge_id": 1,
  "created_at": 1761920926,
  "scheduled_time": 5000,
  "source_data_location": "/home/cicero/ppipe/py_modules/tests/test_data/iris.csv",
  "dispatch_location": "S3",
  "data_intended_location": "challenge1testingchallenge1",
  "data_intended_name": "release_0",
  "rows_to_push": [0, 150],
  "access_bindings": [
    {
      "type": "S3",
      "identity": "arn:aws:iam::123456789012:user/alice",
      "bucket": "ml-challenges"
    },
    {
      "type": "Drive",
      "identity": "user:bob@example.com",
      "folder_id": "abc123",
      "user_permissions": "editor"
    },
    {
      "type": "Drive",
      "identity": "user:carol@example.com",
      "folder_id": null,
      "user_permissions": "viewer"
    }
  ]
}'
"""
