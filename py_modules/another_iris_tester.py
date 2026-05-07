import pandas as pd
import requests

# Simply assumes that current data is simply saved as a csv somewhere...
current_data = pd.read_csv("py_modules/tests/test_data/iris.csv")
challenge_url = "http://127.0.0.1:8000/api/requests/1"

# Get indices for rows that need validation
get_resp = requests.get(challenge_url).json()
data_valiation_indices = get_resp["requests"][0]["type_of_request"]["items"]
request_id = get_resp["requests"][0]["request_id"]
data_validation_subset = current_data.iloc[data_valiation_indices]

# Create dictionary to make request body
items = []
for idx, row in data_validation_subset.iterrows():
    item = row.to_dict()
    item["row"] = idx
    items.append(item)

# Create request body
payload = {
    "type": "BatchPrediction",
    "items": items,
    "count": len(items)
}

# Answer request, remember to add request 
response = requests.put(challenge_url + f"/{request_id}", json=payload)


