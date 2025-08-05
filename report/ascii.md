# ASCII控制字符、组合键及其在终端中的转发机制

## 1. 核心概念澄清

在探讨组合键之前，必须澄清几个核心概念：

1.  **ASCII码的局限性**: ASCII码是一个7位的编码标准（0-127），它只定义了128个字符。其中，0-31和127是**控制字符 (Control Characters)**，32-126是**可打印字符 (Printable Characters)**。ASCII本身**没有**为`Alt`, `Shift`, `Ctrl`这些修饰键，或者`F1`-`F12`功能键，以及方向键等定义任何编码。

2.  **组合键的产生**: 当你在键盘上按下`Ctrl+C`时，键盘**不会**发送一个代表“Ctrl+C”的特殊码。实际发生的是：
    *   终端（或操作系统键盘驱动）检测到`Ctrl`键被按下的同时，`C`键也被按下了。
    *   它根据一个规则，将这个组合键**翻译**成一个ASCII控制字符。规则通常是：取字母`C`的ASCII码，将其第6位和第7位清零（或者说，取其值的低5位）。
    *   `'C'`的ASCII码是67 (二进制 `01000011`)。将其高位清零后得到3 (二进制 `00000011`)。
    *   数字3在ASCII表中对应的控制字符是**ETX (End of Text)**。因此，按下`Ctrl+C`，终端实际上发送的是值为**3**的单个字节。

3.  **`Alt`键和`Shift`键**: 
    *   **`Shift`**: `Shift`键的作用是在键盘驱动或终端层面，直接改变要发送的字符。按下`Shift+A`，发送的就是`'A'`(65)，而不是`'a'`(97)。它不产生控制字符。
    *   **`Alt`**: `Alt`键（有时等同于`Meta`键）的行为比较复杂。在很多终端中，按下`Alt+<key>`会发送一个**ESC序列**，即先发送`ESC`字符（ASCII 27），然后再发送`<key>`字符。例如，`Alt+C`可能会被发送为两个字节序列：`27` `99` (`ESC` `c`)。

4.  **`Ctrl+D`的特殊性**: `Ctrl+D`产生的ASCII码是4 (EOT, End of Transmission)。在终端的规范模式下，当行缓冲区为空时，接收到`EOT`字符会被TTY驱动解释为**文件结束 (End of File, EOF)**，这会导致`read()`系统调用返回0，从而让`bash`等程序认为输入已结束而退出。

## 2. QEMU `-nographic` 和 MobaXterm 的转发机制

*   **QEMU `-nographic`**:
    *   在这种模式下，QEMU将宿主机的**标准输入 (stdin)** 直接连接到虚拟机的第一个串口。
    *   这意味着，**宿主机终端**处理完组合键并生成一个字节（或字节序列）后，这个结果会被写入`stdin`，QEMU会原封不动地将它转发给虚拟机的串口。
    *   **结论**: QEMU `-nographic`会转发**所有**宿主机终端传递给它的字节。它本身不做任何解释或过滤。一个`Ctrl+C`在宿主机终端上被翻译成字节`3`，QEMU就将字节`3`发送给虚拟机。

*   **MobaXterm (SSH/Serial Client)**:
    *   MobaXterm作为一个功能丰富的终端模拟器，其行为与QEMU `-nographic`模式下的宿主机终端非常相似。
    *   它会捕获你的键盘输入，将组合键（如`Ctrl+C`）翻译成对应的ASCII控制字符（字节`3`），然后将这个字节通过网络（SSH）或物理串口线发送出去。
    *   **结论**: MobaXterm这类软件**会**转发所有标准的`Ctrl`组合键产生的控制字符，因为这是实现远程终端交互的基础。对于`Alt`键，它通常会配置为发送ESC序列。

**关键点**: 转发与否，主要取决于**前端的终端模拟器**（你的Linux Shell、MobaXterm、PuTTY等）是如何解释组合键的。一旦它们将组合键翻译成了一个或多个字节，QEMU和SSH协议等后端通道通常会忠实地传输这些字节。

## 3. ASCII控制字符与常见组合键列表

下表列出了ASCII码表中的0-31号控制字符，它们的含义，以及在大多数终端中生成它们的常见键盘组合。

