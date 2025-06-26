import os
import pandas as pd
import pytest
from pathlib import Path
from module.file_splitter import split_csv_by_proportions

@pytest.fixture
def input_csv():
    test_dir = Path(__file__).parent
    csv_path = test_dir / "test_data" / "iris.csv"

    return csv_path

@pytest.fixture
def proportions():
    initial_proportion = 0.5
    splits = 6
    remainder_proportions =  (1 - initial_proportion) / (splits - 1)
    proportions = [initial_proportion] + [remainder_proportions] * (splits - 1)
    return proportions

def test_file_splitter(input_csv, proportions, tmp_path):
    output_dir = tmp_path / "splits"

    split_csv_by_proportions(input_csv, proportions, output_dir)

    # Test correct number of files
    assert len(os.listdir(output_dir)) == len(proportions)

    # Test the proportion_sizes
    # TODO: Use this as like a unit test in the future
    initial_csv_size = len(pd.read_csv(input_csv))

    csv_proportions_size = 0
    for i, csv_file in enumerate(sorted(os.listdir(output_dir))):
        current_proportion_len = len(pd.read_csv(os.path.join(output_dir, csv_file))) 
        csv_proportions_size += current_proportion_len
        
        assert current_proportion_len == int(round(proportions[i] * initial_csv_size))

    assert csv_proportions_size == initial_csv_size

if __name__ == "__main__":
    test_file_splitter()

# # TODO: Split this up into multiple independent tests...
# if __name__ == "__main__":
#     runner = CliRunner()

#     initial_proportion = 0.5
#     splits = 6
#     remainder_proportions =  (1 - initial_proportion) / (splits - 1)
#     proportions = [initial_proportion] + [remainder_proportions] * (splits - 1)

#     # TODO: Check if this is a correct directory when just running as tests
#     input_csv = "test_data/iris.csv"
#     output_dir = "iris_data_test/splits"

#     # TODO: Ensure that the invokation is done exactly as it would be in the command line - I.E. With text and whatnot
#     # result = runner.invoke(split_csv_by_proportions, [input_csv, proportions, output_dir])
#     result = runner.invoke(
#         split_csv_by_proportions,
#         [
#             "--file_name", input_csv,
#             "--proportions", ",".join(map(str, proportions)),
#             "--output_dir", output_dir
#         ]
#     )

#     # Test that it actually ran sucessfully
#     assert result.exit_code == 0

#     # Test correct number of files
#     assert os.listdir(output_dir) == len(proportions)

#     # Test the proportion_sizes
#     # TODO: Use this as like a unit test in the future
#     initial_csv_size = len(pd.read_csv(input_csv))

#     csv_proportions_size = 0
#     for i, csv_file in enumerate(sorted(os.listdir(output_dir))):
#         current_proportion_len = len(pd.read_csv(os.path.join(output_dir, csv_file))) 
#         csv_proportions_size += current_proportion_len
        
#         assert current_proportion_len == int(round(proportions[i] * initial_csv_size))

#     assert csv_proportions_size == initial_csv_size

#     # Clean up afterwards...
#     try:
#         shutil.rmtree(output_dir)
#     except FileNotFoundError:
#         # TODO: Replace this with warning?
#         print("WARNING: Could not find directory to remove for cleaning")
