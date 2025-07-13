#!/bin/bash

# 检查是否提供了关键字参数（如 feat）
if [ $# -eq 0 ]; then
    echo "Usage: $0 <keyword>"
    echo "Example: $0 feat"
    exit 1
fi

KEYWORD=$1
DATE=$(date +"%Y%m%d")
TIMESTAMP=$(date +"%H%M%S")
LOG_DIR="../log/${DATE}"          # 按日期分目录
LOG_FILE="${LOG_DIR}/${KEYWORD}-${TIMESTAMP}.log"

# 创建日志目录（如果不存在）
mkdir -p "$LOG_DIR"

# 执行 make run，同时打印到终端并保存到日志文件（过滤 ANSI 转义字符）
make run | tee >(sed -r "s/\x1B\[([0-9]{1,3}(;[0-9]{1,2})?)?[mGK]//g" > "$LOG_FILE")

# 输出日志路径（可选）
echo "Log saved to: $LOG_FILE"
