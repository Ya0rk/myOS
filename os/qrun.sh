$ cat os/qrun.sh
#!/bin/bash

# 检查是否提供了关键字参数和架构参数
if [ $# -lt 1 ]; then
    echo "Usage: $0 <keyword> [arch]"
    echo "Example: $0 feat loongarch64"
    echo "Example: $0 debug riscv64"
    echo "Supported arch: lriscv64, loongarch64, "
    exit 1
fi

KEYWORD=$1
ARCH=${2:-riscv64}  # 默认架构为 loongarch64

# 验证架构参数
if [[ "$ARCH" != "loongarch64" && "$ARCH" != "riscv64" ]]; then
    echo "Error: Unsupported architecture '$ARCH'"
    echo "Supported architectures: loongarch64, riscv64"
    exit 1
fi

DATE=$(date +"%Y%m%d")
TIMESTAMP=$(date +"%H%M%S")
LOG_DIR="../log/${DATE}"          # 按日期分目录
LOG_FILE="${LOG_DIR}/${KEYWORD}-${ARCH}-${TIMESTAMP}.log"

# 创建日志目录（如果不存在）
mkdir -p "$LOG_DIR"

# 执行 make run，同时打印到终端并保存到日志文件（过滤 ANSI 转义字符）
echo "Running with arch=$ARCH..."
make run ARCH=$ARCH | tee >(sed -r "s/\x1B\[([0-9]{1,3}(;[0-9]{1,2})?)?[mGK]//g" > "$LOG_FILE")

# 输出日志路径
echo "Log saved to: $LOG_FILE"
