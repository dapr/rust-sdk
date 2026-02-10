#!/usr/bin/env bash

# ------------------------------------------------------------
# Copyright 2021 The Dapr Authors
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#     http://www.apache.org/licenses/LICENSE-2.0
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
# ------------------------------------------------------------

# Protobuf generation
APPCALLBACK="appcallback"
COMMON="common"
DAPR="dapr"
RUNTIME="runtime"
RUNTIME_RELEASE_TAG="master"

# Path to store output
PROTO_PATH="proto/dapr/proto"

# Http request CLI
HTTP_REQUEST_CLI=curl

checkHttpRequestCLI() {
    if type "curl" > /dev/null; then
        HTTP_REQUEST_CLI=curl
    elif type "wget" > /dev/null; then
        HTTP_REQUEST_CLI=wget
    else
        echo "Either curl or wget is required"
        exit 1
    fi
}

checkJq() {
    if ! type "jq" > /dev/null; then
        echo "jq is required to parse GitHub API JSON. Please install jq and try again."
        exit 1
    fi
}

setRuntimeReleaseTag() {
    local OPTIND
    while getopts ":v:" opt; do
        case $opt in
            v)
                echo "Passed Runtime Release Tag is: $OPTARG" >&2
                RUNTIME_RELEASE_TAG=$OPTARG
                ;;
            \?)
                echo "Invalid option: -$OPTARG" >&2
                exit 1
                ;;
            :)
                echo "Option -$OPTARG requires an argument." >&2
                exit 1
                ;;
        esac
    done
}

downloadFile() {
    FOLDER_NAME=$1
    FILE_NAME=$2
    FILE_PATH="${PROTO_PATH}/${FOLDER_NAME}/v1"

    # URL for proto file
    PROTO_URL="https://raw.githubusercontent.com/dapr/dapr/${RUNTIME_RELEASE_TAG}/dapr/proto/${FOLDER_NAME}/v1/${FILE_NAME}.proto"

    mkdir -p "${FILE_PATH}"

    echo "Downloading $PROTO_URL ..."
    if [ "$HTTP_REQUEST_CLI" == "curl" ]; then
        (cd ${FILE_PATH} && curl -SsL "$PROTO_URL" -o "${FILE_NAME}.proto")
    else
        wget -q -O "${FILE_PATH}/${FILE_NAME}.proto" "$PROTO_URL"
    fi

    if [ ! -e "${FILE_PATH}/${FILE_NAME}.proto" ]; then
        echo "failed to download $PROTO_URL ..."
        ret_val=$FILE_NAME
        exit 1
    fi
}

downloadAllFromGitHubDir() {
    FOLDER_NAME=$1
    FILE_PATH="${PROTO_PATH}/${FOLDER_NAME}/v1"
    API_URL="https://api.github.com/repos/dapr/dapr/contents/dapr/proto/${FOLDER_NAME}/v1?ref=${RUNTIME_RELEASE_TAG}"

    echo "Fetching file list from $API_URL ..."
    if [ "$HTTP_REQUEST_CLI" == "curl" ]; then
        resp=$(curl -sS "$API_URL") || { echo "Failed to fetch file list from $API_URL"; exit 1; }
    else
        resp=$(wget -q -O - "$API_URL") || { echo "Failed to fetch file list from $API_URL"; exit 1; }
    fi

    proto_files=$(echo "$resp" | jq -r '.[] | select(.name | endswith(".proto")) | .name' || true)

    if [ -z "$proto_files" ]; then
        echo "No .proto files found in $API_URL"
        exit 1
    fi

    mkdir -p "${FILE_PATH}"

    for f in $proto_files; do
        base="${f%.proto}"
        echo "Fetching ${base}.proto"
        downloadFile "$FOLDER_NAME" "$base"
    done
}

fail_trap() {
    result=$?
    if [ $result != 0 ]; then
        echo "Failed to download proto files: $ret_val"
    fi
    exit $result
}

# -----------------------------------------------------------------------------
# main
# -----------------------------------------------------------------------------
trap "fail_trap" EXIT

checkHttpRequestCLI
checkJq
setRuntimeReleaseTag "$@"
downloadAllFromGitHubDir $COMMON
downloadAllFromGitHubDir $RUNTIME
