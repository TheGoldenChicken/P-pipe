import logging


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

