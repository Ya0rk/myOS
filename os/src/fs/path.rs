use core::fmt::Debug;
use alloc::{format, string::{String, ToString}};

#[derive(Debug)]
pub struct Path {
    content: String,
}

impl Path{
    // TODO :优化
    pub fn split_with(self, delima: &str) -> (String, String) {
        let binding = self.get();
        let s = binding.as_str();
        let (mut parent_path, child_name) = s.rsplit_once(delima).unwrap();
        if parent_path.is_empty() {
            parent_path = "/";
        }
        (parent_path.to_string(), child_name.to_string())
    }
    pub fn string2path(path: String) -> Self{
        Self {
            content: path
        }
    }

    pub fn get(&self) -> String {
        self.content.clone()
    }

    pub fn is_root(&self) -> bool {
        self.content == "/"
    }

    pub fn is_absolute(&self) -> bool {
        self.content.starts_with('/')
    }

    pub fn join_path_2_absolute(&self, base: String) -> Self{
        if self.is_absolute() {
            return Self {
                content: self.content.clone(),
            };
        }
    
        // 去除基准路径末尾的 `/`
        let base = base.trim_end_matches('/');
    
        // 拼接路径
        let new_path = if base.is_empty() || base == "/" {
            format!("/{}", self.content)
        } else {
            format!("{}/{}", base, self.content)
        };
    
        Self {
            content: new_path,
        }
    }
}

#[allow(unused)]
pub fn path_test() {
    let p1 = Path::string2path(String::from("foo/bar"));
    let p2 = Path::string2path(String::from("/root"));

    assert!(!p1.is_absolute(), "p1 should be relative path");
    assert!(p2.is_absolute(), "p2 should be absolute path");

    let base = String::from("/home/user/");
    let joined = p1.join_path_2_absolute(base.clone());
    assert_eq!(joined.get(), "/home/user/foo/bar", "join path faid");

    let root = Path::string2path(String::from("/"));
    assert!(root.is_root(), "check root faild");

    println!("path_test passed!");
}