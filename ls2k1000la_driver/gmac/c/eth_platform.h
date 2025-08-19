#ifndef __LS2K_ETH_PLATFORM_H__
#define __LS2K_ETH_PLATFORM_H__

#include <stdbool.h>
#include <stdint.h>

// 替换成平台自定义的printf
int eth_printf(const char *fmt, ...);

// aligned malloc
uint64_t eth_malloc_align(uint64_t size, uint32_t align);

// sync all dcache data
void eth_sync_dcache();

// cached虚拟地址转换为物理地址
// dma仅接受32位的物理地址
uint32_t eth_virt_to_phys(uint64_t va);

// 物理地址转换为cached虚拟地址
uint64_t eth_phys_to_virt(uint32_t pa);

// OS可能会有自定义格式的存储单元
// p是OS传递给驱动的存储单元
// buffer是驱动分配的dma内存
// 将p的数据copy到buffer中
// 返回数据总长度
uint32_t eth_handle_tx_buffer(uint64_t p, uint64_t buffer);

// buffer是接收到的数据，length是字节数
// OS需要分配内存，memcpy接收到的数据，并将地址返回
uint64_t eth_handle_rx_buffer(uint64_t buffer, uint32_t length);

// 中断isr通知OS可以调用rx函数
void eth_rx_ready(struct net_device *gmacdev);

// 中断isr通知链路状态发生变化，status - 1表示up，0表示down
// 链路目前仅支持1000Mbps duplex
void eth_update_linkstate(struct net_device *gmacdev, uint32_t status);

// OS注册中断，中断号为12
void eth_isr_install();

#endif // __LS2K_ETH_PLATFORM_H__
