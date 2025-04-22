# 20250422 MERGE REPORT
author: Lu Jiading

# 主要内容
1. 把config移入hal中，统一hal分支和main分支逻辑
2. 设置新的设备驱动，可以解析设备树，实现pci

## 1.

config包放入hal的各个机器的包中，直接调用hal::config可使用

注意到应当在龙芯和riscv中都实现这个参数

## 2.

可以实现解析设备树功能，应该是可以识别块设备，网卡，串口，等

(已验证块设备，其他设备的验证代码也在)

具体在os/src/driver/virtio_driver/probe.rs实现

原有的os/src/driver/virtio文件夹废弃

可以实现pci设备的注册和使用，已验证块设备

但是与现有的页表设置冲突，所以可以只在LA中使用

具体在os/src/driver/virtio_driver/pci.rs实现

在os/src/driver/virtio_driver/probe.rs的probe函数中调用相关函数

注意到现有的块设备初始化工作在os/src/driver/mod.rs中完成

其地址依赖于probe函数的调用，应当在main函数中调用probe后再使用块设备

注意到main函数的第二个参数莫名其妙地不是设备树的起点地址，所以采用硬编码（该编码在qemu的rustsbi的输出中获得）