# Linux Termios 标志位详解

`termios` 是在 POSIX 兼容系统中用于控制终端（TTY）设备的一套标准接口。它允许程序精细地控制终端的输入、输出、硬件和行规程行为。这些控制是通过设置 `termios` 结构体中的一系列标志位来实现的。

本文档将详细解释 `c_iflag`、`c_oflag`、`c_cflag` 和 `c_lflag` 这四个字段中的主要标志位。

---

## `c_iflag` - 输入模式标志 (Input Flags)

这些标志控制对从设备接收到的字节进行的处理。

*   `IGNBRK` (Ignore BREAK): 如果设置，忽略输入中的 BREAK 条件。如果未设置，BREAK 条件会被处理为输入流中的 `\0` 字符或导致一个 `SIGINT` 信号（见 `BRKINT`）。
*   `BRKINT` (BREAK Interrupt): 如果 `IGNBRK` 未设置，当检测到 BREAK 条件时，会清空输入输出队列并向前台进程组发送一个 `SIGINT` 信号。
*   `IGNPAR` (Ignore Parity Errors): 如果设置，忽略带有奇偶校验错误的字节。
*   `PARMRK` (Mark Parity Errors): 如果 `IGNPAR` 未设置，当接收到带有奇偶校验或帧错误的字节时，会在输入流中插入一个三字节序列 `\377 \0 X`（其中 `X` 是错误的字节）。
*   `INPCK` (Input Parity Check): 启用输入奇偶校验。
*   `ISTRIP` (Input Strip): 如果设置，将每个输入字节的第8位（最高位）剥离，使其变为7位字符。
*   `INLCR` (Input Newline to Carriage Return): 将接收到的换行符 `\n` (NL) 转换成回车符 `\r` (CR)。
*   `IGNCR` (Ignore Carriage Return): 忽略接收到的回车符 `\r`。
*   `ICRNL` (Input Carriage Return to Newline): 将接收到的回车符 `\r` 转换成换行符 `\n`。这是终端输入的常见设置。
*   `IUCLC` (Input Uppercase to Lowercase): (非 POSIX 标准) 将接收到的大写字母转换为小写字母。
*   `IXON` (Input XON/XOFF Flow Control): 启用输出的软件流控。当终端驱动接收到 STOP 字符（通常是 `Ctrl+S`）时，会暂停向设备发送数据；接收到 START 字符（`Ctrl+Q`）时恢复。
*   `IXANY` (Input Any to XON): 如果设置，允许任意字符（而不仅仅是 START 字符）来重新启动被暂停的输出。
*   `IXOFF` (Input XOFF/XON Flow Control): 启用输入的软件流控。当输入缓冲区接近满时，终端驱动会自动发送一个 STOP 字符来请求对端停止发送数据。
*   `IMAXBEL` (Input Max Bell): 当输入队列已满，再次接收到字符时，响铃（BEL, `\a`）。如果未设置，多余的字符会被丢弃。
*   `IUTF8` (Input UTF-8): (Linux 特有) 告知 TTY 驱动输入是 UTF-8 编码，这允许在规范模式下正确处理多字节字符的退格。

---

## `c_oflag` - 输出模式标志 (Output Flags)

这些标志控制对发送到设备的字节进行的后处理。

*   `OPOST` (Output Post-processing): 启用实现定义的输出后处理。这是所有其他 `c_oflag` 标志的总开关。如果关闭，所有其他标志都无效，字符会被“按原样”发送。
*   `OLCUC` (Output Lowercase to Uppercase): (非 POSIX 标准) 将输出的小写字母转换为大写字母。
*   `ONLCR` (Output Newline to CR-NL): 将输出的换行符 `\n` 转换成回车-换行 `\r\n` 序列。这是终端输出的常见设置。
*   `OCRNL` (Output CR to NL): 将输出的回车符 `\r` 转换成换行符 `\n`。
*   `ONOCR` (Output No CR at Column 0): 在第0列（行首）不输出回车符。
*   `ONLRET` (Output NL Performs CR): 设置后，换行符本身会执行回车的功能，因此驱动不应再发送回车符。
*   `OFILL` (Output Fill): 使用填充字符（见 `OFDEL`）来实现延迟，而不是使用定时延迟。
*   `OFDEL` (Output Fill is DEL): 如果设置，填充字符是 DEL (`\x7F`)。否则是 NUL (`\x00`)。
*   **延迟掩码 (Delay Masks)**: `NLDLY`, `CRDLY`, `TABDLY`, `BSDLY`, `VTDLY`, `FFDLY`。这些掩码及其对应的值（如 `NL0`, `NL1`）用于在非常古老的、缓慢的物理打印机终端上设置特定控制字符后的延迟时间，以等待机械部件完成动作。在现代终端模拟器中，这些已无用处。

