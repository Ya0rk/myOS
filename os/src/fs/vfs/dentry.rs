use alloc::{string::String, vec::Vec};
use bitflags::parser::ParseError;
use hashbrown::HashSet;
use log::info;
use lwext4_rust::bindings::printf;
use spin::Mutex;
use spin::{rwlock::RwLock};
use core::hash::{Hash, Hasher};
use crate::{fs::{ffi::InodeType, path, root_inode, Path, INODE_CACHE}};
use alloc::sync::{Arc, Weak};
use super::{inode, InodeTrait, SuperBlockTrait};


/// 一个目录项,文件树在内存当中的映射
pub struct Dentry {
    ///absolute path should be
    pub name: RwLock<String>, 
    /// 父dentry的弱引用
    pub parent: Weak<Dentry>,
    /// 孩子dentry的强引用
    pub children: RwLock<HashSet<Arc<Dentry>>>,
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
        Self::bind(&DENTRY_ROOT, crate::fs::ext4::SUPER_BLOCK.root_inode());
    }
    /// 创建一个根节点dentry
    fn new_root() -> Arc<Self>{
        // rust语法中解决环形引用的问题
        Arc::new_cyclic(|weak_self| {
            Dentry {
                name: RwLock::new(String::from("/")),
                parent: weak_self.clone(),
                children: RwLock::new(HashSet::new()),
                inode: RwLock::new(Vec::new()),
            }
        })
    }
    /// 创建一个没有绑定Inode的dentry,爹指向self,名字为name
    fn new_bare(
        self: &Arc<Self>,
        name: String,
    ) -> Arc<Self> {
        info!("create bare {}", name);
        let mut inode = Vec::new();
        let res = Self {
            name: RwLock::new(name),
            parent: Arc::downgrade(self),
            children: RwLock::new(HashSet::new()),
            inode: RwLock::new(inode),
        };
        Arc::new(res)
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

    fn unbind(self: &Arc<Self>) {
        {
            self.inode.write().pop();
        }
        Self::flush_binding(self);
        info!("finished unbind");
    }

    /// 获取dentry的inode栈顶,根据这个栈顶去刷新dentry的children
    fn flush_binding(self: &Arc<Self>) {
        let mut children_vec = self.children.write();
        children_vec.clear();
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
                                let son = Self::new_bare(&self, temp);
                                children_vec.insert(son);
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
            info!("exec get_inode {:?}, inode vec len is {}", self.name.read(), self.inode.read().len());
        }
        // 首先检查是否已有 inode（读锁）
        {
            let inode_guard = self.inode.read();
            if let Some(inode) = inode_guard.last() {
                return Some(inode.clone());
            }
        }
        // 如果没有 inode，获取写锁并重新检查（避免并发重复计算）
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
        Some(this_inode)
    }

}


lazy_static! {
    pub static ref DENTRY_ROOT: Arc<Dentry> = Dentry::new_root();
}

pub fn dentry_test() {
    info!("stat root inode");
    info!("root inode stat is {:?}", root_inode().fstat());
    info!("start dentry test");
    Dentry::init();
    print!("list all children:   ");
    {DENTRY_ROOT.children.read().iter().for_each(|x| print!("-{}-    ", x.name.read()));println!("");}
    // info!("start unmount");
    // Dentry::unbind(&DENTRY_ROOT);
    print!("list all children:   ");
    {DENTRY_ROOT.children.read().iter().for_each(|x| print!("-{}-    ", x.name.read()));println!("");}
    info!("start get_inode test");
    {DENTRY_ROOT.children.read().iter().for_each(|x| {
        let mut dentry = x;
        if let Some(x) = dentry.get_inode() {
            info!("finished got inode");
            info!("{:?}", x.fstat());
        } else {
            info!("{} no seccess", dentry.name.read());
        }
    })};
    {if let Some(x) = DENTRY_ROOT.get_inode() {
        info!("root dentry stat is {:?}", x.fstat());
    }}
    info!("finished dentry test");
}