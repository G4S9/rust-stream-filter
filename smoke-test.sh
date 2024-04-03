#!/usr/bin/env bash

BASE_URL='https://9gkkmmts3c.execute-api.eu-central-1.amazonaws.com/dev'

###
step1_response=$(curl -s -X POST "${BASE_URL}/phonenumbers" \
    -H "Content-Type: application/json" \
    -d '{"fileName": "phone_numbers.txt"}')
echo "Step 1 Response:"
echo "${step1_response}" | jq
taskId=$(echo "${step1_response}" | jq -r '.taskId')
if [[ -z "${taskId}" ]]; then
  echo "No Task ID found, cannot proceed."
  exit 1
fi

###
url=$(echo "${step1_response}" | jq -r '.url')
signedHeaders=$(echo "${step1_response}" | jq -r '.signedHeaders | to_entries | map("\(.key):\(.value)") | .[]')
file_path='./artifacts/phone_numbers.txt'
echo "Step 2 Response:"
curl -s -X PUT "${url}" -H "${signedHeaders}" --data-binary @"${file_path}"

###
step3_response=$(curl -s -X GET "${BASE_URL}/phonenumbers")
echo "Step 3 Response:"
echo "${step3_response}" | jq

###
step4_response=$(curl -s -X GET "${BASE_URL}/phonenumbers/${taskId}")
echo "Step 4 Response:"
echo "${step4_response}" | jq -r '.'

###
url_for_step5=$(echo "${step4_response}" | jq -r '.url')
step5_response=$(curl -s -X GET "${url_for_step5}")
echo "Step 5 Response First 10 lines:"
echo "${step5_response}" | head -n 10

###
echo "Step 6 Response: $(curl -s -X DELETE "${BASE_URL}/phonenumbers/${taskId}")"

###
echo "Step 7 Response: $(curl -s -X GET "${BASE_URL}/phonenumbers" | jq)"
