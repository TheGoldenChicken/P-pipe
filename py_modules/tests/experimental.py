# import pytest
# import json
# from pathlib import Path
# import pandas as pd

# from py_modules.orchestrator import orchestrator, find_rclone_remote


# def load_transactions(path: Path):
#     """Load JSON file and return list of transaction dicts."""
#     raw = json.loads(path.read_text(encoding="utf-8"))
#     # If the file contains a single dict, wrap it in a list
#     if isinstance(raw, dict):
#         return [raw]
#     elif isinstance(raw, list):
#         return raw
#     else:
#         raise ValueError("Transaction JSON must be dict or list of dicts")


# @pytest.fixture(params=list((Path(__file__).parent / "json_instances").glob("*.json")))
# def rclone_remote_and_delete(request, tmp_path):
#     """
#     Fixture parameterized over all JSON files in tests/json_instances.
#     If a file contains multiple transactions, yield each one separately.
#     """
#     tx_path = request.param
#     csv_path = Path(__file__).parent / "test_data" / "iris.csv"

#     transactions = load_transactions(tx_path)
#     for tx_dict in transactions:
#         tx_dict["source_data_location"] = str(csv_path)

#         rclone_remote = find_rclone_remote(tx_dict["dispatch_location"])
#         if rclone_remote is None:
#             pytest.skip(f"No rclone remote for {tx_dict['dispatch_location']}")

#         yield tx_dict, rclone_remote, csv_path, tmp_path

#         # Cleanup after each transaction
#         import rclone_python as rclone
#         try:
#             rclone.run_cmd(
#                 command="purge",
#                 extra_args=[f"{rclone_remote}/{tx_dict['data_intended_location']}"]
#             )
#         except Exception as e:
#             raise RuntimeError(f"Cleanup failed for {tx_dict['data_intended_location']}: {e}")


# def test_orchestrator_integration(rclone_remote_and_delete):
#     tx_dict, rclone_remote, csv_path, tmp_path = rclone_remote_and_delete

#     orchestrator(tx_dict, local_folder_path=tmp_path)

#     out_folder = tmp_path / tx_dict["data_intended_location"].replace("_", "-")
#     out_file = out_folder / tx_dict["data_intended_name"].replace("_", "-")
#     assert out_file.exists()

#     written_df = pd.read_csv(out_file)
#     start, end = tx_dict["rows_to_push"]
#     original_df = pd.read_csv(csv_path)
#     expected = original_df.iloc[start:end]
#     pd.testing.assert_frame_equal(
#         written_df.reset_index(drop=True),
#         expected.reset_index(drop=True)
#     )
