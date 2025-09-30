import io
import os
import os.path
import logging
from googleapiclient.http import MediaFileUpload, MediaIoBaseUpload
from drive_utils import build_drive_service
import pandas as pd

# TODO: Maybe get intellisense to start working by specifying type-hints for drive_service? It is either a Resource or DriveResource...
# from googleapiclient.discovery import Resource
# Can be seen pretty nicely here:
# from googleapiclient.discovery import build
# from googleapiclient.discovery import Resource
# import importlib.metadata
# print(importlib.metadata.version("google-api-python-client-stubs"))

# drive_service: Resource = build('drive', 'v3')

# drive_service.permissions().create()

# TODO: Move to other file, like utils or smth?
def find_folder_by_name(drive_service, drive_folder_name):
    """
    Should either be used to find out if drive_folder_name is taken. In which case it SHOULD return None
    Or should be used to find the id of a SINGLE folder. In which case SHOULD return One (folder)
    """
    query = f"name = '{drive_folder_name}' and mimeType = 'application/vnd.google-apps.folder'"

    logging.info(f"Searching for folder of name {drive_folder_name} with query: \n {query}")

    results = drive_service.files().list(
        q=query,
        spaces='drive',
        fields='files(id, name)',
        pageSize=10
    ).execute()

    folders = results.get('files', [])
    if folders:
        logging.info(f"Found {len(folders)} folder(s) named '{drive_folder_name}':")
        for folder in folders:
            logging.info(f"- ID: {folder['id']}, Name: {folder['name']}")
        return folders
    else:
        logging.info(f"No folder named '{drive_folder_name}' found.")
        return None


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



# def drive_data_dispatch(drive_service, data_to_dispatch, folder_id, data_part_name=None):
#     """
#     For now, data_to_dispatch should be a file to upload. In future will modify to be any array of data...
#     ... this should work better with PDF and rows_to_push from backend
#     """

#     file_path = data_to_dispatch
#     file_path = 'dispatcher/tests/test_data/iris.csv'  # Replace with your local file path
#     file_metadata = {
#         'name':  data_part_name if data_part_name is not None else os.path.basename(file_path),
#         'parents': [folder_id]
#     }
#     media = MediaFileUpload(file_path, resumable=True)
#     uploaded_file = drive_service.files().create(body=file_metadata, media_body=media, fields='id').execute()
#     logging.info(f"Uploaded file with ID: {uploaded_file.get('id')}")

#     return uploaded_file

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



# TODO: Move this PDF to another file
def local_csv_data_fetcher(file_path, rows_to_get):
    """
    rows_to_get (list[int, int]): list of to:from values to get from the file in question
    """
    data = pd.read_csv(file_path)

    data = data[rows_to_get[0] : rows_to_get[1]]

    return data


if __name__ == "__main__":

    json_mockup = {
        "id": 11,
        "challenge_id": 4,
        "created_at": 1758633913,
        "scheduled_time": 5000,
        "source_data_location": "/home/cicero/ppipe/dispatcher/tests/test_data/iris.csv",
        "data_intended_location": "release_0",
        "rows_to_push": [
            0,
            150
        ]
    }

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

    data_to_dispatch = local_csv_data_fetcher(source_data_location, rows_to_get)
    drive_data_dispatch(drive_service, data_to_dispatch, folder_id, data_part_name=data_part_name)



# TODO: Build test for this whole thing!