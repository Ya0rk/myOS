#!/bin/bash

logfile="$1"

if [ -z "$logfile" ]; then
    echo "Usage: $0 <logfile>"
    exit 1
fi

# 设置状态标志
in_case=0
case_name=""

while IFS= read -r line; do
    if [[ "$line" =~ ^RUN\ LTP\ CASE\ ([^[:space:]]+) ]]; then
        case_name="${BASH_REMATCH[1]}"
        echo "RUN LTP CASE $case_name"
        in_case=1
        continue
    fi

    if [[ $in_case -eq 1 && "$line" =~ ^Summary: ]]; then
        echo "$line"
        # 读取接下来的5行（passed, failed, broken, skipped, warnings）
        for i in {1..5}; do
            read -r stat_line
            echo "$stat_line"
        done
        in_case=0
    fi

    if [[ "$line" =~ ^FAIL\ LTP\ CASE\ ([^[:space:]]+) ]]; then
        echo "$line"
    fi
done < "$logfile"
