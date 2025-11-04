from rclone_python import rclone
import logging


def find_rclone_remote(dispatch_location: str):
    """Returns the name of an rclone remote whose remote type matches the requested dispatch location.

    Args:
        dispatch_location (str): _description_

    Returns:
        str | None The name of the matching rclone remote, or None if not found.
    """

    all_remotes: list[str] = rclone.get_remotes()
    
    for remote in all_remotes:
        if rclone.type_remotes(remote).lower() == dispatch_location.lower():
            return remote

    logging.error(f"No remote amongst {all_remotes} matched dispatch_location: {dispatch_location}")

    # "'return none' is used when there are other possible return values for the function: https://stackoverflow.com/questions/15300550/python-return-return-none-and-no-return-at-all-is-there-any-difference"
    return None


def rclone_copy_file(upload_file_path: str, rclone_remote_name: str, folder_name=None, **kwargs):
    """Copies a file from a location locally to a given remote. Assumes folder of folder_name is already created

    Args:
        upload_file_path (str): full path to the file to upload to rclone remote
        rclone_remote_name (str): name of the rclone remote to use for the upload
        folder_name (str, optional): name of the folder which rclone will upload to. Defaults to None.        
    """


    if folder_name is None:
        raise ValueError("No proper way of handling if folder name is None yet!")

    rclone_full_remote = rclone_remote_name + folder_name

    rclone.copy(upload_file_path, rclone_full_remote)


def rclone_init_dir(folder_name: str, rclone_remote_name: str):
    """TODO: MISSING DOCSTRING

    Args:
        dirname (str): _description_
        rclone_remote_name (str): _description_
    """

    rclone.mkdir(folder_name, rclone_remote_name)

