import io
import logging
from googleapiclient.http import MediaIoBaseUpload
from dispatchers.drive_dispatcher.drive_utils import build_drive_service
from dispatchers.drive_dispatcher.drive_tools import find_folder_by_name

def drive_folder_creation(drive_service, drive_folder_name, drive_folder_description=None):
    """
    Should be run once when a new data storage place is initiated

    drive_folder_name (str): name of folder to create
    drive_folder_description (str): initial description to be placed in .txt file in drive folder
    """

    # TODO: Find out what mimeType actually is and why it is important
    folder_metadata = {
        'name': drive_folder_name,
        'mimeType': 'application/vnd.google-apps.folder'
    }

    folder = drive_service.files().create(body=folder_metadata, fields='id').execute()
    # TODO: Find out if folder_id can be leveraged to allow multiple folders with the same name
    folder_id = folder.get('id')
    logging.info(f"Created folder with ID: {folder_id}")

    return folder_id


def drive_folder_permissions_update(drive_service, folder_id, share_information, notify_user=True):
    """
    share_information (list[dict{str:str})]): list of permissions to update to. dicts should be {'type': 'user', 'role': 'writer', 'emailAddress': "email"} as example
    notify_user (bool): Defaults to false, whether or not to notify the users about the updated permissions

    If successful, should return an instance of the newly created permissions (https://developers.google.com/workspace/drive/api/reference/rest/v3/permissions/create)
    """

    result = drive_service.permissions().create(
        fileId=folder_id,
        body=share_information,
        fields='id',
        sendNotificationEmail=notify_user  # Set to True to notify the user
    ).execute()

    return result


def drive_data_dispatch(drive_service, data_to_dispatch, folder_id, data_part_name=None):
    """
    Uploads a pandas DataFrame or a file to Google Drive.
    If data_to_dispatch is a DataFrame, it will be serialized to CSV before upload.
    """

    # Serialize DataFrame to CSV in memory
    buffer = io.BytesIO()
    # data_to_dispatch.to_csv(buffer, index=False)
    # buffer.seek(0)
    buffer.write(data_to_dispatch.to_csv(index=False).encode('utf-8'))
    buffer.seek(0)

    file_metadata = {
        'name': data_part_name if data_part_name else 'dataframe_upload.csv',
        'parents': [folder_id]
    }
    media = MediaIoBaseUpload(buffer, mimetype='text/csv', resumable=True)

    uploaded_file = drive_service.files().create(
        body=file_metadata,
        media_body=media,
        fields='id'
    ).execute()

    logging.info(f"Uploaded file with ID: {uploaded_file.get('id')}")
    return uploaded_file


def combined_drive_dispatch_workflow(drive_service, drive_folder_name, drive_folder_description, share_information, notify_user, data_to_dispatch, data_part_name):
    
    existing_folders = find_folder_by_name(drive_service, drive_folder_name)
    
    # Ignorant implementation right now: Will simply think if folder name exists, that it is the right folder to do it in (albatros memory?)
    if existing_folders is None:
        logging.info(f"No existing folder of name {drive_folder_name} found. Creating folder...")
        folder_id = drive_folder_creation(drive_service, drive_folder_name, drive_folder_description=drive_folder_description)

        # For now, we assume never wanna update permissions after creating folder
        drive_folder_permissions_update(drive_service, folder_id, share_information, notify_user=notify_user)

    # If there was one existing folder, that must be the right one
    elif len(existing_folders) == 1:
        folder_id = existing_folders[0]["id"]

    else:
        # TODO: Find fitting exception for this:
        raise(f"More than 1 folder of name {drive_folder_name} found. Only 1 allowed!")

    drive_data_dispatch(drive_service, data_to_dispatch, folder_id, data_part_name=data_part_name)
    
    folder_link = f"https://drive.google.com/drive/folders/{folder_id}"
    logging.info(f"Data successfully dispatched to folder: {folder_link}")
    
    # TODO: Consider if we need this...
    # return folder_link

if __name__ == "__main__":

    # Preliminary toy-data, just picked from GET api/transactions
    json_mockup = {
        "id": 11,
        "challenge_id": 4,
        "created_at": 1758633913,
        "scheduled_time": 5000,
        "source_data_location": "/home/cicero/ppipe/py_modules/tests/test_data/iris.csv",
        "data_intended_location": "release_0",
        "rows_to_push": [
            0,
            150
        ]
    }

    # TODO: Add this information, either to transactions or to challenges (depends on when you wanna handle setting up of external locations and whatnot...)
    json_mockup["share_information"] = {
        "type": "user",
        "role": "reader",
        "emailAddress": "dderpson99@gmail.com"
    }

    drive_service = build_drive_service()

    json_specifications = json_mockup

    source_data_location = json_mockup["source_data_location"]
    drive_folder_name = json_mockup['data_intended_location']
    drive_folder_description = json_mockup.get("data_description")
    share_information = json_mockup.get("share_information")
    notify_user = True
    data_part_name = None
    rows_to_get = json_mockup["rows_to_push"]

    # Shitty and temporary solution, only for testing purposes!
    import sys
    from pathlib import Path
    sys.path.append(str(Path(__file__).resolve().parent.parent.parent))
    from fetchers.misc_fetchers import local_csv_data_fetcher
    data_to_dispatch = local_csv_data_fetcher(source_data_location, rows_to_get)

    combined_drive_dispatch_workflow(drive_service, drive_folder_name, drive_folder_description, share_information, notify_user, data_to_dispatch, data_part_name)

# TODO: Build test for this whole thing!