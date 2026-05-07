import boto3



BUCKET = "challenge-1-iris-classification"

s3 = boto3.client(
    "s3",
    aws_access_key_id=ACCESS_KEY,
    aws_secret_access_key=SECRET_KEY,
    aws_session_token=SESSION_TOKEN,
)

response = s3.list_objects_v2(Bucket=BUCKET)

contents = response.get("Contents", [])
if not contents:
    print("Bucket is empty or not accessible.")
else:
    print(f"Found {len(contents)} object(s):\n")
    for obj in contents:
        print(f"  {obj['Key']}  ({obj['Size']} bytes)")
