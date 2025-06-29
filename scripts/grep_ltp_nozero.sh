#!/bin/bash

logfile="$1"

if [ -z "$logfile" ]; then
    echo "Usage: $0 <logfile>"
    exit 1
fi

in_case=0
case_name=""
passed_count=0

while IFS= read -r line; do
    if [[ "$line" =~ ^RUN\ LTP\ CASE\ ([^[:space:]]+) ]]; then
        case_name="${BASH_REMATCH[1]}"
        in_case=1
        continue
    fi

    if [[ $in_case -eq 1 && "$line" =~ ^passed[[:space:]]+([0-9]+) ]]; then
        passed_count="${BASH_REMATCH[1]}"
        if [[ "$passed_count" -ne 0 ]]; then
            echo "$case_name"
        fi
        in_case=0
    fi
done < "$logfile"
