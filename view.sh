#!/bin/bash

# 设置日志目录
LOG_DIR="log"

# 检查日志目录是否存在
if [ ! -d "$LOG_DIR" ]; then
    echo "错误：日志目录 '$LOG_DIR' 不存在！"
    exit 1
fi

# 获取最新的日期目录（按数字排序）
LATEST_DATE_DIR=$(ls "$LOG_DIR" | grep -E '^[0-9]{8}$' | sort -nr | head -1)

if [ -z "$LATEST_DATE_DIR" ]; then
    echo "错误：未找到有效的日期日志目录！"
    exit 1
fi

# 获取该日期下最新的日志文件（按修改时间排序）
LATEST_LOG_FILE=$(ls -t "$LOG_DIR/$LATEST_DATE_DIR" | head -1)

if [ -z "$LATEST_LOG_FILE" ]; then
    echo "错误：未找到日志文件！"
    exit 1
fi

# 完整的日志文件路径
FULL_LOG_PATH="$LOG_DIR/$LATEST_DATE_DIR/$LATEST_LOG_FILE"

# 使用vim查看日志
echo "正在打开最新的日志文件: $FULL_LOG_PATH"
vim "$FULL_LOG_PATH"
