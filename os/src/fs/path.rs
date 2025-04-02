//! Path manipulation utilities.
//! 
//! This module provides functionality for working with file system paths,
//! including path joining, splitting, and path type checking operations.

use core::fmt::Debug;
use alloc::{format, string::{String, ToString}, vec::Vec};

/// Represents a file system path.
/// 
/// This structure wraps a string and provides various operations for path manipulation.
#[derive(Debug)]
pub struct Path {
    /// The underlying path string
    content: String,
}

impl Path {
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
    /// let path = Path::string2path("/home/user/file.txt".to_string());
    /// let (parent, child) = path.split_with("/");
    /// assert_eq!(parent, "/home/user");
    /// assert_eq!(child, "file.txt");
    /// ```
    pub fn split_with(self, delima: &str) -> (String, String) {
        let binding = self.get();
        let s = binding.as_str();
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
    pub fn string2path(path: String) -> Self {
        Self {
            content: parse_path(path)
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
}

/// Unit tests for the Path implementation.
#[allow(unused)]
pub fn path_test() {
    let p1 = Path::string2path(String::from("foo/bar"));
    let p2 = Path::string2path(String::from("/root"));

    assert!(!p1.is_absolute(), "p1 should be relative path");
    assert!(p2.is_absolute(), "p2 should be absolute path");

    let root = Path::string2path(String::from("/"));
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

fn parse_path(path: String) -> String {
    let components: Vec<&str> = path.split("/").collect();
    let mut normalized = Vec::new();
    let is_absolute = path.starts_with('/');

    for comp in components {
        match comp {
            "."  => continue,
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
            _  => normalized.push(comp),
        }
    }

    let content = match is_absolute {
        true  => format!("/{}", normalized.join("/")),
        false => normalized.join("/")
    };

    match content.is_empty() {
        true  => {
            if is_absolute { return "/".to_string(); }
            else { return ".".to_string(); }
        }
        false => return content,
    }
}

pub fn join_path_2_absolute(base: String, suffix: String) -> String {
    if suffix.starts_with("/") {
        return suffix;
    }
    let trim_base = base.trim_end_matches("/").to_string();
    let temp = format!("{}/{}", trim_base, suffix);
    let content = parse_path(temp);

    content
}