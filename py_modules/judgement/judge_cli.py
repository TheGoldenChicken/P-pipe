import click
import json
import sys

from typing import Tuple

from judge import compare_by_row
from request_status import RequestStatus

# TODO: Add support for effectinv own tolerances
def unpack_completed_request_json(completed_request: str):
    try:
        parsed = json.loads(completed_request)
    except json.JSONDecodeError as e:
        raise ValueError(f"Invalid JSON: {e}")

    id = parsed['id']
    challenge_id = parsed['challenge_id']
    created_at = parsed['created_at']
    deadline = parsed.get('deadline', None)
    request_status = parsed['request_status']
    submitted_at = parsed['submitted_at']
    
    type_of_request = parsed['type_of_request']
    expected_response = parsed['expected_response']
    submitted_response = parsed['submitted_response']

    full_dict_return = {
        'id': id,
        'challenge_id': challenge_id,
        'created_at': created_at,
        'deadline': deadline,
        'request_status': request_status,
        'submitted_at': submitted_at,
        'type_of_request': type_of_request,
        'expected_response': expected_response,
        'submitted_response': submitted_response,
    }    

    return full_dict_return

def judge(completed_request_dict: dict) -> Tuple[RequestStatus, dict]:
    """TODO: MISSING DOCSTRING

    Args:
        completed_request_dict (_type_): _description_
    """

    match completed_request_dict['type_of_request']['type']:
        case "DataValidation":
            submitted_items = completed_request_dict['submitted_response']['items']
            expected_items = completed_request_dict['expected_response']['items']
            status, result = compare_by_row(submitted_list=submitted_items, expected_list=expected_items)

        case _:
            raise NotImplementedError(f"No Python judge support for type of request: {completed_request_dict['type_of_request']['type']}") 
    
    # Could exit python from here, but best to go all the way up again, and properly exit through the CLI, I guess
    return status, result

@click.command()
@click.option("--completed_request", required=True, help="completed request as a JSON String")
def judge_cli(completed_request: str):
    completed_request_dict = unpack_completed_request_json(completed_request)

    status, result = judge(completed_request_dict)

    # json comes as text output
    print(json.dumps(result))

    # Exit with exit code corresponding to the given status
    sys.exit(status.value)

if __name__ == "__main__":
    judge_cli()

# Example input
# [{"id":1,"challenge_id":1,"created_at":1765193918,"type_of_request":{"type":"DataValidation","items":[5,6,7],"count":3},"expected_response":{"type":"BatchPrediction","items":[{"row":5,"petal.width":0.4,"variety":"Setosa","sepal.width":3.9,"sepal.length":5.4,"petal.length":1.7},{"row":6,"variety":"Setosa","petal.width":0.3,"sepal.width":3.4,"petal.length":1.4,"sepal.length":4.6},{"row":7,"variety":"Setosa","petal.width":0.2,"sepal.width":3.4,"petal.length":1.5,"sepal.length":5.0}],"count":3},"deadline":1700000000,"request_status":"Pending","submitted_at":1765193918252,"submitted_response":{"type":"BatchPrediction","items":[{"petal.length":1.7,"row":5,"sepal.width":3.9,"variety":"Setosa","petal.width":0.4,"sepal.length":5.4},{"petal.width":0.3,"sepal.length":4.6,"row":6,"petal.length":1.4,"variety":"Setosa","sepal.width":3.4},{"sepal.length":5.0,"sepal.width":3.4,"petal.length":1.5,"petal.width":0.2,"row":7,"variety":"Setosa"}],"count":3}}]
# call using
# python py_modules/judgement/judge_cli.py --completed_request '{"id":1,"challenge_id":1,"created_at":1765193918,"type_of_request":{"type":"DataValidation","items":[5,6,7],"count":3},"expected_response":{"type":"BatchPrediction","items":[{"row":5,"petal.width":0.4,"variety":"Setosa","sepal.width":3.9,"sepal.length":5.4,"petal.length":1.7},{"row":6,"variety":"Setosa","petal.width":0.3,"sepal.width":3.4,"petal.length":1.4,"sepal.length":4.6},{"row":7,"variety":"Setosa","petal.width":0.2,"sepal.width":3.4,"petal.length":1.5,"sepal.length":5.0}],"count":3},"deadline":1700000000,"request_status":"Pending","submitted_at":1765193918252,"submitted_response":{"type":"BatchPrediction","items":[{"petal.length":1.7,"row":5,"sepal.width":3.9,"variety":"Setosa","petal.width":0.4,"sepal.length":5.4},{"petal.width":0.3,"sepal.length":4.6,"row":6,"petal.length":1.4,"variety":"Setosa","sepal.width":3.4},{"sepal.length":5.0,"sepal.width":3.4,"petal.length":1.5,"petal.width":0.2,"row":7,"variety":"Setosa"}],"count":3}}'

# Example Python input
# data = [
#     {
#         "id": 1,
#         "challenge_id": 1,
#         "created_at": 1765193918,
#         "type_of_request": {
#             "type": "DataValidation",
#             "items": [5, 6, 7],
#             "count": 3
#         },
#         "expected_response": {
#             "type": "BatchPrediction",
#             "items": [
#                 {
#                     "row": 5,
#                     "petal.width": 0.4,
#                     "variety": "Setosa",
#                     "sepal.width": 3.9,
#                     "sepal.length": 5.4,
#                     "petal.length": 1.7
#                 },
#                 {
#                     "row": 6,
#                     "variety": "Setosa",
#                     "petal.width": 0.3,
#                     "sepal.width": 3.4,
#                     "petal.length": 1.4,
#                     "sepal.length": 4.6
#                 },
#                 {
#                     "row": 7,
#                     "variety": "Setosa",
#                     "petal.width": 0.2,
#                     "sepal.width": 3.4,
#                     "petal.length": 1.5,
#                     "sepal.length": 5.0
#                 }
#             ],
#             "count": 3
#         },
#         "deadline": 1700000000,
#         "request_status": "Pending",
#         "submitted_at": 1765193918252,
#         "submitted_response": {
#             "type": "BatchPrediction",
#             "items": [
#                 {
#                     "petal.length": 1.7,
#                     "row": 5,
#                     "sepal.width": 3.9,
#                     "variety": "Setosa",
#                     "petal.width": 0.4,
#                     "sepal.length": 5.4
#                 },
#                 {
#                     "petal.width": 0.3,
#                     "sepal.length": 4.6,
#                     "row": 6,
#                     "petal.length": 1.4,
#                     "variety": "Setosa",
#                     "sepal.width": 3.4
#                 },
#                 {
#                     "sepal.length": 5.0,
#                     "sepal.width": 3.4,
#                     "petal.length": 1.5,
#                     "petal.width": 0.2,
#                     "row": 7,
#                     "variety": "Setosa"
#                 }
#             ],
#             "count": 3
#         }
#     }
# ]

# # Convert to a single JSON string
# input_str = json.dumps(data)

# print(input_str)
