# Git 核心数据结构深度解析

## 摘要

本文深入 Git 的源代码，对构成其强大功能的基石——核心数据结构——进行全面而详细的解析。通过分析每个数据结构的定义、在内存和磁盘上的表示、核心作用以及相关的操作函数，本文旨在为希望深入理解 Git 内部工作原理的开发者提供一份详尽的指南。内容涵盖了从底层的对象（Blob, Tree, Commit, Tag）到上层的引用 (Ref) 和暂存区 (Index)，并解释了它们如何协同工作，共同构建起 Git 精巧的版本控制模型。

---

## 1. 基础：Git 对象模型 (The Object Model)

Git 的核心是一个简单的键值对数据库。你可以向 Git 仓库中放入任何类型的内容，Git 会为你返回一个唯一的键（一个 40 位的 SHA-1 哈希值），通过这个键可以在任何时候再次取回该内容。

所有的数据都存储为四种基本对象类型之一。这些对象都存储在 `.git/objects/` 目录下。

### 1.1. `struct object` - 内存中的通用对象表示

*   **定义位置**: `object.h`
*   **核心定义 (简化后)**:
    ```c
    struct object {
        unsigned char sha1[20]; // 20 字节的 SHA-1 哈希值
        enum object_type type;  // 对象类型 (OBJ_BLOB, OBJ_TREE, etc.)
        unsigned int flags;     // 标志位 (如 PARSED, REACHABLE)
        // ... 其他用于缓存和遍历的成员 ...
    };
    ```
*   **含义与作用**:
    这是 Git 在**内存中**表示所有四种基本对象的通用结构体。它像一个“基类”，包含了所有对象的共同特征：一个唯一的 SHA-1 哈希值和一个类型标识。当 Git 从磁盘加载一个对象时，它会先创建一个 `struct object`，然后根据其 `type` 字段，将其转换为更具体的对象结构体（如 `struct blob`）。

*   **相关操作**:
    *   **读 (Parse)**: `parse_object(const unsigned char *sha1)` (位于 `object.c`) 是核心的读取函数。它根据 SHA-1 哈希从对象数据库中找到数据，解压，解析其头部（"type size\0"），然后创建一个 `struct object` 并返回。
    *   **写**: 写入操作通常是类型特定的，例如 `write_blob_from_file`。

---

### 1.2. Blob 对象 - 文件内容的快照

*   **磁盘格式**: `zlib` 压缩后的 `blob <size>\0<file_content>`
*   **内存结构**: `struct blob` (定义于 `blob.h`)
    ```c
    struct blob {
        struct object object;
    };
    ```
    (注意：`struct blob` 只是 `struct object` 的一个别名，因为 Blob 对象除了通用属性外没有其他元数据。)
*   **含义与作用**:
    Blob (Binary Large Object) 对象是 Git 用来存储**文件内容**的。它只存储文件的原始数据，不包含文件名、路径、权限等任何元数据。Git 本质上是一个**内容寻址**的系统，文件名和文件内容的关联是在 Tree 对象中实现的。

*   **相关操作**:
    *   **写 (增)**: `write_object_file(const void *buf, unsigned long len, const char *type, unsigned char *sha1)` (位于 `object.c`) 是底层的写入函数。更上层的 `git hash-object` 命令或 `git add` 过程会调用它，将文件内容写入对象数据库，并返回一个 SHA-1 哈希。
    *   **读**: `read_object_file(const unsigned char *sha1, enum object_type *type, unsigned long *size)` (位于 `object.c`) 用于读取原始的、解压后的对象内容。`git cat-file -p <blob_sha1>` 命令最终会调用它。

---

### 1.3. Tree 对象 - 目录结构的快照

*   **磁盘格式**: `zlib` 压缩后的 `tree <size>\0<entry1><entry2>...`
    *   每个 `entry` 的格式是：`<mode> <filename>\0<20_byte_sha1>`
*   **内存结构**: `struct tree` (定义于 `tree.h`)
    ```c
    struct tree {
        struct object object;
        void *buffer;         // 指向原始的、解压后的 tree 对象数据
        unsigned long size;
        // ... 用于缓存解析结果的成员 ...
    };
    ```
*   **含义与作用**:
    Tree 对象代表了一个**目录的快照**。它像一个清单，列出了该目录下包含的文件和子目录。
    *   对于文件，它记录了文件的**模式 (权限)、文件名**和指向该文件内容的 **Blob 对象的 SHA-1**。
    *   对于子目录，它记录了**目录名**和指向该子目录的 **Tree 对象的 SHA-1**。
    通过 Tree 对象的嵌套引用，Git 能够精确地表示出整个项目在某个时间点的完整目录结构。

