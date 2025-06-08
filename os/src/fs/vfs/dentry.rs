use alloc::{
    format,
    string::{String, ToString},
    vec::{self, Vec},
};
use bitflags::parser::ParseError;
use hashbrown::{HashMap, HashSet};
use log::{error, info, warn};
use lwext4_rust::{
    bindings::{false_, printf, true_},
    InodeTypes,
};
// use riscv::{interrupt::Mutex, register::fcsr::read};
// use sbi_rt::{NonRetentive, SharedPtr};
use super::{inode, InodeTrait, SuperBlockTrait};
use crate::{
    fs::{
        ffi::InodeType, mkdir, open, path, procfs::PROCFS_SUPER_BLOCK, root_inode, AbsPath,
        FileClass, FileTrait, OpenFlags,
    },
    utils::{Errno, SysResult},
};
use alloc::sync::{Arc, Weak};
use core::hash::{Hash, Hasher};
use spin::rwlock::RwLock;

#[derive(Copy, Clone, PartialEq)]
pub enum DentryStatus {
    /// 这个 dentry 的 inode 是有效的，并且已经初始化（只有对于文件夹才有意义）
    Valid,
    /// 这个 dentry 的 inode 是有效的，但是没有初始化（只有对于文件夹才有意义）
    Unint,
    /// 这个 dentry 是无效的
    Negtive,
}

impl DentryStatus {
    fn new() -> RwLock<Self> {
        RwLock::new(Self::Unint)
    }
}

/// 一个目录项,文件树在内存当中的映射
pub struct Dentry {
    /// file name should be
    name: RwLock<String>,
    /// file path
    path: RwLock<Option<String>>,
    /// 父dentry的弱引用
    parent: Weak<Dentry>,
    /// 孩子dentry的强引用
    children: RwLock<HashMap<String, Arc<Dentry>>>,
    /// 用栈去存储当前的挂载的inode对象
    inode: RwLock<Vec<Arc<dyn InodeTrait>>>,
    /// dentry的状态
    status: RwLock<DentryStatus>,
}

impl PartialEq for Dentry {
    fn eq(&self, other: &Self) -> bool {
        self.name.read().clone() == other.name.read().clone()
    }
}

impl Eq for Dentry {}

impl Hash for Dentry {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.read().hash(state);
    }
}

/// 根节点dentry
lazy_static! {
    static ref DENTRY_ROOT: Arc<Dentry> = Dentry::new_root();
}

impl Dentry {
    /// 初始化dentry系统,将根节点和ext4文件系统绑定
    pub fn init_dentry_sys() {
        info!("[dentry init]");
        Self::bind(&DENTRY_ROOT, crate::fs::ext4::SUPER_BLOCK.root_inode());
        info!("list root dir");
        {
            DENTRY_ROOT
                .children
                .read()
                .iter()
                .for_each(|x| println!("{}", x.0));
        };
        // info!("mount ProcFs");
        // mkdir("/proc".into(), 0);
        // let procFs = Self::get_dentry_from_path("/proc").unwrap();
        // procFs.mount(PROCFS_SUPER_BLOCK.clone());
    }
    /// 创建一个根节点dentry
    fn new_root() -> Arc<Self> {
        // rust语法中解决环形引用的问题
        Arc::new_cyclic(|weak_self| Dentry {
            name: RwLock::new(String::from("/")),
            path: RwLock::new(Some(String::from("/"))),
            parent: weak_self.clone(),
            children: RwLock::new(HashMap::new()),
            inode: RwLock::new(Vec::new()),
            status: DentryStatus::new(),
        })
    }
    /// 创建一个没有绑定Inode的dentry,爹指向self,名字为name
    /// 注意到这个应该为单纯的名字而不是绝对路径
    fn new_bare(self: &Arc<Self>, name: &str) -> Arc<Self> {
        // info!("create bare {}", name);
        let mut inode = Vec::new();
        let res = Self {
            name: RwLock::new(String::from(name)),
            path: RwLock::new(None),
            parent: Arc::downgrade(self),
            children: RwLock::new(HashMap::new()),
            inode: RwLock::new(inode),
            status: DentryStatus::new(),
        };
        let result = Arc::new(res);
        result
    }
    /// 创建一个儿子节点
    fn new(self: &Arc<Self>, name: &str, inode: Arc<dyn InodeTrait>) -> Arc<Self> {
        info!("[dentry::new] {}: {}", self.get_abs_path(), name);
        let mut inodes = Vec::new();
        let res = Self {
            name: RwLock::new(String::from(name)),
            path: RwLock::new(None),
            parent: Arc::downgrade(self),
            children: RwLock::new(HashMap::new()),
            inode: RwLock::new(inodes),
            status: DentryStatus::new(),
        };
        let res = Arc::new(res);
        self.children
            .write()
            .insert(String::from(name), res.clone());
        // info!("[Dentry::new] {} insert child {} ", self.name.read(), name);
        res.inode.write().push(inode);
        res
    }
    /// 查看是否是有效的 dentry
    fn is_valid(&self) -> bool {
        let status = *self.status.read();
        status == DentryStatus::Valid || status == DentryStatus::Unint
    }

