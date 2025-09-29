from pathlib import Path
import os
import sys
sys.path.insert(0, os.getcwd())
from dispatcher_module.dispatcher import simple_dispatcher
from dispatcher_module.file_splitter import split_csv_by_proportions

def main():
    data_folder = "test_data"
    input_csv_path = Path(data_folder) / "iris.csv"
    proportions = [0.4, 0.2, 0.2, 0.2]
    split_output_dir = Path(data_folder) / "splits"
    
    split_csv_by_proportions(input_csv_path, proportions, split_output_dir)

    file_to_dispatch = os.listdir(split_output_dir)[0]
    file_to_dispatch = split_output_dir / file_to_dispatch

    dispatcher_output_dir = Path(data_folder) / "output_files"
    simple_dispatcher(file_to_dispatch, dispatcher_output_dir, merge_files=False)

    return "derp" # Right now, need to return a PyString, should change that...

if __name__ == "__main__": 
    main()