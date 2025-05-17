use alloc::{format, string::String, vec::{self, Vec}};
use bitflags::parser::ParseError;
use hashbrown::{HashMap, HashSet};
use log::{debug, error, info, warn};
use lwext4_rust::{bindings::{false_, printf, true_}, InodeTypes};
// use riscv::{interrupt::Mutex, register::fcsr::read};
use sbi_rt::{NonRetentive, SharedPtr};
use spin::{rwlock::RwLock};
use core::hash::{Hash, Hasher};
use crate::{fs::{ffi::InodeType, mkdir, open_file, path, root_inode, FileClass, FileTrait, OpenFlags, Path}, utils::SysResult};
use alloc::sync::{Arc, Weak};
use super::{inode, InodeTrait, SuperBlockTrait};


/// 一个目录项,文件树在内存当中的映射
pub struct Dentry {
    ///absolute path should be
    name: RwLock<String>, 
    /// 父dentry的弱引用
    parent: Weak<Dentry>,
    /// 孩子dentry的强引用
    children: RwLock<HashMap<String, Arc<Dentry>>>,
    /// 用栈去存储当前的挂载的inode对象
    inode: RwLock<Vec<Arc<dyn InodeTrait>>>,
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


impl Dentry {
    /// 初始化dentry系统,将更节点和ext4文件系统绑定
    pub fn init() {
        info!("[dentry init]");
        Self::bind(&DENTRY_ROOT, crate::fs::ext4::SUPER_BLOCK.root_inode());
        info!("list root dir");
        {DENTRY_ROOT.children.read().iter().for_each(|x| println!("{}", x.0));};
    }
    /// 创建一个根节点dentry
    fn new_root() -> Arc<Self>{
        // rust语法中解决环形引用的问题
        Arc::new_cyclic(|weak_self| {
            Dentry {
                name: RwLock::new(String::from("/")),
                parent: weak_self.clone(),
                children: RwLock::new(HashMap::new()),
                inode: RwLock::new(Vec::new()),
            }
        })
    }
    /// 创建一个没有绑定Inode的dentry,爹指向self,名字为name
    fn new_bare(
        self: &Arc<Self>,
        name: &String,
    ) -> Arc<Self> {
        // info!("create bare {}", name);
        let mut inode = Vec::new();
        let res = Self {
            name: RwLock::new(String::from(name)),
            parent: Arc::downgrade(self),
            children: RwLock::new(HashMap::new()),
            inode: RwLock::new(inode),
        };
        let result = Arc::new(res);
        // self.children.write().insert(name.clone(), result.clone());
        result
    }
    /// 创建一个儿子节点
    fn new(
        self: &Arc<Self>,
        name: &String,
        inode: Arc<dyn InodeTrait>,    
    ) -> Arc<Self> {
        let mut inodes = Vec::new();
        let res = Self {
            name: RwLock::new(String::from(name)),
            parent: Arc::downgrade(self),
            children: RwLock::new(HashMap::new()),
            inode: RwLock::new(inodes),
        };
        let res = Arc::new(res);
        self.children.write().insert(name.clone(), res.clone());
        debug!("[Dentry::new] {} insert child {} ", self.name.read(), name);
        res.inode.write().push(inode);
        res
    }

    fn get_parent(self: Arc<Self>) -> Arc<Self> {
        let mut dentry = self.clone();
        if let Some(parent) = dentry.parent.upgrade() {
            dentry = parent;
        }
        dentry
    }