    /// 查看是否是无效的 dentry
    fn is_negtive(&self) -> bool {
        *self.status.read() == DentryStatus::Negtive
    }

    fn clear(&self) {
        self.inode.write().clear();
        *self.status.write() = DentryStatus::Negtive;
    }
    fn get_status(&self) -> DentryStatus {
        *self.status.read()
    }
    fn set_status(&self, status: DentryStatus) {
        *self.status.write() = status
    }

    /// 安全的获得 parent 方法， 当不存在上级文件夹的时候会返回 None
    fn parent(&self) -> Option<Arc<Self>> {
        let parent = self.parent.upgrade()?;
        Some(parent)
    }

    fn get_abs_path(&self) -> String {
        let name = self.name.read();
        {
            let read = self.path.read();
            if let Some(path) = read.as_ref() {
                return String::from(path);
            };
        }
        {
            let write = self.path.write();
            let parent_path = if let Some(parent) = self.parent() {
                parent.get_abs_path()
            } else {
                String::new()
            };
            if parent_path.ends_with("/") {
                format!("{}{}", parent_path, name)
            } else {
                format!("{}/{}", parent_path, name)
            }
        }
    }

    /// pattern为文件名字
    fn get_child(self: &Arc<Self>, pattern: &str) -> Option<Arc<Self>> {
        info!("{} visit {}", self.get_abs_path(), pattern);
        let status = { self.get_status() };
        match status {
            DentryStatus::Valid => {}
            DentryStatus::Unint => {
                self.init();
            }
            DentryStatus::Negtive => return None,
        };
        if pattern.ends_with("..") {
            info!("return parent");
            return self.parent();
        } else if pattern.ends_with("/") || pattern.ends_with(".") || pattern == "" {
            info!("return name is {}", self.name.read());
            return Some(self.clone());
        }
        // 直接检索当前的文件夹
        {
            let children = self.children.read();
            if let Some(dentry) = children.get(pattern) {
                return Some(dentry.clone());
            }
        }
        None
    }

    /// name为相对路径
    ///
    /// 增加孩子, 只要父母是合法的, 那就一定会返回Dentry
    /// 即当Inode不存在的时候就会创建Inode
    pub fn add_child(self: Arc<Self>, name: &str, flag: OpenFlags) -> Option<Arc<dyn InodeTrait>> {
        // 如果是无效的 dentry 就直接返回
        if self.is_negtive() {
            return None;
        }
        // concat path for creating
        let self_name = self.get_abs_path();
        let target_name = if self_name.ends_with("/") {
            format!("{}{}", self_name, name)
        } else {
            format!("{}/{}", self_name, name)
        };
        info!("[Dentry::add_child] add {} in {}", name, self_name);
        // get inode
        if let Some(parent_inode) = self.get_inode() {
            if parent_inode.node_type() != InodeType::Dir {
                warn!(
                    "[Dentry::add_child] {}: should add child in a dir",
                    self.name.read()
                );
                return None;
            }

            if let Some(child_dentry) = self.clone().get_child(&target_name) {
                // 如果存在这个节点就直接返回获得的Inode
                return child_dentry.get_inode();
            } else {
                //如果不存在就创建,使用Inode的do_create方法
                if let Some(inode) = parent_inode.do_create(&target_name, flag.node_type().into()) {
                    self.new(&target_name, inode.clone());
                    return Some(inode.clone());
                } else {
                    // 创建不成功
                    return None;
                }
            }
        }

        None
    }

