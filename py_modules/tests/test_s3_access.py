import boto3
import argparse


def test_s3_access(bucket: str, access_key: str, secret_key: str, session_token: str) -> None:
    s3 = boto3.client(
        "s3",
        aws_access_key_id=access_key,
        aws_secret_access_key=secret_key,
        aws_session_token=session_token,
    )

    print(f"Listing objects in bucket: {bucket}")
    response = s3.list_objects_v2(Bucket=bucket)
    objects = response.get("Contents", [])

    if not objects:
        print("Bucket is empty (but access succeeded).")
        return

    print(f"Found {len(objects)} object(s):")
    for obj in objects:
        print(f"  {obj['Key']} ({obj['Size']} bytes)")

    first_key = objects[0]["Key"]
    print(f"\nFetching first object: {first_key}")
    obj_response = s3.get_object(Bucket=bucket, Key=first_key)
    data = obj_response["Body"].read(256)
    print(f"First 256 bytes: {data[:256]}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Test S3 bucket access with STS credentials")
    parser.add_argument("bucket", help="S3 bucket name")
    parser.add_argument("access_key", help="AWS access key ID")
    parser.add_argument("secret_key", help="AWS secret access key")
    parser.add_argument("session_token", help="AWS session token")
    args = parser.parse_args()

    test_s3_access(args.bucket, args.access_key, args.secret_key, args.session_token)