    /// pattern为绝对路径
    fn get_child(self: Arc<Self>, pattern: &String) -> Option<Arc<Self>> {
        // info!("visit {}", pattern);
        if pattern.ends_with("/") || pattern.ends_with(".") || pattern.as_str() == "" {
            // info!("return name is {}", self.name.read());
            return Some(self.clone());
        } else if pattern.ends_with("..") {
            // info!("return name is {}", self.name.read());
            return Some(self.clone().get_parent());
        }
        let pattern = Path::string2path(pattern.clone());
        let pattern_abs = pattern.get();
        let pattern_filename = pattern.get_filename();
        {    
            let children = self.children.read();
            if let Some(dentry) = children.get(&pattern_abs) {
                return Some(dentry.clone());
            }
        }
        // 注意到这里其实是一个临时的机制,因为一个子文件可能并不属于这个文件系统,而是外部挂载而来
        // 应当改进flush_binding、mount、unmount的逻辑,实现粒度更小的操作
        let dents = self.get_inode().unwrap().read_dents().unwrap();
        let mut children_vec = self.children.write();
        for dent in dents {
            let name = Vec::from(dent.d_name);
            match String::from_utf8(name) {
                Ok(name) => {
                    let real_name = name.replace("\0", "");
                    debug!("compare between {:?} and {:?}", real_name, pattern_filename);
                    if real_name == pattern_filename {
                        // info!("hit {}", name);
                        if name.ends_with("lost_found") {continue;}//临时添加
                        let temp = if self.name.read().ends_with("/") {
                            alloc::format!("{}{}", self.name.read(), real_name)
                        } else {
                        alloc::format!("{}/{}", self.name.read(), real_name)
                        }; // temp是最后要创造的儿子的名字,使用父节点的名字进行拼接
                        let temp = temp.replace('\0', "");//去除掉“\0”字符
                        let son = Self::new_bare(&self, &temp);
                        children_vec.insert(temp, son.clone());
                        return Some(son);
                    }
                }
                Err(e) => {
                    info!("no valid name {:?}", e);
                }
            }
        }
        None
    }

    /// name为相对路径
    /// 
    /// 增加孩子, 只要父母是合法的, 那就一定会返回Dentry
    /// 即当Inode不存在的时候就会创建Inode
    pub fn add_child(self: Arc<Self>, name: &String, flag: OpenFlags) -> Option<Arc<dyn InodeTrait>> {
        let self_name = self.name.read();
        debug!("[Dentry::add_child] add {} in {}", name, self_name);
        let target_name = if self_name.ends_with("/") {
            format!("{}{}", self_name, name)
        } else {
            format!("{}/{}", self_name, name)
        };
        if let Some(parent_inode) = self.get_inode() {
            if parent_inode.node_type() != InodeType::Dir {
                warn!("[Dentry::add_child] {}: should add child in a dir", self.name.read());
                return None;
            } else {
                if let Some(child_dentry) = self.clone().get_child(&target_name) {
                    // 如果存在这个节点就直接返回获得的Inode
                    return child_dentry.get_inode();
                } else {
                    //如果不存在就创建,使用Inode的do_create方法
                    if let Some(inode) = parent_inode.do_create(&target_name, flag.node_type()) {
                        // self.children.write().insert(target_name.clone(), self.new(&target_name, inode.clone()));
                        // return Some(inode);
                        self.new(&target_name, inode.clone());
                        return Some(inode.clone());
                    } else {
                        // 创建不成功
                        return None;
                    }
                }
            }
        }

        None
    }


    /// 将一个dentry和inode绑定,如果inode是一个文件夹,就把为他的儿子创建一个新的dentry
    fn bind(self: &Arc<Self>, inode:Arc<dyn InodeTrait>) {
        //将inodepush进inode栈,然后flush,注意到这里需要用大括号包裹,不然会死锁
        {
            self.inode.write().push(inode);
        }
        Self::flush_binding(self);
        info!("finished bind");
    }

    pub fn unbind(self: &Arc<Self>) {
        {
            self.inode.write().pop();
        }
        Self::flush_binding(self);
        info!("finished unbind");
    }

