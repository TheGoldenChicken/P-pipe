import shutil
import click
import os

# TODO: If merge_files=True, have dispatcher overwrite current .csv file so user will have to check if it has changed since last
def simple_dispatcher(input_file, output_dir, merge_files=False):
    if merge_files is False:
        os.makedirs(output_dir, exist_ok=True)
        # TODO: Change this to use pathlib?
        output_file = os.path.join(output_dir, os.path.basename(input_file)) 
        shutil.copyfile(input_file, output_file)

@click.command()
@click.option('--input_file', help='The path to the file to dispatch')
@click.option('--output_dir', help='The path to the dir to dispatch the input file to')
# TODO: Perhaps set this as a flag? Might make unit testing a bit harder! Also see if it even makes sense to have it as a bool type? Confusing, perhaps?
@click.option('--merge_files', help='Whether to overwrite existing files each time the dispatcher runs', required=False, type=bool)
def simple_dispatcher_cmd(input_file, output_dir, merge_files=False):
    simple_dispatcher(input_file, output_dir, merge_files)
    

if __name__ == '__main__':
    simple_dispatcher_cmd()