| Dec | Hex | 缩写 | 名称 (Name) | 转义/Caret表示法 | 常见组合键 | 是否被QEMU/MobaXterm转发？ |
|:---:|:---:|:----:|:--------------------|:--------------------|:------------------|:-----------------------------|
| 0 | 00 | NUL | Null | `\0`, `^@` | `Ctrl+@` | 是 |
| 1 | 01 | SOH | Start of Heading | `^A` | `Ctrl+A` | 是 (常用于Shell行首) |
| 2 | 02 | STX | Start of Text | `^B` | `Ctrl+B` | 是 (常用于Shell左移光标) |
| 3 | 03 | ETX | End of Text | `^C` | `Ctrl+C` | **是** (被TTY解释为`INTR`信号) |
| 4 | 04 | EOT | End of Transmission | `^D` | `Ctrl+D` | **是** (被TTY解释为`EOF`) |
| 5 | 05 | ENQ | Enquiry | `^E` | `Ctrl+E` | 是 (常用于Shell行尾) |
| 6 | 06 | ACK | Acknowledge | `^F` | `Ctrl+F` | 是 (常用于Shell右移光标) |
| 7 | 07 | BEL | Bell | `\a`, `^G` | `Ctrl+G` | 是 (会使终端发出哔声) |
| 8 | 08 | BS | Backspace | `\b`, `^H` | `Backspace` or `Ctrl+H` | **是** (被TTY解释为`ERASE`) |
| 9 | 09 | HT | Horizontal Tab | `\t`, `^I` | `Tab` or `Ctrl+I` | **是** |
| 10 | 0A | LF | Line Feed | `\n`, `^J` | `Enter` or `Ctrl+J` | **是** (通常是`EOL`) |
| 11 | 0B | VT | Vertical Tab | `\v`, `^K` | `Ctrl+K` | 是 (常用于Shell剪切到行尾) |
| 12 | 0C | FF | Form Feed | `\f`, `^L` | `Ctrl+L` | 是 (常用于Shell清屏) |
| 13 | 0D | CR | Carriage Return | `\r`, `^M` | `Enter` or `Ctrl+M` | **是** (通常与LF等效或一起发送) |
| 14 | 0E | SO | Shift Out | `^N` | `Ctrl+N` | 是 (常用于Shell下一条历史) |
| 15 | 0F | SI | Shift In | `^O` | `Ctrl+O` | 是 |
| 16 | 10 | DLE | Data Link Escape | `^P` | `Ctrl+P` | 是 (常用于Shell上一条历史) |
| 17 | 11 | DC1 | Device Control 1 (XON) | `^Q` | `Ctrl+Q` | 是 (用于软件流控，恢复传输) |
| 18 | 12 | DC2 | Device Control 2 | `^R` | `Ctrl+R` | 是 (常用于Shell反向搜索历史) |
| 19 | 13 | DC3 | Device Control 3 (XOFF) | `^S` | `Ctrl+S` | 是 (用于软件流控，暂停传输) |
| 20 | 14 | DC4 | Device Control 4 | `^T` | `Ctrl+T` | 是 (常用于Shell交换字符) |
| 21 | 15 | NAK | Negative Acknowledge | `^U` | `Ctrl+U` | 是 (常用于Shell剪切整行) |
| 22 | 16 | SYN | Synchronous Idle | `^V` | `Ctrl+V` | 是 (常用于Shell“字面”输入) |
| 23 | 17 | ETB | End of Transmission Block | `^W` | `Ctrl+W` | 是 (常用于Shell剪切单词) |
| 24 | 18 | CAN | Cancel | `^X` | `Ctrl+X` | 是 |
| 25 | 19 | EM | End of Medium | `^Y` | `Ctrl+Y` | 是 (常用于Shell粘贴) |
| 26 | 1A | SUB | Substitute | `^Z` | `Ctrl+Z` | **是** (被TTY解释为`SUSP`信号) |
| 27 | 1B | ESC | Escape | `\e`, `^[` | `Esc` | **是** (用于`Alt`组合键和终端转义序列) |
| 28 | 1C | FS | File Separator | `^\` | `Ctrl+\` | **是** (被TTY解释为`QUIT`信号) |
| 29 | 1D | GS | Group Separator | `^]` | `Ctrl+]` | 是 |
| 30 | 1E | RS | Record Separator | `^^` | `Ctrl+^` | 是 |
| 31 | 1F | US | Unit Separator | `^_` | `Ctrl+_` | 是 |
| 127 | 7F | DEL | Delete | `^?` | `Delete` or `Ctrl+?` | **是** (有时也作为`ERASE`) |

**注意**: `Alt+Shift+S`这样的组合键**没有**对应的ASCII码。它的行为完全由**应用程序**（如桌面环境、编辑器）的快捷键设置来定义，它不会被翻译成一个简单的控制字符发送到终端。

## 4. 信息来源

1.  **ASCII码表和控制字符**: 这是计算机科学的基础知识，任何一本计算机组成原理教材或在线编码参考网站都有详细列表。一个很好的在线资源是 `https://www.asciitable.com/`。
2.  **组合键的翻译**: 这个行为是Unix/Linux终端的传统标准。相关信息可以在`termios`的手册页（`man termios`）中找到，其中描述了`c_cc`数组中各个控制字符（`INTR`, `EOF`, `ERASE`等）的默认值和作用。
3.  **QEMU转发机制**: QEMU的官方文档中关于字符设备的部分（`qemu-doc.html`）描述了`-serial`, `-chardev`等参数的行为。`stdio`后端的行为就是直接读写标准I/O，这隐含了它不解析内容的特性。
    *   **源码位置**: QEMU源码中处理字符设备和终端的部分，如`ui/console.c`和`chardev/char-stdio.c`，可以证实其作为数据通道的转发行为。
4.  **终端模拟器行为**: 这是终端模拟器（如xterm, gnome-terminal, MobaXterm）的标准功能。它们的文档或源代码会详细说明它们如何处理键盘输入和转义序列。
