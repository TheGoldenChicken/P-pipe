import pandas as pd

# TODO: Move this PDF to another file
def local_csv_data_fetcher(file_path, rows_to_get):
    """
    rows_to_get (list[int, int]): list of to:from values to get from the file in question
    """
    data = pd.read_csv(file_path)

    data = data[rows_to_get[0] : rows_to_get[1]]

    return data
