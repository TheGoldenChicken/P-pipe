import os
import shutil

from file_splitter import simple_dispatcher


# TODO: Move this to top of file or to a dedicated test folder...
from click.testing import CliRunner

def test_simple_dispatcher():
    runner = CliRunner()
    # TODO: Combine this with running the file_splitter for actual testing on a dumm file
    input_file = 'split_1.csv'
    input_dir = 'iris_data_test/splits'
    output_dir = 'iris_data_dispatch'
    result = runner.invoke(simple_dispatcher, [os.path.join(input_dir, input_file), output_dir])

    assert result.exit_code == 0
    # TODO: Make this check if all splitfiles are there...
    assert len(os.listdir(output_dir)) > 0

    # Clean up afterwards...
    try:
        shutil.rmtree(output_dir)
    except FileNotFoundError:
        # TODO: Replace this with warning?
        print("WARNING: Could not find directory to remove for cleaning")

# TODO: Will this fuck up the command line tool thingy?
if __name__ == "__main__":
    test_simple_dispatcher()