    /// 获取dentry的inode栈顶,根据这个栈顶去刷新dentry的children
    fn flush_binding(self: &Arc<Self>) {
        {let mut children_vec = self.children.write();
        children_vec.clear();}
        match self.inode.read().last() {
            None => {
                return;
            }
            Some(inode) => {
                if inode.is_dir() {
                    info!("inode is dir");
                    let dents = inode.read_dents().unwrap();
                    for dent in dents {
                        let name = Vec::from(dent.d_name);
                        match String::from_utf8(name) {
                            Ok(name) => {
                                if name.ends_with("lost_found") {continue;}//临时添加
                                let temp = if self.name.read().ends_with("/") {
                                    alloc::format!("{}{}", self.name.read(), name)
                                } else {
                                    alloc::format!("{}/{}", self.name.read(), name)
                                }; // temp是最后要创造的儿子的名字,使用父节点的名字进行拼接
                                let temp = temp.replace('\0', "");//去除掉“\0”字符
                                let son = Self::new_bare(&self, &temp);
                                son.get_inode();
                                self.children.write().insert(temp, son);
                            }
                            Err(e) => {
                                info!("no valid name {:?}", e);
                            }
                        }
                    }
                }
            }
        }
    }
    
    /// 将一个dentry和一个superblock绑定
    fn mount<T: SuperBlockTrait>(self: &Arc<Self>, sb: T) {
        if sb.root_inode().node_type() != InodeType::Dir {
            info!("you can't mount a inode which is not TYPE DIR");
            return;
        }
        Dentry::bind(self, sb.root_inode());
        info!("bind a superblock to dentry!");
    }
    /// 从一个dentry上获取inode
    fn get_inode(self: &Arc<Self>) -> Option<Arc<dyn InodeTrait>> {
        {
            // info!("[get_inode] {:?}, inode vec len is {}", self.name.read(), self.inode.read().len());
        }
        // 首先检查是否已有 inode（读锁）
        {
            let inode_guard = self.inode.read();
            if let Some(inode) = inode_guard.last() {
                return Some(inode.clone());
            }
        }
        // 如果没有 inode，获取写锁并重新检查（避免并发重复计算）
        let res = {
            let mut inode_guard = self.inode.write();
            // 双重检查，防止在获取写锁期间其他线程已写入 inode
            if let Some(inode) = inode_guard.last() {
                return Some(inode.clone());
            }
            {
                if self.name.read().ends_with("/..") {
                    info!("check in {:?}", *(self.name.read()));
                    let this = Weak::upgrade(&self.parent)?.clone();
                    let parent = Weak::upgrade(&this.parent)?.clone();
                    return parent.get_inode();
                }
            }
            {
                if self.name.read().ends_with("/.") {
                    info!("check in {:?}", *(self.name.read()));
                    return Weak::upgrade(&self.parent)?.clone().get_inode();
                }
            }
            // 获取父节点
            let parent_dentry = self.parent.upgrade()?;
            // 获取父节点的 inode 并执行 walk
            let parent_inode = parent_dentry.get_inode()?;
            let this_inode = parent_inode.walk(&self.name.read())?;
            // 存储 inode 到栈
            inode_guard.push(this_inode.clone());
            this_inode
        };
        // {
        //     self.flush_binding();
        // }
        Some(res)
    }

    pub fn get_inode_from_path(path: &String) -> Option<Arc<dyn InodeTrait>> {
        // if INODE_CACHE.has_inode(path) {
        //     return INODE_CACHE.get(path)
        // }
        // info!("[get_inode_from_path] {}", path);
        if !path.starts_with('/') {
            panic!("path should start with /");
        }
        let mut dentry_now = DENTRY_ROOT.clone();
        if path == "/" {
            return dentry_now.get_inode();
        }
        let mut path_now = String::from("/");

        let path_split = path.split('/').collect::<Vec<_>>();
        for name in path_split {
            if path_now.ends_with("/") {
                path_now = alloc::format!("{}{}", path_now, name);
            } else {
                path_now = alloc::format!("{}/{}", path_now, name);
            }
            match dentry_now.get_child(&path_now) { 
                Some(child) => {
                    dentry_now = child;
                    dentry_now.get_inode();
                }
                None => {
                    debug!("[get_inode_from_path] no such file or directory: {}", path_now);
                    return None;
                }
            }
        }
        // info!("[get_inode_from_path] successful {}", path);
        // if let Some(inode) = dentry_now.get_inode() {
        //     INODE_CACHE.insert(path, inode);
        // };
        dentry_now.get_inode()
    }
    pub fn get_dentry_from_path(path: &String) -> Option<Arc<Self>> {
        if !path.starts_with('/') {
            panic!("path {} should start with /", path);
        }
        let mut dentry_now = DENTRY_ROOT.clone();
        if path == "/" {
            return Some(dentry_now);
        }
        let mut path_now = String::from("/");

        let path_split = path.split('/').collect::<Vec<_>>();
        for name in path_split {
            if path_now.ends_with("/") {
                path_now = alloc::format!("{}{}", path_now, name);
            } else {
                path_now = alloc::format!("{}/{}", path_now, name);
            }
            match dentry_now.get_child(&path_now) { 
                Some(child) => {
                    dentry_now = child;
                    dentry_now.get_inode();
                }
                None => {
                    info!("no such file or directory: {}", path_now);
                    return None;
                }
            }
        }
        Some(dentry_now)
    }