*   **相关操作**:
    *   **读 (Parse)**: `parse_tree(struct tree *item)` (位于 `tree.c`) 会解析 `tree->buffer` 中的原始数据，并填充树的内部缓存，方便快速查找。`git ls-tree <tree_sha1>` 命令会使用这个功能。
    *   **写 (增)**: `write_tree(const unsigned char *sha1)` (位于 `write-cache.c`) 是一个核心函数，它会根据**当前的暂存区 (index)** 内容，创建一个新的 Tree 对象（以及必要的子 Tree 对象），并将其写入对象数据库。这是 `git commit` 的关键步骤之一。

---

### 1.4. Commit 对象 - 历史记录的节点

*   **磁盘格式**: `zlib` 压缩后的 `commit <size>\0<commit_metadata>`
    *   `commit_metadata` 是纯文本，格式如下：
        ```
        tree <tree_sha1>
        parent <parent_commit_sha1>
        author <name> <email> <timestamp> <timezone>
        committer <name> <email> <timestamp> <timezone>
        
        <commit message>
        ```
*   **内存结构**: `struct commit` (定义于 `commit.h`)
    ```c
    struct commit {
        struct object object;
        struct commit_list *parents; // 指向父提交的链表
        struct tree *tree;           // 指向根 Tree 对象
        char *buffer;                // 指向原始的、解压后的 commit 对象数据
        // ... 其他用于缓存和遍历的成员 ...
    };
    ```
*   **含义与作用**:
    Commit 对象是 Git 历史记录中的一个**节点**。它将一个时间点的项目状态（通过 Tree 对象）与一系列元数据关联起来。
    *   `tree`: 指向了该次提交时项目**根目录的 Tree 对象**，代表了项目在该时间点的完整快照。
    *   `parent`: 指向一个或多个**父提交**。这构成了 Git 的有向无环图 (DAG) 历史链。一个普通的提交有一个父提交，一个合并提交 (merge commit) 有两个或多个父提交。
    *   `author` / `committer`: 记录了作者和提交者的信息。
    *   `commit message`: 提交信息。

*   **相关操作**:
    *   **读 (Parse)**: `parse_commit(struct commit *item)` (位于 `commit.c`) 会解析 `commit->buffer` 中的元数据，并填充 `parents`, `tree` 等字段。`git log` 命令的核心就是解析 Commit 对象并沿着 `parent` 链回溯。
    *   **写 (增)**: `git commit` 命令最终会调用 `commit_tree` (位于 `commit.c`) 或类似函数，它会收集 `tree`, `parent`, `author` 等信息，格式化成 Commit 对象的文本格式，然后调用 `write_object_file` 将其写入对象数据库。

---

### 1.5. Tag 对象 - 特定的命名引用

*   **磁盘格式**: `zlib` 压缩后的 `tag <size>\0<tag_metadata>`
    *   `tag_metadata` 格式：
        ```
        object <sha1_of_tagged_object>
        type <type_of_tagged_object>
        tag <tag_name>
        tagger <name> <email> <timestamp> <timezone>
        
        <tag message>
        ```
*   **内存结构**: `struct tag` (定义于 `tag.h`)
    ```c
    struct tag {
        struct object object;
        struct object *tagged; // 指向被标记的对象
        char *tag;             // 标签名
        // ...
    };
    ```
*   **含义与作用**:
    Tag 对象用于为一个特定的 Commit (或其他对象) 创建一个**永久的、不可移动的命名引用**。与分支不同，标签创建后通常不会再改变。附注标签 (`git tag -a`) 会创建 Tag 对象，它包含了标签创建者、日期、消息，并且可以被 GPG 签名，而轻量标签 (`git tag`) 只是一个直接指向 Commit 的引用，不创建 Tag 对象。

*   **相关操作**:
    *   `git tag -a` 命令会创建一个 Tag 对象。
    *   `git show <tag_name>` 会先解析 Tag 对象，再解析它指向的 Commit 对象。

---

## 2. 上层建筑：引用与暂存区

如果说对象是 Git 的“数据”，那么引用和暂存区就是 Git 的“指针”和“草稿纸”。

### 2.1. 引用 (Refs) - 人类可读的指针

