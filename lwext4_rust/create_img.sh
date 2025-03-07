# # 格式化为 ext4
# sudo mkfs.ext4 ext4.img
# sudo chmod 777 ext4.img

# # 先删除以前存在的文件夹
# sudo rm -rf ../lwext4_rust/fs
# # 创建文件夹用于挂载镜像文件
# sudo mkdir ../lwext4_rust/fs

# # 挂载镜像文件
# sudo mount ../lwext4_rust/ext4.img ../lwext4_rust/fs

# # 复制基本的用户程序
# sudo cp ../user/target/riscv64gc-unknown-none-elf/release/initproc ../lwext4_rust/fs/
# sudo cp ../user/target/riscv64gc-unknown-none-elf/release/user_shell ../lwext4_rust/fs/
# # sudo cp ../user/target/riscv64gc-unknown-none-elf/release/cat_filea ../lwext4_rust/fs/

# # 复制测试用例
# sudo cp -r ../testcase/24/* ../lwext4_rust/fs/

# sudo umount ../lwext4_rust/fs
# sudo rmdir ../lwext4_rust/fs

# 制作一个全0的镜像文件
dd if=/dev/zero of=ext4.img bs=4M count=128

DIR=lwext4_rust

# 格式化为 ext4
sudo mkfs.ext4 ext4.img
sudo chmod 777 ext4.img

sudo mkdir ../${DIR}/fs 
sudo mount ../${DIR}/ext4.img ../${DIR}/fs 

# 复制基本的用户程序
sudo cp ../user/target/riscv64gc-unknown-none-elf/release/initproc ../lwext4_rust/fs/
sudo cp ../user/target/riscv64gc-unknown-none-elf/release/user_shell ../lwext4_rust/fs/
# sudo cp ../user/target/riscv64gc-unknown-none-elf/release/cat_filea ../lwext4_rust/fs/

# 复制测试用例
sudo cp -r ../testcase/24/* ../lwext4_rust/fs/

sudo umount ../${DIR}/fs 
sudo rmdir ../${DIR}/fs