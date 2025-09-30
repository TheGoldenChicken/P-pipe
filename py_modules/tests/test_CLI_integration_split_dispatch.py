import os
import pytest
from pathlib import Path
from click.testing import CliRunner

from module.file_splitter import split_csv_by_proportions_cmd
from dispatcher.module.dispatcher import simple_dispatcher_cmd
from dispatcher.drive_dispatcher import drive_dispatcher

# tmp_path_factory needed to create a tmp_path that is usuable for the session
# ... necessary for one test to test the output of another test...
# TODO: Remove it, we don't actually need it, since all is run in a single test...
# Though, we could just have it be two tests, though that would change unittests to be integrations tests...
@pytest.fixture(scope="class")
def shared_tmp_dir(tmp_path_factory):
    return tmp_path_factory.mktemp("shared")

@pytest.fixture
def input_csv():
    # Resolve path relative to this test file
    test_dir = Path(__file__).parent
    csv_path = test_dir / "test_data" / "splits" / "split_1.csv"
    return csv_path

@pytest.fixture
def proportions():
    initial_proportion = 0.5
    splits = 6  
    remainder_proportions =  (1 - initial_proportion) / (splits - 1)
    proportions = [initial_proportion] + [remainder_proportions] * (splits - 1)
    return proportions

def test_cli_integration_split_dispatch(input_csv, proportions, shared_tmp_dir):
    runner = CliRunner()

    # TODO: Consider if this should be done in the fixtures instead!
    splits_path = shared_tmp_dir / "splits"
    kwargs = [
        "--csv_file", input_csv,
        "--proportions", ','.join(str(p) for p in proportions), # Mimics how input is put in the terminal!
        "--output_dir", splits_path
    ]
    # TODO: Check if this is best practice - to name it something other than 'result'
    result_split = runner.invoke(split_csv_by_proportions_cmd, kwargs)
    assert result_split.exit_code == 0 # TODO: Consider if checking if exit_code == 0 is worse than doing exit_code == <Result okay> (more rusty? "Enum-like"?)
    
    dispatched_dir = shared_tmp_dir / "dispatched"

    kwargs = [
            "--input_file", input_csv,
            "--output_dir", dispatched_dir,
            "--merge_files", False
        ]

    result_dispatch = runner.invoke(simple_dispatcher_cmd, kwargs)
    assert result_dispatch.exit_code == 0
    
    assert len(os.listdir(dispatched_dir)) > 0

    
if __name__ == "__main__":
    test_cli_integration_split_dispatch()