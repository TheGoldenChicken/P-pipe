import pytest
import json
import pandas as pd
from pathlib import Path
from rclone_python import rclone

from py_modules.orchestrator import unpack_transaction_json, orchestrator
from py_modules.rclone_functions import find_rclone_remote

def test_valid_transaction():
    here = Path(__file__).parent
    tx_path = here / "json_instances" / "transaction_instance_s3.json"
    csv_path = here / "test_data" / "iris.csv"

    # Load transaction JSON and inject the actual CSV path
    tx_json = tx_path.read_text(encoding="utf-8")

    # transaction = json.dumps({
    #     "id": None,
    #     "challenge_id": 42,
    #     "created_at": None,
    #     "scheduled_time": 1120,
    #     "source_data_location": "s3://bucket/data.csv",
    #     "dispatch_location": "S3",
    #     "data_intended_location": "challenge_42_testingchallenge1",
    #     "data_intended_name": "release_2",
    #     "rows_to_push": [210, 300],
    #     "access_bindings": [
    #         {"type": "S3", "identity": "ec2userstuff", "bucket": "somebucket"},
    #         {
    #             "type": "Drive",
    #             "identity": "dderpson99@gmail.com",
    #             "folder_id": "abcd123",
    #             "user_permissions": "Read",
    #         },
    #     ],
    # })

    result = unpack_transaction_json(tx_json)

    assert result["challenge_id"] == 42
    assert result["scheduled_time"] == 1120
    assert result["rows_to_push"] == [210, 300]
    assert result["access_bindings"][0]["type"] == "S3"
    assert result["access_bindings"][1]["identity"] == "dderpson99@gmail.com"

@pytest.fixture(params=[
    "transaction_instance_s3.json",
    "transaction_instance_drive.json"
])
def rclone_remote_and_delete(request, tmp_path):
    """
    MISSING DOCSTRING
    """

    here = Path(__file__).parent
    tx_path = here / "json_instances" / request.param
    csv_path = here / "test_data" / "iris.csv"

    # Load transaction JSON and inject CSV path
    tx_json = tx_path.read_text(encoding="utf-8")
    tx_dict = unpack_transaction_json(tx_json)
    tx_dict["source_data_location"] = str(csv_path)

    # Resolve rclone remote
    rclone_remote = find_rclone_remote(tx_dict["dispatch_location"])
    if rclone_remote is None:
        pytest.skip("Skipping: rclone remote not configured for dispatch_location")

    tx_dict['source_data_location'] = csv_path

    # Yield values to the test
    yield tx_dict, rclone_remote, csv_path, tmp_path

    # Cleanup after test
    try:
        rclone.purge(f"{rclone_remote}/{tx_dict['data_intended_location']}")
    except Exception as e:
        raise RuntimeError(f"Could not clean up remote folder: {e}")
    

def test_orchestrator_integration_s3(rclone_remote_and_delete):
    tx_dict, rclone_remote, csv_path, tmp_path = rclone_remote_and_delete

    # Run orchestrator
    orchestrator(tx_dict, local_folder_path=tmp_path)

    # # Assert output file exists under transaction_releases/
    # out_folder = Path(tmp_path) / tx_dict["data_intended_location"].replace("_", "-")
    # out_file = out_folder / tx_dict["data_intended_name"].replace("_", "-")
    # assert out_file.exists()

    # # Verify slice matches rows_to_push
    # written_df = pd.read_csv(out_file)
    # start, end = tx_dict["rows_to_push"]
    # original_df = pd.read_csv(csv_path)
    # expected = original_df.iloc[start:end]
    # pd.testing.assert_frame_equal(
    #     written_df.reset_index(drop=True),
    #     expected.reset_index(drop=True)
    # )

# def test_orchestrator_integration_s3(tmp_path):
#     # Locate test fixtures relative to this test file
#     here = Path(__file__).parent
#     tx_path = here / "json_instances" / "transaction_instance.json"
#     csv_path = here / "test_data" / "iris.csv"

#     # Load transaction JSON and inject the actual CSV path
#     tx_json = tx_path.read_text(encoding="utf-8")
#     tx_dict = unpack_transaction_json(tx_json)
#     tx_dict["source_data_location"] = str(csv_path)


#     rclone_remote = find_rclone_remote(tx_dict['dispatch_location'])
#     assert rclone_remote is not None, "No rclone remote found, cannot remove after testing!"

#     # Run orchestrator; skip if no rclone remote configured
#     try:
#         orchestrator(tx_dict, local_folder_path=tmp_path)
#     except ValueError as e:
#         if "No rclone remote found" in str(e):
#             pytest.skip("Skipping: rclone remote not configured for dispatch_location")
#         raise

#     # TODO: This currently fails, likely because of pathing issues. Check why.
#     # # Assert output file exists under transaction_releases/
#     # out_folder = Path(tmp_path) / tx_dict["data_intended_location"].replace("_", "-")
#     # out_file = out_folder / tx_dict["data_intended_name"].replace("_", "-")
#     # assert out_file.exists()

#     # # Verify slice matches rows_to_push
#     # written_df = pd.read_csv(out_file)
#     # start, end = tx_dict["rows_to_push"]
#     # original_df = pd.read_csv(csv_path)
#     # expected = original_df.iloc[start:end]
#     # pd.testing.assert_frame_equal(written_df.reset_index(drop=True),
#     #                               expected.reset_index(drop=True))


#     # TODO: Add assert or raise here to test that deletion was sucecssful or something?
#     # Clean up by deleting remote folder again
#     # Using purge instead of delete, as this also removes the folder itself
#     try:
#         rclone.purge(f"{rclone_remote}/{tx_dict['data_intended_location']}")
#     # TODO: Bare exception...
#     except:
#         raise "Could not clean up after executing test!"

