import requests
import pandas as pd

data_validation_rows = [5, 6, 7]
# response = requests.put("http://127.0.0.1:8000/api/requests/1/2", json=payload)
response = requests.get("http://127.0.0.1:8000/api/requests/1")


print("Status code:", response.status_code)
print("Response:", response.json()["requests"][0]["request_id"])


# payload = {
#     "type": "BatchPrediction",
#     "items": [
#         {
#             "row": 5,
#             "sepal.length": 5.4,
#             "sepal.width": 3.9,
#             "petal.length": 1.7,
#             "petal.width": 0.4,
#             "variety": "Setosa"
#         },
#         {
#             "row": 6,
#             "sepal.length": 4.6,
#             "sepal.width": 3.4,
#             "petal.length": 1.4,
#             "petal.width": 0.3,
#             "variety": "Setosa"
#         },
#         {
#             "row": 7,
#             "sepal.length": 5.0,
#             "sepal.width": 3.4,
#             "petal.length": 1.5,
#             "petal.width": 0.2,
#             "variety": "Setosa"
#         }
#     ],
#     "count": 3
# }
