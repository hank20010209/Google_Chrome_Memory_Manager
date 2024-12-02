from flask import Flask, jsonify, request, abort
import os
import json
from jsonpath_ng.ext import parse

app = Flask(__name__)

LOG_FILE = os.getcwd() + "/.." + "/memory_management_for_chrome/" + "output.json"

def read_log_file():
    # print("Grafana: {}\n", LOG_FILE)
    if not os.path.exists(LOG_FILE):
        return {"error": "log.json file not found"}
    try:
        with open(LOG_FILE, "r", encoding="utf-8") as file:
            data_dict = json.load(file)
            # print("data_dict: ",data_dict)
            return data_dict
    except json.JSONDecodeError:
        return {"error": "Invalid JSON format in log.json"}

def query_json(json_data, query):
    try:
        jsonpath_expr = parse(query)
        val_list = [match.value for match in jsonpath_expr.find(json_data)]
        return val_list
    except Exception as e:
        return {"error": f"Invalid JSONPath query: {str(e)}"}

@app.route("/", methods=["GET"])
def handle_request():
    log_data = read_log_file()
    return log_data

if __name__ == "__main__":
    app.run(debug=True, host="0.0.0.0", port=5000)
