//! Path manipulation utilities.
//! 
//! This module provides functionality for working with file system paths,
//! including path joining, splitting, and path type checking operations.

use core::fmt::Debug;
use alloc::{format, string::{String, ToString}};

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
            content: path
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

    /// Joins a relative path with a base path to create an absolute path.
    /// 
    /// # Arguments
    /// 
    /// * `base` - The base path to join with
    /// 
    /// # Returns
    /// 
    /// A new Path containing the joined absolute path
    /// 
    /// # Examples
    /// 
    /// ```
    /// let path = Path::string2path("foo/bar".to_string());
    /// let joined = path.join_path_2_absolute("/home/user".to_string());
    /// assert_eq!(joined.get(), "/home/user/foo/bar");
    /// ```
    pub fn join_path_2_absolute(&self, base: String) -> Self {
        if self.is_absolute() {
            return Self {
                content: self.content.clone(),
            };
        }
    
        let base = base.trim_end_matches('/');
    
        let new_path = if base.is_empty() || base == "/" {
            format!("/{}", self.content)
        } else {
            format!("{}/{}", base, self.content)
        };
    
        Self {
            content: new_path,
        }
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

    let base = String::from("/home/user/");
    let joined = p1.join_path_2_absolute(base.clone());
    assert_eq!(joined.get(), "/home/user/foo/bar", "join path faid");

    let root = Path::string2path(String::from("/"));
    assert!(root.is_root(), "check root faild");

    println!("path_test passed!");
}