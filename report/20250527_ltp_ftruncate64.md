
`sys_ftruncate64系统调用有点异常，没有通过assert，好像是f->mp == 0, 挂载点为空`

这个影响到很多ltp测试用例。所以我暂时在inode的truncate中注释了file.file_truncate

```
[INFO][HARTID0][TASK4][create_open_file] flags=OpenFlags(O_RDWR | O_CREAT | O_EXCL | O_LARGEFILE), abs_path=/dev/shm/ltp_mount01_4, parent_path=/dev/shm
[INFO][HARTID0][TASK4][get_inode_from_path] /dev/shm
[INFO][HARTID0][TASK4][get_inode_from_path] /dev/shm/ltp_mount01_4
[INFO][HARTID0][TASK4][get_inode_from_path] no such file or directory: /dev/shm/ltp_mount01_4
[INFO][HARTID0][TASK4][do_create] start /dev/shm/ltp_mount01_4
[INFO][HARTID0][TASK4][do_create] succe /dev/shm/ltp_mount01_4
[INFO][HARTID0][TASK4][sys_openat] taskid = 4, alloc fd finished, new fd = 3
[INFO][HARTID0][TASK4][sys_fchmodat] start
[INFO][HARTID0][TASK4][sys_ftruncate64] start
[INFO][HARTID0][TASK4]assertion failed:
file: /home/sean/myOS/lwext4_rust/c/lwext4/src/ext4.c
line: 1632
```