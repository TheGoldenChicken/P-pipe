import numpy as np
from request_status import RequestStatus

# TODO: Add support for two other type_of_request cases... Right now we only support DataValidation

# TODO: Add preprocessing steps to ease the process a bit more?
# Like for example, casting all values to float if possible, making names lower, etc...

def compare_values(v1, v2, rtol=1e-6, atol=1e-6):
    """Compare two values with tolerance for floats."""
    if isinstance(v1, (int, float)) and isinstance(v2, (int, float)):
        return np.isclose(v1, v2, rtol=rtol, atol=atol)
    return v1 == v2

def dicts_equal(d1, d2, rtol=1e-6, atol=1e-6):
    """Compare two dicts key-by-key with float tolerance, allowing missing keys."""
    keys = set(d1.keys()) | set(d2.keys())
    for key in keys:
        v1 = d1.get(key, None)
        v2 = d2.get(key, None)
        if not compare_values(v1, v2, rtol=rtol, atol=atol):
            return False, key, v1, v2
        
    # Not interested in returning key or values if theh match..
    return True, None, None, None


def compare_by_row(expected_list, submitted_list, rtol=1e-6, atol=1e-6):
    """
    Compare two lists of dicts using 'row' as the identifier.
    Assumes we do not have syntaxerror or DEADLINEEXCEEDED ALREADY
    - Counts partial matches for intersecting rows.
    - Reports missing/extra rows.
    - Uses np.isclose for numeric tolerance.
    """
    # Index by row
    exp_by_row = {item["row"]: item for item in expected_list}
    sub_by_row = {item["row"]: item for item in submitted_list}

    exp_rows = set(exp_by_row.keys())
    sub_rows = set(sub_by_row.keys())

    # Row sets
    intersect_rows = exp_rows & sub_rows
    missing_rows = sorted(exp_rows - sub_rows)
    extra_rows = sorted(sub_rows - exp_rows)

    # Compare intersecting rows
    matches = 0
    mismatches = []
    for r in sorted(intersect_rows):
        equal, key, v1, v2 = dicts_equal(exp_by_row[r], sub_by_row[r], rtol=rtol, atol=atol)
        if equal:
            matches += 1
        else:
            mismatches.append({
                "row": r,
                "key": key,
                "expected": v1,
                "submitted": v2
            })

    # Summary stats
    total_expected = len(expected_list)
    total_submitted = len(submitted_list)
    percent_expected = (matches / total_expected * 100) if total_expected else 0.0
    percent_submitted = (matches / total_submitted * 100) if total_submitted else 0.0

    status = RequestStatus.PENDING
    match matches:
        case 0:
            status = RequestStatus.INCORRECT
        case n if n < len(expected_list):
            status = RequestStatus.PARTIAL_CORRECT
        case n if n == len(expected_list):
            status = RequestStatus.CORRECT
            

    return status, {
        "matches": matches,
        "rows_compared": len(intersect_rows),
        "expected_total": total_expected,
        "submitted_total": total_submitted,
        "missing_rows_in_submitted": missing_rows,
        "extra_rows_in_submitted": extra_rows,
        "mismatches": mismatches,  # list of {row, key, expected, submitted}
        "percent_of_expected": percent_expected,
        "percent_of_submitted": percent_submitted,
        "message": (
            f"{matches} rows matched (out of {len(intersect_rows)} overlapping rows). "
            f"Missing rows: {missing_rows or 'none'}. Extra rows: {extra_rows or 'none'}."
        )
    }

if __name__ == "__main__":

    # ---- Example ----
    expected_items = [
        {"row": 5, "petal.length": 1.7, "sepal.length": 5.4, "petal.width": 0.4, "sepal.width": 3.9, "variety": "Setosa"},
        {"row": 6, "petal.length": 1.4, "sepal.length": 4.6, "petal.width": 0.3, "sepal.width": 3.4, "variety": "Setosa"},
        {"row": 7, "petal.length": 1.5, "sepal.length": 5.0, "petal.width": 0.2, "sepal.width": 3.4, "variety": "Setosa"}
    ]

    submitted_items = [
        {"row": 5, "petal.length": 1.7000001, "sepal.length": 5.4, "petal.width": 0.4, "sepal.width": 3.9, "variety": "Setosa"},
        {"row": 6, "petal.length": 1.4, "sepal.length": 4.6, "petal.width": 0.3, "sepal.width": 3.4, "variety": "Setosa"},
        {"row": 7, "petal.length": 1.5, "sepal.length": 5.0, "petal.width": 0.2, "sepal.width": 3.4, "variety": "Setosa"}
    ]

    status, result = compare_by_row(expected_items, submitted_items, rtol=1e-6, atol=1e-6)
    print(result["message"])
    print(f"Matches: {result['matches']}, Overlap rows: {result['rows_compared']}")
    print(f"Missing rows: {result['missing_rows_in_submitted']}")
    print(f"Extra rows: {result['extra_rows_in_submitted']}")
    print(f"Mismatches detail: {result['mismatches']}")
    print(f"Match % (expected): {result['percent_of_expected']:.2f}%")
    print(f"Match % (submitted): {result['percent_of_submitted']:.2f}%")
