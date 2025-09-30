import os
import os.path
import logging
from dotenv import load_dotenv

from googleapiclient.discovery import build
from google_auth_oauthlib.flow import InstalledAppFlow
from google.auth.transport.requests import Request
from google.oauth2.credentials import Credentials



SCOPES = ['https://www.googleapis.com/auth/drive.file']

def oauth2_or_token_authentication():
    try:
        load_dotenv()
        creds_path = os.getenv("GOOGLE_CREDENTIALS")
        token_path = os.getenv("GOOGLE_TOKEN")

    except FileNotFoundError:
        raise("No GOOGLE_CREDENTIALS and/or GOOGLE_TOKEN path found, is you .env file present?")
    
    retry_creds = False
    
    # Load existing credentials if available
    creds = None
    if os.path.exists(token_path):
        logging.info(f"Drive token file found at {token_path}")
        creds = Credentials.from_authorized_user_file(token_path, SCOPES)

        if creds.valid:
            logging.warning("Token found was not valid")
            retry_creds = True
    else:
        logging.warning(f"No token file found at {token_path}")
        retry_creds = True

    # If no valid credentials, run the flow
    if retry_creds:
        if creds and creds.expired and creds.refresh_token:
            logging.info("Credentials expired, refreshing")
            creds.refresh(Request())
        else:
            logging.warning("Credentials expired and unable to refresh, or not present at all. Manual oauth2 necessary!")
            if not os.path.exists(creds_path):
                raise FileNotFoundError(f"No google credentials found at {creds_path}! Cannot continue with oauth2")
            flow = InstalledAppFlow.from_client_secrets_file(creds_path, SCOPES)
            creds = flow.run_local_server(port=0)

        # Save the credentials for next time
        with open(token_path, 'w') as token:
            logging.info(f"Saving new credentials to {token_path}")
            token.write(creds.to_json())
    
    if creds is None:
        # TODO: Find fitting error to raise here
        raise("Unable to create proper drive credentials!")

    return creds

def build_drive_service():
    creds = oauth2_or_token_authentication()
    # flow = InstalledAppFlow.from_client_secrets_file('dispatcher/credentials.json', SCOPES)
    # creds = flow.run_local_server(port=0)
    service = build('drive', 'v3', credentials=creds)
    return service

if __name__ == "__main__":
    # oauth2_or_token_authentication()
    service = build_drive_service()