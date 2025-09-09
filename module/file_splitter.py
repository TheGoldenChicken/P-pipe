import os
import pandas as pd
import click

# TODO: Potentially add exceptions and such to this? Does it makes sense to have it return exit codes and whatnot?
def split_csv_by_proportions(csv_file, proportions, output_dir="splits"):
    """
    Splits a CSV file into multiple parts according to given proportions.
    
    Args:
        csv_file (str): Path to the input CSV file.
        proportions (list of float): List of proportions that sum to 1.0.
        output_dir (str): Directory to save the split CSV files.
    """
    if not abs(sum(proportions) - 1.0) < 1e-6:
        raise ValueError("Proportions must sum to 1.0")

    df = pd.read_csv(csv_file)
    total_rows = len(df)
    os.makedirs(output_dir, exist_ok=True)

    start = 0
    for i, p in enumerate(proportions):
        # TODO: Is there any reason to use int(round)?
        # TODO: Ensure this gets one-off rows and allat correctly... so no rows are missing
        end = start + int(round(p * total_rows))
        df_part = df.iloc[start:end]
        df_part.to_csv(os.path.join(output_dir, f"split_{i+1}.csv"), index=False)
        start = end

# TODO: Ensure consistent naming across files: input_file, csv_file, etc...
@click.command()
@click.option("--csv_file", help="Path to the file to generate splits from")
# TODO: Potentially add like a callback lambda to convert it in-line? But then again... KISS?
@click.option('--proportions', help="List of proportions to generate splits from. Comma seperated, no brackets!")
@click.option('--output_dir', help="Output directory to place splits in")
def split_csv_by_proportions_cmd(csv_file, proportions, output_dir="splits"):
    proportions = [float(p.strip()) for p in proportions.split(',')]

    split_csv_by_proportions(csv_file, proportions, output_dir)

if __name__ == "__main__":
    # Example use:
    # python ../file_splitter.py --csv_file tests/test_data/iris.csv --proportions 0.5,0.25,0.25 --output_dir "tests/test_data/splits"
    split_csv_by_proportions_cmd()