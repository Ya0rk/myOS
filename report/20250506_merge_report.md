# 20250506 merge report

主要修改来自fs::mod.rs中打开文件的方法

引入了dentry机制,从而删除了inode cache

dentry机制增加获得inode的健壮性检查,并可以为后续功能完善,增加文件系统一致性提供方便,后续应该会参考Linux的设计

修复了inode和file的getdents方法,与之相关的系统调用是sys_getdents64,修改了lwext4中设置的偏移量,引入了File中的偏移量检查

linux设计得比较美观

增加76号系统调用(目前没有真正地实现todo)

应当减少inode的功能,将相关功能移到super block上(todo)


