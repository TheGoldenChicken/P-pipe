import streamlit as st
import requests
import json
from datetime import datetime, timedelta
from typing import List, Dict, Any
import time

# Configuration
API_BASE_URL = "http://localhost:8000"
CHALLENGES_ENDPOINT = f"{API_BASE_URL}/api/challenges"
TRANSACTIONS_ENDPOINT = f"{API_BASE_URL}/api/transactions"
COMPLETED_TRANSACTIONS_ENDPOINT = f"{API_BASE_URL}/api/completed_transactions"
REQUESTS_ENDPOINT = f"{API_BASE_URL}/api/requests"
COMPLETED_REQUESTS_ENDPOINT = f"{API_BASE_URL}/api/completed_requests"

# File picker function using tkinter
def browse_file():
    """Open a file dialog to select a file"""
    try:
        import tkinter as tk
        from tkinter import filedialog

        root = tk.Tk()
        root.withdraw()
        root.wm_attributes('-topmost', 1)

        file_path = filedialog.askopenfilename(
            title="Select Dataset File",
            filetypes=[
                ("CSV files", "*.csv"),
                ("All files", "*.*")
            ]
        )
        root.destroy()
        return file_path
    except Exception as e:
        st.error(f"Error opening file dialog: {e}")
        return None

st.set_page_config(page_title="P-PIPE", page_icon="🅿️", layout="wide")

st.title("🅿️-🅿️IPE")
st.markdown("Mockup frontend for the Production & Pipeline Infrastructure Preparation Exercise")

# Initialize auto-refresh timestamp
if 'last_refresh' not in st.session_state:
    st.session_state.last_refresh = time.time()

# Check if 10 seconds have passed for auto-refresh
current_time = time.time()
if current_time - st.session_state.last_refresh >= 10:
    st.session_state.last_refresh = current_time
    st.rerun()

