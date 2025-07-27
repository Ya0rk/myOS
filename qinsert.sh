#!/bin/bash

set +e  # 遇到错误不退出

echo ">> Step 1: Unmounting mount_point if mounted..."
sudo umount mount_point
echo "   -> umount exit code: $?"

echo ">> Step 2: Creating mount_point directory..."
sudo mkdir -p mount_point
echo "   -> mkdir exit code: $?"

echo ">> Step 3: Mounting sdcard-rv.img to mount_point..."
sudo mount sdcard-rv.img ./mount_point/
echo "   -> mount exit code: $?"

echo ">> Step 4: Copying file \$1 to ./mount_point/musl/..."
sudo cp -f "$1" ./mount_point/musl/
echo "   -> cp exit code: $?"

echo ">> Step 5: Unmounting mount_point..."
sudo umount mount_point
echo "   -> umount exit code: $?"

echo ">> Script finished."