*   **磁盘格式**:
    *   **松散引用 (Loose Refs)**: 位于 `.git/refs/` 目录下的普通文本文件。例如，`.git/refs/heads/main` 文件中只包含一行 40 位的 SHA-1 哈希值。
    *   **打包引用 (Packed Refs)**: 为了效率，Git 会定期将松散引用打包到一个名为 `.git/packed-refs` 的单一文本文件中。
*   **内存结构**: `struct ref` (定义于 `refs.h`)
    ```c
    struct ref {
        struct ref *next;
        unsigned char old_sha1[20];
        unsigned char new_sha1[20];
        char *name;
        // ...
    };
    ```
*   **含义与作用**:
    引用 (Reference, ref) 是对 Commit 对象（或其他对象）的一个**人类可读的、可移动的指针**。分支 (branch)、标签 (tag)、远程跟踪分支 (remote-tracking branch) 等都是不同类型的引用。它们使得我们可以用 `main` 这样的名字来代替一长串无意义的 SHA-1 哈希。`HEAD` 是一个特殊的引用，它指向当前所在的分支。

*   **相关操作**:
    *   **读**: `refs_resolve_ref_unsafe` (位于 `refs.c`) 是核心的引用解析函数。它会先在 `.git/refs/` 目录下查找松散引用，如果找不到，再去 `.git/packed-refs` 文件中查找。
    *   **写 (增/删/改)**: `refs_update_ref` (位于 `refs.c`) 用于安全地更新一个引用。`git branch`, `git tag`, `git checkout`, `git commit` 等大量命令都会导致引用的更新。

---

### 2.2. 暂存区 (Index / Staging Area)

*   **磁盘格式**: 一个名为 `.git/index` 的**二进制文件**。它包含了一个文件头和一系列排序好的 `cache_entry` 结构。
*   **内存结构**: `struct index_state` (定义于 `read-cache.h`)
    ```c
    struct index_state {
        struct cache_entry **cache; // 一个指向 cache_entry 指针的数组
        unsigned int cache_nr;      // 数组中的条目数
        // ...
    };

    // 暂存区中的每个条目
    struct cache_entry {
        struct stat_data ce_stat_data; // 文件的元数据 (ctime, mtime, size, etc.)
        unsigned char sha1[20];        // 文件的 Blob 对象的 SHA-1
        char name[FLEX_ARRAY];         // 文件名
    };
    ```
*   **含义与作用**:
    暂存区是 Git 最核心也最常被误解的概念之一。它是一个**二进制文件**，扮演着**下一次提交的“预备清单”**或“草稿”的角色。它不是一个目录，而是一个扁平化的文件列表。
    *   当运行 `git add <file>` 时，Git 会：
        1.  根据文件内容创建一个 Blob 对象。
        2.  用该文件的路径、元数据和新 Blob 的 SHA-1 **更新或创建**暂存区中的对应条目。
    *   当运行 `git commit` 时，Git 会：
        1.  使用**暂存区的内容**（而不是工作区的内容）来构建新的 Tree 对象。
        2.  创建一个指向该 Tree 对象的新 Commit 对象。

*   **相关操作**:
    *   **读**: `read_index_from` (位于 `read-cache.c`) 用于从 `.git/index` 文件加载内容到 `struct index_state` 内存结构中。
    *   **写**: `write_index` (位于 `write-cache.c`) 用于将内存中的 `struct index_state` 写回到 `.git/index` 文件。
    *   **增/删/改**: `add_to_index`, `remove_from_index` 等函数（位于 `read-cache.c`）用于在内存中操作暂存区条目。`git add`, `git rm`, `git mv` 等命令都会调用这些函数。

### 总结

| 数据结构 | 角色 | 存储位置 | 核心作用 |
| :--- | :--- | :--- | :--- |
| **Blob** | 数据 | `.git/objects/` | 存储文件内容 |
| **Tree** | 目录 | `.git/objects/` | 存储目录结构和文件名 |
| **Commit** | 历史 | `.git/objects/` | 连接历史快照，形成版本链 |
| **Tag** | 标签 | `.git/objects/` | 为特定提交提供永久命名 |
| **Ref** | 指针 | `.git/refs/`, `.git/packed-refs` | 为提交提供可移动的人类可读名称（如分支） |
| **Index** | 草稿 | `.git/index` | 准备下一次提交的内容清单 |

这六个数据结构共同构成了 Git 的核心。理解它们之间的关系——**Ref 指向 Commit，Commit 指向 Tree，Tree 指向 Blob 和其他 Tree，而 Index 则是构建新 Tree 的蓝图**——是理解所有 Git 命令背后原理的关键。
