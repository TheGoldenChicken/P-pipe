import os
import pytest
from pathlib import Path
from module.file_splitter import split_csv_by_proportions
from module.dispatcher import simple_dispatcher

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

def test_integration_split_dispatch(input_csv, proportions, shared_tmp_dir):
    splits_path = shared_tmp_dir / "splits"

    split_csv_by_proportions(input_csv, proportions, splits_path)

    output_dir = shared_tmp_dir / "dispatched"

    # TODO: Change this to be the "advanced" dispatcher when we make it - I.E the one that can move more than one file at a time!
    simple_dispatcher(str(input_csv), str(output_dir), merge_files=False)

    # Dispatcher test condition
    # TODO: Update when we switch to mroe "advanced" dispatcher
    assert len(os.listdir(output_dir)) > 0
    

if __name__ == "__main__":
    test_integration_split_dispatch()