//! Path manipulation utilities.
//!
//! This module provides functionality for working with file system paths,
//! including path joining, splitting, and path type checking operations.

use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};
use core::fmt::Debug;
use log::info;

use crate::task::current_task;

/// Represents a file system path.
///
/// This structure wraps a string and provides various operations for path manipulation.
#[derive(Debug, Clone)]
pub struct AbsPath {
    /// The underlying path string
    content: String,
}

impl From<&str> for AbsPath {
    /// 使用这里之前，需要明确一定是绝对路径
    fn from(value: &str) -> Self {
        Self {
            content: value.to_string(),
        }
    }
}

impl AbsPath {
    /// Splits a path into parent and child components using the given delimiter.
    ///
    /// # Arguments
    ///
    /// * `delima` - The delimiter to split on (usually "/")
    ///
    /// # Returns
    ///
    /// A tuple containing (parent_path, child_name) as strings
    ///
    /// # Examples
    ///
    /// ```
    /// let path = AbsPath::new("/home/user/file.txt".to_string());
    /// let (parent, child) = path.split_with("/");
    /// assert_eq!(parent, "/home/user");
    /// assert_eq!(child, "file.txt");
    /// ```
    pub fn split_last_with(&self, delima: &str) -> (String, String) {
        let binding = self.get();
        let s = binding.as_str();
        // info!("split: path = {}, delima = {}", self.get(), delima);
        let (mut parent_path, child_name) = s.rsplit_once(delima).unwrap();
        if parent_path.is_empty() {
            parent_path = "/";
        }
        (parent_path.to_string(), child_name.to_string())
    }

    /// Creates a new Path from a String.
    ///
    /// # Arguments
    ///
    /// * `path` - The string to convert into a Path
    pub fn new(path: String) -> Self {
        Self {
            content: parse_path(path),
        }
    }

    /// Returns a clone of the path's content.
    ///
    /// # Returns
    ///
    /// A String containing the path
    pub fn get(&self) -> String {
        self.content.clone()
    }

    /// Checks if the path represents the root directory ("/").
    ///
    /// # Returns
    ///
    /// `true` if the path is "/", `false` otherwise
    pub fn is_root(&self) -> bool {
        self.content == "/"
    }

    /// Checks if the path is absolute (starts with '/').
    ///
    /// # Returns
    ///
    /// `true` if the path starts with '/', `false` otherwise
    pub fn is_absolute(&self) -> bool {
        self.content.starts_with('/')
    }

    /// 通过一个路径获得文件名
    ///
    /// 使用/对目录进行分割
    ///
    /// 然后获得的最后一个条目就是文件名
    ///
    /// 注意到如果’/‘是最后一个字符返回的是空字符串
    pub fn get_filename(&self) -> String {
        let path = self.content.trim_end_matches('/');
        match path.rfind('/') {
            Some(pos) => {
                if pos + 1 < path.len() {
                    path[pos + 1..].to_string()
                } else {
                    // 如果'/'是最后一个字符，返回空字符串
                    String::new()
                }
            }
            None => {
                // 如果没有'/'，整个路径就是文件名
                path.to_string()
            }
        }
    }

    // 获取到parent的绝对路径，/a/b/c/d结果为/a/b/c
    pub fn get_parent_abs(&self) -> String {
        let (mut parent_path, child_name) = self.split_last_with("/");
        if parent_path == "" {
            parent_path = "/".to_string();
        }
        parent_path
    }
}

/// 处理路径中..和.以及多于的/,可以参考下方的test实例
fn parse_path(path: String) -> String {
    let components: Vec<&str> = path.split("/").collect();
    let mut normalized = Vec::new();
    let is_absolute = path.starts_with('/');

    for comp in components {
        match comp {
            "." => continue,
            ".." => {
                if is_absolute {
                    if !normalized.is_empty() {
                        normalized.pop();
                    }
                } else {
                    // 相对路径下，保留无法返回的..
                    if normalized.last().map_or(true, |&s| s == "..") {
                        normalized.push("..");
                    } else {
                        normalized.pop();
                    }
                }
            }
            "" => continue,
            _ => normalized.push(comp),
        }
    }

    let content = match is_absolute {
        true => format!("/{}", normalized.join("/")),
        false => normalized.join("/"),
    };

    match content.is_empty() {
        true => {
            if is_absolute {
                return "/".to_string();
            } else {
                return ".".to_string();
            }
        }
        false => return content,
    }
}

/// 根据当前的路径和传入的目标路径， 获取目标地址 绝对路径
///
/// base: 是基地址，可以是当前绝对路径或者是其他绝对路径
///
/// path: 是目标路径，可以是绝对路径或相对路径
pub fn resolve_path(base: String, path: String) -> AbsPath {
    // 首先处理一下传入的path, 去掉多于的//、..和.等
    let path = parse_path(path);
    // 已经是绝对路径直接返回,忽略base
    if path.starts_with("/") {
        return AbsPath::new(path);
    }

    // 根据当前路径进行拼接
    let trim_base = base.trim_end_matches("/").to_string();
    let target_abs = format!("{}/{}", trim_base, path);
    let target_abs = parse_path(target_abs);

    AbsPath::new(target_abs)
}

/// Unit tests for the Path implementation.
#[allow(unused)]
pub fn path_test() {
    let p1 = AbsPath::new(String::from("foo/bar"));
    let p2 = AbsPath::new(String::from("/root"));

    assert!(!p1.is_absolute(), "p1 should be relative path");
    assert!(p2.is_absolute(), "p2 should be absolute path");

    let root = AbsPath::new(String::from("/"));
    assert!(root.is_root(), "check root faild");

    // 测试路径规范化
    assert_eq!(parse_path("a/b/../c".to_string()), "a/c");
    assert_eq!(parse_path("a/b/./c".to_string()), "a/b/c");
    assert_eq!(parse_path("../a/../b".to_string()), "../b");
    assert_eq!(parse_path("/./../a".to_string()), "/a");
    assert_eq!(parse_path("//a//b//".to_string()), "/a/b");

    // 测试更多边界情况
    assert_eq!(parse_path("/".to_string()), "/");
    assert_eq!(parse_path("/..".to_string()), "/");
    assert_eq!(parse_path("".to_string()), ".");
    assert_eq!(parse_path(".".to_string()), ".");
    assert_eq!(parse_path("..".to_string()), "..");
    assert_eq!(parse_path("../../a".to_string()), "../../a");
    assert_eq!(parse_path("/a/../../b".to_string()), "/b");

    println!("path_test passed!");
}