    pub fn hard_link(old: String, new: String) -> bool {
        info!("[hard_link] {} TO {}", &old, &new);
        if let (Some(dentry) ,Some(inode)) = (Self::get_dentry_from_path(&old), Self::get_inode_from_path(&old)) {
            if inode.node_type() == InodeType::Dir {
                error!("[hard link] failed target is dir! {} TO {} ", &old, &new);
                return false;
            }
            let path = Path::string2path(new.clone());
            let parent_path = path.get_parent_abs();
            let child_path_relative = path.get_filename();
            if let Some(parent) = Self::get_dentry_from_path(&parent_path) {
                inode.link(&new);
                info!("[hard_link] inode info: cache: {}, size: {}", inode.get_page_cache().is_none(), inode.get_size());
                if let Some(new_dentry) = parent.clone().get_child(&new) {
                    new_dentry.unbind();
                    new_dentry.bind(inode);
                    true
                } else {
                    parent.new(&child_path_relative, inode);
                    true
                }
            } else {
                error!("[hard link] failed to get target parent {} TO {} ", &old, &new);
                false
            }
        } else {
            error!("[hard link] failed to get origin {} TO {} ", &old, &new);
            false
        }
    }

}


lazy_static! {
    static ref DENTRY_ROOT: Arc<Dentry> = Dentry::new_root();
}

macro_rules! test_inode {
    ($path:expr) => {
        if let Some(inode) = Dentry::get_inode_from_path(&String::from($path)) {
            info!("[test_inode] inode stat for {}: {:?}", $path, inode.fstat());
        } else {
            info!("[test_inode] no such file or directory: {}", $path);
        }
    };
}


pub async fn dentry_test() {
    info!("stat root inode");
    info!("root inode stat is {:?}", root_inode().fstat());
    info!("start dentry test");
    // Dentry::init();
    print!("list all children:   ");
    {DENTRY_ROOT.children.read().iter().for_each(|x| print!("-{}-    ", x.0));println!("");}
    // info!("start unmount");
    // Dentry::unbind(&DENTRY_ROOT);
    // print!("list all children:   ");
    // {DENTRY_ROOT.children.read().iter().for_each(|x| print!("-{}-    ", x.name.read()));println!("");}
    info!("-------------start baisc get_inode test-----------------------------------------");
    {
        info!("test 0");
        if let Some(inode) = Dentry::get_inode_from_path(&String::from("/test_dir0")) {
            info!("inode stat {:?}", inode.fstat());
        } else {
            info!("no such file or directory: /test_dir0");
        }
        info!("test 1");
        if let Some(inode) = Dentry::get_inode_from_path(&String::from("/test_dir0/file_a")) {
            info!("inode stat {:?}", inode.fstat());
        } else {
            info!("no such file or directory: /test_dir0/file_a");
        }
        info!("test 2");
        if let Some(inode) = Dentry::get_inode_from_path(&String::from("/test_dir0/test_dir1/file_b")) {
            info!("inode stat {:?}", inode.fstat());
        } else {
            info!("no such file or directory: /test_dir0/test_dir1/file_b");
        }
    }
    info!("-------------finished baisc get_inode test----------------------------------------");
    info!("-------------start confuse get_inode test-----------------------------------------");
        // info!("test 3");
        // test_inode!("////test_dir0");
        // info!("test 4");
        // test_inode!("//.//../..///.//test_dir0/test_dir1/./file_b");
        // info!("test 5");
        // test_inode!("/musl/basic/mnt/test_mount");
        test_inode!("/musl/busybox");
        test_inode!("/././././././././././././././././no_exist");
        test_inode!("/musl/busybox_testcode.sh");
        test_inode!("/musl");
    info!("-------------finished confuse get_inode test----------------------------------");
    info!("-------------start dentry mkdir test------------------------------------------");
        mkdir("/musl/basic/mnt/test_mkdir", 0);
        test_inode!("/musl/basic/mnt/test_mkdir");
    info!("-------------finished dentry mkdir test---------------------------------------");
    
    info!("-------------start dentry write and read test---------------------------------");
        if let Some(file) = open_file("/musl/basic/mnt/test_mkdir/file_a", OpenFlags::O_CREAT | OpenFlags::O_RDWR) {
            let mut file = file.file().unwrap();
            let buf = alloc::vec![1, 20];
            match file.write(&buf).await {
                Ok(_) => {
                    println!("[dentry] succeed write \n {:?}", buf);
                }
                Err(_) => {
                    println!("[dentry] failed write");
                }
            };
            file.get_inode().sync();
        } else {
            info!("[dentry] open failed");
        };
        if let Some(file) = open_file("/musl/basic/mnt/test_mkdir/file_a", OpenFlags::O_RDWR) {
            let mut file = file.file().unwrap();
            let mut buf = alloc::vec![0, 20];
            match file.read(&mut buf).await {
                Ok(_) => {
                    println!("[dentry] read from /musl/basic/mnt/test_mkdir/file_a \n {:?}", buf);
                }
                Err(_) => {
                    println!("[dentry] failed read");
                }
            } 
        } else {
            info!("[dentry] open failed");
        };
    info!("-------------finished dentry write and read test");
    
    // info!("start get_inode test");
    // {DENTRY_ROOT.children.read().iter().for_each(|x| {
    //     let mut dentry = x;
    //     if let Some(x) = dentry.get_inode() {
    //         info!("finished got inode");
    //         info!("{:?}", x.fstat());
    //     } else {
    //         info!("{} no seccess", dentry.name.read());
    //     }
    // })};
    // {if let Some(x) = DENTRY_ROOT.get_inode() {
    //     info!("root dentry stat is {:?}", x.fstat());
    // }}

    info!("--------------------------------------test hard link--------------------------------------");
    mkdir("/test_dir", 0);
    let flags = OpenFlags::O_RDWR | OpenFlags::O_CREAT;
    if let Some(FileClass::File(file)) = open_file("/test_dir/filea", flags) {
        info!("Successfully create /test_dir/filea");
        let buf = [1; 256];
        file.write(&buf).await;
        info!("/test_dir/filea write\n{:?}", buf);
    } else {
        error!("Failed to create /test_dir/filea");
    }
    match Dentry::hard_link("/test_dir/filea".into(), "/test_dir/fileb".into()) {
        true => {
            info!("link succeed");
        }
        false => {
            error!("link failed");
        }
    }
    if let Some(FileClass::File(file)) = open_file("/test_dir/fileb", OpenFlags::O_RDONLY) {
        info!("Successfully open /test_dir/fileb");
        let mut buf = [0; 256];
        file.read(&mut buf).await;
        info!("/test_dir/fileb read \n{:?}", buf);
    } else {
        error!("Failed to open /test_dir/fileb");
        
    }
    info!("--------------------------------------finish hard link test-------------------------------------------");
    info!("finished dentry test");
    // panic!("dentry test");
}