# Sidebar for viewing existing data
with st.sidebar:
    # View switcher
    view_mode = st.radio(
        "Select View",
        ["📦 Transactions", "📋 Requests"],
        horizontal=True
    )

    if st.button("🔄 Refresh All", use_container_width=True):
        st.session_state.last_refresh = time.time()
        st.rerun()

    st.markdown("---")

    # Challenges Section (always visible)
    st.header("🎯 Challenges")
    try:
        response = requests.get(CHALLENGES_ENDPOINT, timeout=5)
        if response.status_code == 200:
            challenges = response.json()
            if challenges:
                for challenge in challenges:
                    with st.expander(f"ID: {challenge['id']} - {challenge['challenge_name']}"):
                        st.json(challenge)
            else:
                st.info("No challenges found")
        else:
            st.error(f"Failed to fetch challenges: {response.status_code}")
    except requests.exceptions.RequestException as e:
        st.error(f"Cannot connect to API: {e}")

    st.markdown("---")

    if view_mode == "📦 Transactions":
        # Transactions Section
        st.header("📦 Transactions")
        try:
            response = requests.get(TRANSACTIONS_ENDPOINT, timeout=5)
            if response.status_code == 200:
                transactions = response.json()
                st.caption(f"Found {len(transactions)} pending transaction(s)")
                if transactions:
                    for txn in transactions:
                        dispatch_loc = txn.get('dispatch_location', 'N/A')
                        scheduled_time = txn.get('scheduled_time', 0)
                        current_time_ms = int(datetime.now().timestamp() * 1000)
                        minutes_until = (scheduled_time - current_time_ms) / 1000 / 60

                        if minutes_until > 0:
                            time_info = f"in {int(minutes_until)}m"
                        else:
                            time_info = f"{int(abs(minutes_until))}m overdue"

                        with st.expander(f"ID: {txn['id']} - Challenge {txn['challenge_id']} → {dispatch_loc} ({time_info})"):
                            st.json(txn)
                else:
                    st.info("No pending transactions")
            else:
                st.error(f"Failed to fetch transactions: {response.status_code}")
        except requests.exceptions.RequestException as e:
            st.error(f"Cannot connect to API: {e}")

        st.markdown("---")

        # Completed Transactions Section
        st.header("✅ Completed Transactions")
        try:
            response = requests.get(COMPLETED_TRANSACTIONS_ENDPOINT, timeout=5)
            if response.status_code == 200:
                completed_txns = response.json()
                st.caption(f"Found {len(completed_txns)} completed transaction(s)")
                if completed_txns:
                    for txn in completed_txns:
                        status_emoji = "✅" if txn.get('transaction_status').lower() == 'success' else "❌"
                        dispatch_loc = txn.get('dispatch_location', 'N/A')
                        with st.expander(f"{status_emoji} ID: {txn['id']} - Challenge {txn['challenge_id']} → {dispatch_loc}"):
                            st.json(txn)
                else:
                    st.info("No completed transactions")
            else:
                st.error(f"Failed to fetch completed transactions: {response.status_code}")
        except requests.exceptions.RequestException as e:
            st.error(f"Cannot connect to API: {e}")

    else:  # Requests view
        # Requests Section
        st.header("📋 Requests")
        try:
            response = requests.get(REQUESTS_ENDPOINT, timeout=5)
            if response.status_code == 200:
                pending_requests = response.json()
                st.caption(f"Found {len(pending_requests)} pending request(s)")
                if pending_requests:
                    for req in pending_requests:
                        req_id = req.get('id', 'N/A')
                        challenge_id = req.get('challenge_id', 'N/A')
                        req_type = req.get('type_of_request', {}).get('type', 'Unknown')

                        deadline = req.get('deadline')
                        if deadline:
                            current_time_ms = int(datetime.now().timestamp() * 1000)
                            minutes_until = (deadline - current_time_ms) / 1000 / 60
                            if minutes_until > 0:
                                time_info = f"⏰ {int(minutes_until)}m left"
                            else:
                                time_info = f"⏰ {int(abs(minutes_until))}m overdue"
                        else:
                            time_info = "⏰ No deadline"

                        with st.expander(f"ID: {req_id} - Challenge {challenge_id} - {req_type} ({time_info})"):
                            st.json(req)
                else:
                    st.info("No pending requests")
            else:
                st.error(f"Failed to fetch requests: {response.status_code}")
        except requests.exceptions.RequestException as e:
            st.error(f"Cannot connect to API: {e}")

        st.markdown("---")

        # Completed Requests Section
        st.header("✅ Completed Requests")
        try:
            response = requests.get(COMPLETED_REQUESTS_ENDPOINT, timeout=5)
            if response.status_code == 200:
                completed_reqs = response.json()
                st.caption(f"Found {len(completed_reqs)} completed request(s)")
                if completed_reqs:
                    for req in completed_reqs:
                        req_id = req.get('id', 'N/A')
                        challenge_id = req.get('challenge_id', 'N/A')
                        req_type = req.get('type_of_request', {}).get('type', 'Unknown')
                        status = req.get('request_status', 'unknown').lower()

                        # Status emoji based on request status
                        if status == 'correct':
                            status_emoji = "✅"
                        elif status == 'partial_correct':
                            status_emoji = "⚠️"
                        elif status == 'incorrect':
                            status_emoji = "❌"
                        else:
                            status_emoji = "❓"

                        with st.expander(f"{status_emoji} ID: {req_id} - Challenge {challenge_id} - {req_type}"):
                            st.json(req)
                else:
                    st.info("No completed requests")
            else:
                st.error(f"Failed to fetch completed requests: {response.status_code}")
        except requests.exceptions.RequestException as e:
            st.error(f"Cannot connect to API: {e}")

    # Auto-refresh indicator
    st.markdown("---")
    st.caption("🔄 Auto-refreshing every 10s")

    # Danger zone - Clear all data
    st.markdown("---")
    st.subheader("⚠️ Danger Zone")

    # Initialize confirmation state
    if 'confirm_clear_all' not in st.session_state:
        st.session_state.confirm_clear_all = False

    if not st.session_state.confirm_clear_all:
        if st.button("🗑️ Clear All Data", use_container_width=True, type="secondary"):
            st.session_state.confirm_clear_all = True
            st.rerun()
    else:
        st.warning("⚠️ This will DELETE ALL challenges, transactions, and requests!")
        col1, col2 = st.columns(2)

        with col1:
            if st.button("✅ Confirm", use_container_width=True, type="primary"):
                # Call all destroy endpoints
                try:
                    with st.spinner("Deleting all data..."):
                        # Delete all challenges
                        response1 = requests.delete(CHALLENGES_ENDPOINT, timeout=5)
                        # Delete all transactions
                        response2 = requests.delete(TRANSACTIONS_ENDPOINT, timeout=5)
                        # Delete all completed transactions
                        response3 = requests.delete(COMPLETED_TRANSACTIONS_ENDPOINT, timeout=5)
                        # Delete all requests
                        response4 = requests.delete(REQUESTS_ENDPOINT, timeout=5)
                        # Delete all completed requests
                        response5 = requests.delete(COMPLETED_REQUESTS_ENDPOINT, timeout=5)

                        if all(r.status_code in [200, 204] for r in [response1, response2, response3, response4, response5]):
                            st.success("✅ All data cleared successfully!")
                        else:
                            st.error("❌ Some deletions failed. Check the backend logs.")

                except requests.exceptions.RequestException as e:
                    st.error(f"❌ Error connecting to API: {e}")

                st.session_state.confirm_clear_all = False
                time.sleep(1)  # Brief pause to show the message
                st.rerun()

        with col2:
            if st.button("❌ Cancel", use_container_width=True):
                st.session_state.confirm_clear_all = False
                st.rerun()

