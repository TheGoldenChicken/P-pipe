import click
import json
import logging

from expected_response_getter import get_expected_response

@click.command()
@click.option("--request", required=True, help="Generated part request to create rest of request based off of")
@click.option("--transaction", required=False, default=None, help="Transaction associated with request")
def expected_response_generator_cli(request: str, transaction: str):
    # TODO: Potentially add unpack_json function here to 

    try:
        request = json.loads(request)
        if transaction is not None:
            transaction = json.loads(transaction)
    except json.JSONDecodeError as e:
        raise ValueError(f"Invalid JSON: {e}")

    if transaction is None:
        logging.info("No transaction given, finding source data location from given request")
        source_data_location = request.get('source_data_location', None)
        if source_data_location is None:
            raise TypeError("Transaction not given, and request has no source data location!")

    expected_response_json_formatted = get_expected_response(transaction=transaction, request=request)

    # json comes as text output
    print(json.dumps(expected_response_json_formatted))


if __name__ == "__main__":
    expected_response_generator_cli()


# Example input
# {"type":"DataValidation","items":[120,148,122,123,124,149,126,127,128,129,130,131,132,133,134,135,136,137,138,139,140,141,142,143,144,145,146,147],"count":28}
# {"id":null,"challenge_id":42,"created_at":null,"scheduled_time":1120,"source_data_location":"py_modules/tests/test_data/iris.csv","dispatch_location":"S3","data_intended_location":"challenge_42_testingchallenge1","data_intended_name":"release_2","rows_to_push":[210,300],"access_bindings":[{"type":"S3","identity":"ec2userstuff","bucket":"somebucket"},{"type":"Drive","identity":"dderpson99@gmail.com","folder_id":"abcd123","user_permissions":"Read"}],"challenge_options":{"possible_request_types":null,"makes_requests_on_transaction_push":null,"makes_requests_randomly":null,"min_time_between_requests":null,"max_time_between_requests":null,"requests_deadline":null,"validate_request_immediately_on_answer":null,"allow_retries_on_request":null,"return_completed_request_on_student_answer":null}}
# call using
# python py_modules/judgement/expected_response_cli.py \
#   --request '{"type":"DataValidation","items":[120,148,122,123,124,149,126,127,128,129,130,131,132,133,134,135,136,137,138,139,140,141,142,143,144,145,146,147],"count":28}' \
#   --transaction '{"id":null,"challenge_id":42,"created_at":null,"scheduled_time":1120,"source_data_location":"py_modules/tests/test_data/iris.csv","dispatch_location":"S3","data_intended_location":"challenge_42_testingchallenge1","data_intended_name":"release_2","rows_to_push":[210,300],"access_bindings":[{"type":"S3","identity":"ec2userstuff","bucket":"somebucket"},{"type":"Drive","identity":"dderpson99@gmail.com","folder_id":"abcd123","user_permissions":"Read"}],"challenge_options":{"possible_request_types":null,"makes_requests_on_transaction_push":null,"makes_requests_randomly":null,"min_time_between_requests":null,"max_time_between_requests":null,"requests_deadline":null,"validate_request_immediately_on_answer":null,"allow_retries_on_request":null,"return_completed_request_on_student_answer":null}}'
