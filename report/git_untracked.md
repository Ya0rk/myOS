# `git status` 中未跟踪文件的发现机制：完整控制流解析

## 摘要

本文以一个刚刚 `git init` 并添加了新文件的仓库为例，从用户执行 `git status` 命令开始，深入 Git 源代码，详细追踪了其发现、识别和处理未跟踪 (untracked) 文件的完整控制流。内容涵盖了从命令分发、状态收集器的初始化，到核心的目录扫描与分类，再到最终结果的打印，旨在揭示 `git status` 背后复杂而高效的文件系统交互逻辑。

---

## 场景设定

1.  `mkdir my_repo && cd my_repo`
2.  `git init`
3.  `echo "hello" > file.txt`
4.  `mkdir dir && echo "world" > dir/another.txt`
5.  `echo "*.log" > .gitignore`
6.  `echo "temp" > test.log`
7.  用户在 `my_repo` 目录下执行 `git status`。

此时，仓库中没有提交，暂存区 (`.git/index`) 为空。工作区有四个条目：`file.txt`, `dir/`, `.gitignore`, `test.log`。

---

## 完整控制流详解

### 阶段一：命令分发与 `wt_status` 初始化

1.  **`main()` (位于 `git.c`)**:
    *   **动作**: `git` 的主入口函数接收到命令行参数 `["status"]`。
    *   **目的**: 它解析参数，识别出要执行的命令是 `status`，然后查找并调用与该命令关联的处理函数。

2.  **`cmd_status()` (位于 `builtin/status.c`)**:
    *   **动作**: 这是 `git status` 命令的 C 函数入口。它开始为状态报告做准备。
    *   **目的**:
        a.  **分配 `wt_status` 结构体**: 在栈上或堆上分配一个 `struct wt_status` 变量 `s`。
        b.  **初始化**: 调用 `wt_status_init(&s)`。这个函数会将 `s` 的内存清零，初始化 `string_list` 成员，并根据 `.git/config` 文件中的配置（如 `status.showUntrackedFiles`）设置 `s.show_untracked_files` 的默认值（通常是 `SHOW_NORMAL_UNTRACKED_FILES`）。
        c.  **解析命令行选项**: 解析 `git status` 后面的选项（本例中没有），并相应地修改 `s` 中的标志位。

### 阶段二：状态收集的核心 `wt_status_collect`

`cmd_status()` 接着会调用 `wt_status_collect(&s)`。这个函数是状态收集的总指挥。

3.  **`wt_status_collect()` (位于 `wt-status.c`)**:
    *   **动作**: 此函数按顺序协调对仓库不同状态的检查。
    *   **目的**:
        a.  **检查 `HEAD`**: 它会检查 `HEAD` 是否存在。在本例中，由于是初始仓库，`HEAD` 指向一个不存在的分支，`s.is_initial` 标志会被设为 `true`。
        b.  **收集已跟踪文件的变更**: 它会调用 `wt_status_collect_changes_worktree(&s)` 和 `wt_status_collect_changes_index(&s)`。但在我们的场景中，暂存区是空的，`HEAD` 也不存在，所以这些函数几乎什么都不做，不会发现任何“已修改”或“已暂存”的文件。
        c.  **关键调用**: 最后，它调用 `wt_status_collect_untracked(&s)`，将控制权交给专门处理未跟踪文件的函数。

### 阶段三：发现未跟踪文件的核心 `wt_status_collect_untracked`

这是我们追踪的重点。

4.  **`wt_status_collect_untracked()` (位于 `wt-status.c`)**:
    *   **动作**: 准备并启动目录扫描。
    *   **目的**:
        a.  **检查开关**: 确认 `s.show_untracked_files` 不是 `false`。
        b.  **初始化 `dir_struct`**: 创建并初始化一个 `struct dir_struct dir` 变量。这个变量是目录扫描的“上下文”和“结果容器”。
        c.  **加载忽略规则**: 调用 `setup_standard_excludes(&dir)`。
            *   **控制流深入**: `setup_standard_excludes()` (位于 `dir.c`) 会：
                i.  读取核心忽略文件 (`.git/info/exclude`)。
                ii. **从当前目录 (`.`) 开始向上层递归查找 `.gitignore` 文件**。在本例中，它会找到并读取我们创建的 `.gitignore` 文件。
                iii. 将解析出的规则（`"*.log"`）加载到 `dir` 结构体的一个内部列表中。
        d.  **启动扫描**: 调用 `fill_directory(&dir, istate, &s->pathspec)`。

### 阶段四：递归扫描与分类的核心 `fill_directory`