# Main form
st.header("Create New Challenge")

# File picker section (outside form since buttons can't be in forms)
st.subheader("📁 Select Dataset")
file_col1, file_col2 = st.columns([3, 1])

# Initialize session state for file path
if 'selected_file_path' not in st.session_state:
    st.session_state.selected_file_path = "/home/cicero/ppipe/py_modules/tests/test_data/iris.csv"

with file_col1:
    manual_path = st.text_input(
        "Dataset Path*",
        value=st.session_state.selected_file_path,
        placeholder="e.g., /home/user/data.csv or s3://bucket/data.csv",
        help="Local file path or S3 path to the dataset"
    )
    # Update session state if manually typed
    if manual_path != st.session_state.selected_file_path:
        st.session_state.selected_file_path = manual_path

with file_col2:
    st.write("")  # Spacer for alignment
    if st.button("📁 Browse", use_container_width=True):
        selected_path = browse_file()
        if selected_path:
            st.session_state.selected_file_path = selected_path
            st.rerun()

init_dataset_location = st.session_state.selected_file_path

st.markdown("---")

with st.form("challenge_form"):
    # Basic Information
    st.subheader("📋 Basic Information")
    col1, col2 = st.columns(2)

    with col1:
        challenge_name = st.text_input(
            "Challenge Name*",
            placeholder="e.g., iris-classification",
            help="Unique identifier for this challenge"
        )

        st.info(f"📄 Dataset: {init_dataset_location if init_dataset_location else 'Not selected'}")

        init_dataset_rows = st.number_input(
            "Total Dataset Rows*",
            min_value=1,
            value=150,
            help="Total number of rows in the dataset"
        )

    with col2:
        init_dataset_name = st.text_input(
            "Dataset Name",
            placeholder="e.g., Iris Dataset",
            help="Optional display name for the dataset"
        )
        init_dataset_description = st.text_area(
            "Dataset Description",
            placeholder="e.g., Classic iris classification dataset...",
            help="Optional description of what the dataset contains"
        )

    # Dispatch Configuration
    st.subheader("🚀 Dispatch Configuration")
    col1, col2 = st.columns(2)

    with col1:
        dispatches_to = st.multiselect(
            "Dispatch Targets*",
            options=["S3", "Drive"],
            default=["S3"],
            help="Where to dispatch the data releases"
        )

    with col2:
        dispatch_count = len(dispatches_to) if dispatches_to else 1
        st.info(f"Selected {dispatch_count} dispatch target(s)")

    # Release Schedule
    st.subheader("📅 Release Schedule")
    col1, col2, col3 = st.columns(3)

    with col1:
        time_offset_minutes = st.number_input(
            "First Release (minutes from now)",
            min_value=0,
            value=5,
            help="When to make the first data release"
        )
        time_of_first_release = int((datetime.now() + timedelta(minutes=time_offset_minutes)).timestamp() * 1000)

    with col2:
        time_between_releases = st.number_input(
            "Time Between Releases (seconds)*",
            min_value=1,
            value=3600,
            help="Interval between data releases"
        )

    with col3:
        num_releases = st.number_input(
            "Number of Releases*",
            min_value=1,
            value=3,
            help="How many data releases to schedule"
        )

    # Release Proportions
    st.write("**Release Proportions***")

    col_caption, col_balance = st.columns([3, 1])
    with col_caption:
        st.caption("Define what proportion of the data to release at each stage (must sum to 1.0)")
    with col_balance:
        auto_balance = st.checkbox("⚖️ Auto-balance", value=True, help="Automatically distribute proportions equally")

    proportion_cols = st.columns(min(num_releases, 5))
    release_proportions = []

    for i in range(num_releases):
        col_idx = i % 5
        with proportion_cols[col_idx]:
            if auto_balance:
                # Calculate equal proportions
                equal_val = round(1.0 / num_releases, 3)
                prop = st.number_input(
                    f"Release {i+1}",
                    min_value=0.0,
                    max_value=1.0,
                    value=equal_val,
                    step=0.01,
                    key=f"prop_{i}",
                    disabled=True,
                    help="Auto-balanced to equal proportions"
                )
                release_proportions.append(equal_val)
            else:
                # Manual input
                default_val = round(1.0 / num_releases, 3)
                prop = st.number_input(
                    f"Release {i+1}",
                    min_value=0.0,
                    max_value=1.0,
                    value=default_val,
                    step=0.01,
                    key=f"prop_{i}"
                )
                release_proportions.append(prop)

    total_proportion = sum(release_proportions)
    if abs(total_proportion - 1.0) > 0.01:
        st.warning(f"⚠️ Proportions sum to {total_proportion:.3f}, should be 1.0")
    else:
        st.success(f"✅ Proportions sum to {total_proportion:.3f}")

    # Challenge Options
    st.subheader("⚙️ Challenge Options (Optional)")
    with st.expander("Configure Advanced Options"):
        makes_requests_on_transaction_push = st.checkbox(
            "Make requests on transaction push",
            help="Automatically create requests when data is pushed"
        )
        makes_requests_randomly = st.checkbox(
            "Make requests randomly",
            help="Create requests at random intervals"
        )

        if makes_requests_randomly:
            col1, col2 = st.columns(2)
            with col1:
                min_time_between_requests = st.number_input(
                    "Min Time Between Requests (seconds)",
                    min_value=1,
                    value=60
                )
            with col2:
                max_time_between_requests = st.number_input(
                    "Max Time Between Requests (seconds)",
                    min_value=1,
                    value=300
                )
        else:
            min_time_between_requests = None
            max_time_between_requests = None

        requests_deadline = st.number_input(
            "Request Deadline (seconds, 0 for none)",
            min_value=0,
            value=0
        )

        validate_request_immediately = st.checkbox(
            "Validate requests immediately on answer",
            value=True
        )
        allow_retries = st.checkbox("Allow retries on requests")
        return_completed_on_answer = st.checkbox(
            "Return completed request on student answer",
            value=True
        )

    # Build challenge options
    challenge_options = {}
    if makes_requests_on_transaction_push:
        challenge_options["makes_requests_on_transaction_push"] = True
    if makes_requests_randomly:
        challenge_options["makes_requests_randomly"] = True
        challenge_options["min_time_between_requests"] = min_time_between_requests
        challenge_options["max_time_between_requests"] = max_time_between_requests
    if requests_deadline > 0:
        challenge_options["requests_deadline"] = requests_deadline
    if validate_request_immediately:
        challenge_options["validate_request_immediately_on_answer"] = True
    if allow_retries:
        challenge_options["allow_retries_on_request"] = True
    if return_completed_on_answer:
        challenge_options["return_completed_request_on_student_answer"] = True

    # Submit button
    col1, col2, col3 = st.columns([1, 1, 1])
    with col2:
        submit_button = st.form_submit_button("Create Challenge", use_container_width=True)

