# 制作一个全0的镜像文件
dd if=/dev/zero of=fat32.img bs=3M count=16

# 格式化为 fat32
sudo mkfs.vfat -F 32 fat32.img
sudo chmod 777 fat32.img

# 创建文件夹用于挂载镜像文件
sudo mkdir ../simple-fat32/fs

# 挂载镜像文件
sudo mount ../simple-fat32/fat32.img ../simple-fat32/fs

# 复制基本的用户程序
sudo cp ../user/target/riscv64gc-unknown-none-elf/release/initproc ../simple-fat32/fs/
sudo cp ../user/target/riscv64gc-unknown-none-elf/release/user_shell ../simple-fat32/fs/
sudo cp ../user/target/riscv64gc-unknown-none-elf/release/cat_filea ../simple-fat32/fs/

# 复制测试用例
sudo cp -r ../testcase/24/* ../simple-fat32/fs/

sudo umount ../simple-fat32/fs
sudo rmdir ../simple-fat32/fs