    //
    pub fn release_self(self: Arc<Self>) -> Option<()> {
        let child_name = self.name.read().clone();
        let parent = self.parent()?;
        let mut parent_children = parent.children.write();
        parent_children.remove(&child_name);
        Some(())
    }

    /// 将一个dentry和inode绑定,如果inode是一个文件夹,就把为他的儿子创建一个新的dentry
    fn bind(self: &Arc<Self>, inode: Arc<dyn InodeTrait>) {
        //将inodepush进inode栈,然后flush,注意到这里需要用大括号包裹,不然会死锁
        info!("dentry bind {}", self.get_abs_path());
        {
            self.inode.write().push(inode);
        }
        self.set_status(DentryStatus::Unint);
        self.init();
        info!("finished bind");
    }

    // FIXME: 可能存在错误， 并没有实现逻辑
    pub fn unbind(self: &Arc<Self>) {
        {
            self.inode.write().pop();
        }
        self.set_status(DentryStatus::Unint);
        self.init();
        info!("finished unbind");
    }

    fn init(self: &Arc<Self>) -> SysResult<()> {
        info!("dentry init {}", self.get_abs_path());
        {
            // 查看当前状态， 如果已经初始化成功就返回， 如果是无效的也直接返回
            match self.get_status() {
                DentryStatus::Negtive => return Err(Errno::ENOENT),
                DentryStatus::Valid => return Ok(()),
                DentryStatus::Unint => {}
            };
        }
        let inode = if let Some(inode) = self.get_inode() {
            inode
        } else {
            return Err(Errno::ENOENT);
        };
        if inode.node_type() == InodeType::Dir {
            let dents = inode.read_dents().unwrap();
            for dent in dents {
                let name_byte = dent.d_name;
                let end = name_byte
                    .iter()
                    .position(|&b| b == 0)
                    .unwrap_or(name_byte.len());
                let real_name = String::from_utf8_lossy(&name_byte[0..end]);
                let son = Self::new_bare(&self, &real_name);
                // info!("insert child dentry {}", &real_name);
                self.children.write().insert(real_name.to_string(), son);
            }
        } else {
            self.children.write().clear();
        }
        self.set_status(DentryStatus::Valid);
        Ok(())
    }

    /// 将一个dentry和一个superblock绑定
    fn mount(self: &Arc<Self>, sb: Arc<dyn SuperBlockTrait>) {
        if sb.root_inode().node_type() != InodeType::Dir {
            info!("you can't mount a inode which is not TYPE DIR");
            return;
        }
        Dentry::bind(self, sb.root_inode());
        self.init();
        info!("bind a superblock to dentry!");
    }
    /// 从一个dentry上获取inode
    ///
    /// 这个行为只会在 dentry 不是 negtive 的情况下有效
    ///
    /// 实际上这个函数是 get_inode 和
    ///
    /// 本应该属于 filesystem 的 alloc_inode
    ///
    /// 的功能二合一了
    fn get_inode(self: &Arc<Self>) -> Option<Arc<dyn InodeTrait>> {
        // {
        //     info!(
        //         "[get_inode] {:?}, inode vec len is {}",
        //         self.name.read(),
        //         self.inode.read().len()
        //     );
        // }
        {
            if self.is_negtive() {
                return None;
            }
        }
        // 首先检查是否已有 inode（读锁）
        {
            let inode_guard = self.inode.read();
            if let Some(inode) = inode_guard.last() {
                return Some(inode.clone());
            }
        }
        {
            // 获取父节点
            let parent_dentry = self.parent.upgrade()?;
            // 获取父节点的 inode 并执行 walk
            let parent_inode = parent_dentry.get_inode()?;
            let this_inode = parent_inode.loop_up(&self.get_abs_path())?;
            // 存储 inode 到栈
            self.inode.write().push(this_inode.clone());
            Some(this_inode)
        }
    }