# Handle form submission
if submit_button:
    # Validation
    if not challenge_name:
        st.error("❌ Challenge name is required")
    elif '-' in challenge_name:
        st.error("❌ Challenge name cannot contain dashes (-)")
    elif not init_dataset_location:
        st.error("❌ Dataset location is required")
    elif not dispatches_to:
        st.error("❌ At least one dispatch target is required")
    elif abs(sum(release_proportions) - 1.0) > 0.01:
        st.error("❌ Release proportions must sum to 1.0")
    else:
        # Build the challenge payload
        challenge_payload = {
            "challenge_name": challenge_name,
            "init_dataset_location": init_dataset_location,
            "init_dataset_rows": init_dataset_rows,
            "init_dataset_name": init_dataset_name if init_dataset_name else None,
            "init_dataset_description": init_dataset_description if init_dataset_description else None,
            "dispatches_to": dispatches_to,
            "time_of_first_release": time_of_first_release,
            "release_proportions": release_proportions,
            "time_between_releases": time_between_releases * 1000,  # Convert seconds to milliseconds
            "access_bindings": None,
            "challenge_options": challenge_options if challenge_options else {}
        }

        # Display the payload
        with st.expander("📦 View Request Payload"):
            st.json(challenge_payload)

        # Send the request
        try:
            with st.spinner("Creating challenge..."):
                response = requests.post(
                    CHALLENGES_ENDPOINT,
                    json=challenge_payload,
                    headers={"Content-Type": "application/json"},
                    timeout=10
                )

            if response.status_code == 200:
                st.success("✅ Challenge created successfully!")
                result = response.json()

                with st.expander("📊 View Response"):
                    st.json(result)

                # Offer to create another
                if st.button("Create Another Challenge"):
                    st.rerun()
            else:
                st.error(f"❌ Failed to create challenge: {response.status_code}")
                st.code(response.text)

        except requests.exceptions.RequestException as e:
            st.error(f"❌ Error connecting to API: {e}")
            st.info("Make sure the Rust backend is running on http://localhost:8000")

# Footer
st.markdown("---")
st.caption("Made with absolutely no ❤️ by a cold-hearted, dumbass LLM, using Streamlit")