5.  **`fill_directory()` (位于 `dir.c`)**:
    *   **动作**: 这是 Git 中一个非常核心的函数，负责递归地读取目录内容，并根据暂存区和忽略规则进行分类。
    *   **目的与流程**:
        a.  **打开目录**: 使用 `opendir(".")` 系统调用打开当前目录。
        b.  **读取条目**: 在一个循环中，使用 `readdir()` 系统调用读取目录中的每一个条目。
        c.  **处理 `.gitignore`**: 当它读到 `.gitignore` 这个条目时，它会识别出这是一个已跟踪的文件（因为它通常被建议提交），或者是一个它需要处理的特殊文件，通常不会将其本身列为“未跟踪”。
        d.  **处理 `file.txt`**:
            i.  **检查暂存区**: 调用 `index_name_is_other(istate, "file.txt", ...)`。由于暂存区为空，此函数返回 `true`（表示“这是一个暂存区不知道的文件”）。
            ii. **检查忽略规则**: 调用 `is_ignored("file.txt", ...)`。`"file.txt"` 不匹配 `*.log` 规则，所以此函数返回 `false`。
            iii. **分类**: 因为该文件既未被跟踪也未被忽略，所以 `fill_directory` 会将 `file.txt` 的信息添加到一个新的 `dir_entry` 结构体中，并将其指针存入 `dir.entries` 数组。
        e.  **处理 `test.log`**:
            i.  **检查暂存区**: `index_name_is_other` 返回 `true`。
            ii. **检查忽略规则**: `is_ignored("test.log", ...)`。`"test.log"` 匹配 `*.log` 规则，此函数返回 `true`。
            iii. **分类**: 因为文件被忽略了，所以它**不会**被添加到 `dir.entries` 数组中。如果 `git status --ignored` 被调用，它会被添加到 `dir.ignored` 数组。
        f.  **处理 `dir/` (目录)**:
            i.  **检查暂存区**: `index_name_is_other` 返回 `true`。
            ii. **检查忽略规则**: 目录本身通常不会被忽略规则直接匹配（除非规则是 `dir/`）。
            iii. **递归**: 因为这是一个目录，`fill_directory` 会**递归地调用自身**，进入 `dir/` 目录，并对 `another.txt` 重复上述 c, d, e 的过程。最终，`dir/another.txt` 会被添加到 `dir.entries` 数组中。

6.  **返回到 `wt_status_collect_untracked`**:
    *   **动作**: `fill_directory` 执行完毕后返回。此时，`dir.entries` 数组中包含了 `file.txt` 和 `dir/another.txt`。`dir.ignored` 数组为空（除非使用了 `--ignored`）。
    *   **目的**:
        a.  **最终填充**: `wt_status_collect_untracked` 中的 `for` 循环开始遍历 `dir.entries` 数组。
        b.  对于 `file.txt` 和 `dir/another.txt`，它调用 `string_list_insert(&s->untracked, ent->name)`，将这两个文件名**正式添加**到 `wt_status` 结构体的 `untracked` 列表中。
        c.  调用 `dir_clear(&dir)` 释放扫描过程中分配的内存。

### 阶段五：结果打印

7.  **`wt_status_print()` (位于 `wt-status.c`)**:
    *   **动作**: `wt_status_collect` 执行完毕后，`cmd_status` 调用 `wt_status_print(&s)`。
    *   **目的**:
        a.  **打印头部**: 打印 `On branch main`, `No commits yet` 等信息。
        b.  **打印未跟踪文件**: 检查 `s->untracked` 列表是否为空。
        c.  如果不为空，它会打印出 "Untracked files:" 的标题。
        d.  然后，它**遍历 `s->untracked` 列表**，将 `file.txt` 和 `dir/` (Git 会智能地折叠目录) 打印到标准输出。

### 总结

| 阶段 | 核心函数/模块 | 主要动作 (ABI 调用) | 产出/结果 |
| :--- | :--- | :--- | :--- |
| 1. **分发** | `main`, `cmd_status` | `fork`, `execve` | 初始化 `wt_status` 结构体 |
| 2. **收集** | `wt_status_collect` | (协调) | 调用 `wt_status_collect_untracked` |
| 3. **准备扫描** | `wt_status_collect_untracked` | `open`, `read` (`.gitignore`) | 初始化 `dir_struct`，加载忽略规则 |
| 4. **核心扫描** | `fill_directory` | `opendir`, `readdir`, `lstat` | 递归遍历目录，将未跟踪文件分类到 `dir.entries` |
| 5. **填充结果** | `wt_status_collect_untracked` | (内存操作) | 将 `dir.entries` 的内容转移到 `s.untracked` |
| 6. **打印** | `wt_status_print` | `write` (标准输出) | 遍历 `s.untracked` 列表并格式化输出给用户 |

这个流程展示了 `git status` 如何通过一系列精心设计的模块化函数，将底层的系统调用 (`readdir`, `lstat`) 与高层的 Git 逻辑（暂存区状态、忽略规则）相结合，最终高效地完成了对未跟踪文件的识别。
