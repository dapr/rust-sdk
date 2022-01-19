#!/bin/bash

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

# Path to store output
PROTO_PATH="dapr/proto"

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

downloadFile() {
    FOLDER_NAME=$1
    FILE_NAME=$2
    FILE_PATH="${PROTO_PATH}/${FOLDER_NAME}/v1"

    # URL for proto file
    PROTO_URL="https://raw.githubusercontent.com/dapr/dapr/master/dapr/proto/${FOLDER_NAME}/v1/${FILE_NAME}.proto"

    mkdir -p "${FILE_PATH}"

    echo "Downloading $PROTO_URL ..."
    if [ "$HTTP_REQUEST_CLI" == "curl" ]; then
        (cd ${FILE_PATH} && curl -SsL "$PROTO_URL" -o "${FILE_NAME}.proto")
    else
        wget -q -P "$PROTO_URL" "${FILE_PATH}/${FILE_NAME}.proto"
    fi

    if [ ! -e "${FILE_PATH}/${FILE_NAME}.proto" ]; then
        echo "failed to download $PROTO_URL ..."
        ret_val=$FILE_NAME
        exit 1
    fi
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
downloadFile $COMMON $COMMON
downloadFile $RUNTIME $DAPR
downloadFile $RUNTIME $APPCALLBACK
