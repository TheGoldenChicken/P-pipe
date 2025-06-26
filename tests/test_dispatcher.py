import os
import pytest
from pathlib import Path
from module.dispatcher import simple_dispatcher


# TODO: Add more stuff here I guess... more test cases
@pytest.fixture
def input_file():
    # Resolve path relative to this test file
    test_dir = Path(__file__).parent
    csv_path = test_dir / "test_data" / "splits" / "split_1.csv"
    return csv_path
    
def test_dispatcher(input_file, tmp_path):
    output_dir = tmp_path

    simple_dispatcher(str(input_file), str(output_dir), merge_files=False)

    assert len(os.listdir(output_dir)) > 0

# TODO: Will this fuck up the command line tool thingy?
if __name__ == "__main__":
    test_dispatcher()

# def test_simple_dispatcher(tmp_path):
#     # runner = CliRunner()
#     # TODO: Combine this with running the file_splitter for actual testing on a dumm file
#     input_file = 'split_1.csv'
#     input_dir = 'iris_data_test/splits'
#     output_dir = 'iris_data_dispatch'
#     # result = runner.invoke(simple_dispatcher, [os.path.join(input_dir, input_file), output_dir])

#     assert result.exit_code == 0
#     # TODO: Make this check if all splitfiles are there...
#     assert len(os.listdir(output_dir)) > 0

#     # Clean up afterwards...
#     try:
#         shutil.rmtree(output_dir)
#     except FileNotFoundError:
#         # TODO: Replace this with warning?
#         print("WARNING: Could not find directory to remove for cleaning")