---

## `c_cflag` - 控制模式标志 (Control Flags)

这些标志控制与硬件相关的终端设置，如串口线路参数。

*   `CSIZE`: 字符大小掩码。其值应为 `CS5`, `CS6`, `CS7`, `CS8` 之一。
*   `CS5`, `CS6`, `CS7`, `CS8`: 分别将字符大小设置为 5、6、7、8 比特。
*   `CSTOPB` (Control Stop Bits): 如果设置，使用两个停止位。否则，使用一个停止位。
*   `CREAD` (Control Read): 启用接收器，允许从线路读取数据。
*   `PARENB` (Parity Enable): 启用奇偶校验。在输出时会生成校验位，在输入时会进行校验检查。
*   `PARODD` (Parity Odd): 如果 `PARENB` 已设置，此标志选择奇校验。否则，使用偶校验。
*   `HUPCL` (Hang Up on Close): 当最后一个持有该终端文件描述符的进程关闭它时，降低调制解调器控制线（如 DTR），从而挂断电话线。
*   `CLOCAL` (Control Local): 忽略调制解调器的状态线（如载波检测 CD）。这表示这是一个“本地”连接，不依赖调制解调器状态。连接串口时通常需要设置此标志。

---

## `c_lflag` - 本地模式标志 (Local Flags)

这些标志控制终端的高级功能，如行编辑、信号生成和回显。

*   `ISIG` (Input Signal): 当接收到 `INTR` (`Ctrl+C`), `QUIT` (`Ctrl+\`), `SUSP` (`Ctrl+Z`) 或 `DSUSP` (`Ctrl+Y`) 特殊字符时，为前台进程组生成相应的信号 (`SIGINT`, `SIGQUIT`, `SIGTSTP`)。
*   `ICANON` (Canonical Mode): 启用规范模式。这是最重要的标志之一。
    *   **开启时 (规范模式)**: 输入是按行处理的。用户可以进行行内编辑（使用 `ERASE`, `KILL` 等特殊字符），直到按下回车 (`\n`) 或 `EOF`，整行数据才对 `read()` 可用。
    *   **关闭时 (非规范模式)**: 输入数据不按行缓冲。`read()` 调用会根据 `VMIN` 和 `VTIME` 的设置立即返回可用的字符。`vi` 和 `emacs` 等全屏编辑器工作在非规范模式下。
*   `ECHO`: 将输入的字符回显到终端。
*   `ECHOE` (Echo Erase): 如果 `ICANON` 也被设置，当接收到 `ERASE` 字符（通常是退格键）时，向终端发送 "backspace-space-backspace" 序列，以在屏幕上擦除上一个字符。
*   `ECHOK` (Echo Kill): 如果 `ICANON` 也被设置，当接收到 `KILL` 字符（通常是 `Ctrl+U`）时，通过回显一个换行符来视觉上清除当前行。
*   `ECHONL` (Echo Newline): 如果 `ICANON` 也被设置，即使 `ECHO` 未设置，也回显换行符 `\n`。
*   `NOFLSH` (No Flush): 当生成 `SIGINT`, `SIGQUIT`, `SIGTSTP` 信号时，禁止清空输入和输出队列。
*   `TOSTOP` (To Stop): 如果一个后台进程试图向其控制终端写入 (`SIGTTOU`) 或读取 (`SIGTTIN`)，则向该进程发送 `SIGSTOP` 信号。
*   `ECHOCTL` (Echo Control): 如果 `ECHO` 也被设置，将 ASCII 控制字符（值 0-31）回显为 `^X` 的形式（例如，`Ctrl+C` 显示为 `^C`）。
*   `ECHOPRT` (Echo Print): (已废弃) 在擦除字符时，将其“打印”出来。例如，擦除 'a' 时，显示 `\a`。
*   `ECHOKE` (Echo Kill Erase): 如果 `ICANON` 也被设置，通过更彻底的方式来回显 `KILL` 字符，即为行上的每个字符都发送擦除序列。
*   `FLUSHO`: (非标准) 切换输出刷新模式。当设置时，输出被丢弃。通常由 `Ctrl+O` 键切换。
*   `PENDIN`: (与 `FLUSHO` 配合使用) 当输出刷新被重新启用时（即 `FLUSHO` 被清除），重新显示所有待处理的、尚未被读取的输入。
*   `IEXTEN` (Input Extension): 启用实现定义的输入处理。这是一个总开关，用于开启一些非 POSIX 标准但很常见的特殊字符处理，如 `LNEXT` (`Ctrl+V`) 和 `WERASE` (`Ctrl+W`)。
*   `EXTPROC` (External Process): (Linux 特有，已废弃) 使用外部进程进行终端处理。
