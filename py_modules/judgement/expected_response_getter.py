import pandas as pd

# Helper function suposed to be used in more than one place... Not relevant *right* now
def df_to_accepted_json(df: pd.DataFrame) -> list[dict]:
    """
    Meant to get the "items" field of a RequestType based on a given dataframe
    """
    # Copy to not fuck up original df
    subset = df.copy()
    # Insert row information
    subset.insert(0, "row", subset.index)
    items_field = subset.to_dict(orient="records")

    return items_field

# NOTE: We don't need data_validation_given_data, since this is just indices, which we already have...

def data_validation_expected_response(transaction: dict, request: dict):
    rows_to_get: list[int] = request['items'] # In the case of DataValidation, items should just be a list[int]

    # TODO Potentially use orchestrator.unpack_transaction_json here...
    # TODO: Replace with this actualy data fetcher, same as for orchestrator.orchestrator function...
    
    # Get subset of data based on transaction
    source_data_location = transaction['source_data_location']
    expected_data_part: pd.DataFrame = pd.read_csv(source_data_location).iloc[rows_to_get]
    items_field: list[dict] = df_to_accepted_json(expected_data_part)

    # Created expected response as a BatchPrediction type
    expected_response = {
        "type": "BatchPrediction",
        "items": items_field,
        "count": len(items_field)
    }

    return expected_response


def batch_prediction_given_data(transaction, given_request):
    pass
    # Should get the data to pass to the user from a list of indices in a request, much like how data_validation creates an expected response, except shouldn't include 
    # any target columns...


def batch_prediction_expected_response(transaction, given_request):
    pass
    # Should get the expected response for a batch prediciton... duh


# We don't need computed_feature_given_data, since that is just rows and information on how to use features to calculate it
# We DO need a computed_feature_expected_response that takes like a lambda and uses that to calculate the features....

# TODO: See if there isn't another way in which we can get the transaction data, we won't have a valid transaction when we're making requests randomly...
def get_expected_response(transaction: dict, request: dict) -> dict:
    match request['type']:
        case "DataValidation":
            expected_response = data_validation_expected_response(transaction, request)        
        case "BatchPrediction": 
            raise NotImplementedError("Expected response for BatchPrediction not implemented yet!")
        case "CalculatedFeature": 
            raise NotImplementedError("Expected response for CalculatedFeature not implemented yet!")
        case _:
            raise TypeError(f"Unknown RequestType for given_request {request['type']}!")

    return expected_response


