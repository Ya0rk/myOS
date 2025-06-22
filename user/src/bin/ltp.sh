#!/bin/sh

echo "#### OS COMP TEST GROUP START ltp-musl ####"

# Define target directory
target_dir="/musl/ltp/testcases/bin"

# List of test cases (no array, using space-separated string)
ltp_cases="chown02 abs01 accept01"

for case in $ltp_cases; do
  file="$target_dir/$case"

  if [ -f "$file" ]; then
    echo "RUN LTP CASE $case"

    "$file"
    ret=$?

    # echo "FAIL LTP CASE $case : $ret"
#   else
#     echo "SKIP LTP CASE $case : not found"
  fi
done

echo "#### OS COMP TEST GROUP END ltp-musl ####"