    /// 根据绝对路径获取对应的inode
    pub fn get_inode_from_path(path: &str) -> SysResult<Arc<dyn InodeTrait>> {
        info!("get inode from path {}", path);
        info!("    [get_inode_from_path] {}", path);
        if !path.starts_with('/') {
            panic!("path should start with /");
        }
        let mut dentry_now = DENTRY_ROOT.clone();
        if path == "/" {
            return Ok(dentry_now.get_inode().unwrap());
        }
        let mut path_now = String::from("/");

        let path_split = path.split('/').enumerate();
        let size_of_path = path_split.clone().count();
        for (i, name) in path_split {
            match dentry_now.get_child(name) {
                Some(child) => {
                    dentry_now = child;
                    match dentry_now.get_inode() {
                        Some(mid_inode) => {
                            if !mid_inode.is_dir() && i < size_of_path - 1 {
                                return Err(Errno::ENOTDIR);
                            }
                        }
                        None => {
                            return Err(Errno::ENOENT);
                        }
                    };
                }
                None => {
                    info!(
                        "[get_inode_from_path] no such file or directory: {}",
                        path_now
                    );
                    return Err(Errno::ENOENT);
                }
            }
        }
        if let Some(inode) = dentry_now.get_inode() {
            if inode.is_valid() {
                Ok(inode)
            } else {
                Err(Errno::ENOENT)
            }
        } else {
            Err(Errno::ENOENT)
        }
    }

    /// 根据绝对路径找到dentry
    /// path： 绝对路径
    pub fn get_dentry_from_path(path: &str) -> SysResult<Arc<Self>> {
        if !path.starts_with('/') {
            panic!("path {} should start with /", path);
        }
        let mut dentry_now = DENTRY_ROOT.clone();
        if path == "/" {
            return Ok(dentry_now);
        }
        let mut path_now = String::from("/");

        let path_split = path.split('/').enumerate();
        let size_of_path = path_split.clone().count();
        for (i, name) in path_split {
            match dentry_now.get_child(&path_now) {
                Some(child) => {
                    dentry_now = child;
                    match dentry_now.get_inode() {
                        Some(mid_inode) => {
                            if !mid_inode.is_dir() && i < size_of_path - 1 {
                                return Err(Errno::ENOTDIR);
                            }
                        }
                        None => {
                            return Err(Errno::ENOENT);
                        }
                    };
                }
                None => {
                    info!(
                        "[get_inode_from_path] no such file or directory: {}",
                        path_now
                    );
                    return Err(Errno::ENOENT);
                }
            }
        }
        Ok(dentry_now)
    }
}

macro_rules! test_inode {
    ($path:expr) => {
        if let Ok(inode) = Dentry::get_inode_from_path(&String::from($path)) {
            error!("[test_inode] inode stat for {}: {:?}", $path, inode.fstat());
        } else {
            error!("[test_inode] no such file or directory: {}", $path);
        }
    };
}
macro_rules! banner {
    ($info:expr) => {
        error!(
            "=============================\tTEST {}\t=============================",
            $info
        );
    };
}

pub async fn dentry_test() {
    banner!("get inode from path");
    test_inode!("/musl");
    test_inode!("/musl/../musl");
    test_inode!("/musl/basic/brk");
    test_inode!("/glibc/basic/mnt/invalid");
    test_inode!("//././././..///././musl/../glibc/basic");
    panic!();
}
