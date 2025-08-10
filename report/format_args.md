当然可以。这是一个非常棒的增强功能，它能让这个宏变得更加灵活和强大。为了实现这个功能，我们需要引入一个**辅助属性**，让它可以被放置在函数参数上，然后由我们的主 `#[trace]` 宏来解析和使用。

我们将这个新的辅助属性命名为 `#[format("...")]`。

下面是更新后的完整解决方案。

---

### 解决方案

#### 1. 过程宏 Crate: `trace_macro` (已更新)

`Cargo.toml` 文件保持不变。

**`trace_macro/src/lib.rs` (已更新)**
```rust
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, ItemFn, FnArg, Pat, ReturnType, Ident, Lit, Meta, NestedMeta};

/// 一个属性宏，用于在函数进入和退出时自动记录日志。
///
/// # 用法
///
/// `level` 参数是必须的，并且必须是 `log` crate 支持的日志级别字符串。
///
/// 可以为每个参数单独指定格式化字符串，通过 `#[format("...")]` 属性。
/// 如果不指定，默认使用 `"{::?}"` (Debug trait)。
///
/// #[trace(level = "info")]
/// fn my_function(
///     #[format("{}", a)] a: impl std::fmt::Display, 
///     #[format("{:#X}", b)] b: i32
/// ) -> String {
///     // ...
/// }
#[proc_macro_attribute]
pub fn trace(attr: TokenStream, item: TokenStream) -> TokenStream {
    // 1. 解析宏的属性参数 (level = "...")
    let attr_args = parse_macro_input!(attr as Vec<NestedMeta>);
    let log_level = match parse_log_level(&attr_args) {
        Ok(level) => level,
        Err(e) => return e.to_compile_error().into(),
    };

    // 2. 解析被修饰的函数 (需要可变访问以移除我们的辅助属性)
    let mut func = parse_macro_input!(item as ItemFn);

    // 3. 提取函数的重要部分
    let func_name = &func.sig.ident;
    let func_name_str = func_name.to_string();
    let func_body = &func.block;
    let func_return_type = &func.sig.output;

    // 4. 判断是否为成员函数
    let is_method = matches!(func.sig.inputs.iter().next(), Some(FnArg::Receiver(_)));

    // 5. 生成入口日志代码 (已更新)
    let mut entry_log_args = vec![];
    let mut entry_log_params = vec![];

    // 使用 iter_mut() 以便我们可以修改参数 (移除我们的属性)
    for arg in func.sig.inputs.iter_mut() {
        if let FnArg::Typed(pat_type) = arg {
            if let Pat::Ident(pat_ident) = &*pat_type.pat {
                let param_name = pat_ident.ident.to_string();
                let param_ident = &pat_ident.ident;

                // 默认格式化为 Debug
                let mut format_spec = "{:?}".to_string();

                // 查找并使用 #[format("...")] 属性，然后移除它
                pat_type.attrs.retain(|attr| {
                    if attr.path.is_ident("format") {
                        if let Ok(lit_str) = attr.parse_args::<syn::LitStr>() {
                            format_spec = lit_str.value();
                        }
                        // 返回 false 表示移除此属性
                        false
                    } else {
                        // 保留所有其他属性 (如 #[allow(...)])
                        true
                    }
                });

                entry_log_args.push(format!("{} = {{}}", param_name, format_spec));
                entry_log_params.push(quote! { &#param_ident });
            }
        } else if let FnArg::Receiver(receiver) = arg {
            // 对 self 参数的特殊处理
            let param_name = if receiver.mutability.is_some() { "self" } else { "self" };
            entry_log_args.push(format!("{} = {{:?}}", param_name));
            entry_log_params.push(quote! { &self });
        }
    }

    let entry_log_msg = if entry_log_args.is_empty() {
        format!("Entering {{}}")
    } else {
        format!("Entering {{}} with {}", entry_log_args.join(", "))
    };

    let entry_log = if is_method {
        quote! {
            log::#log_level!(#entry_log_msg, format_args!("{}::{}", std::any::type_name::<Self>(), #func_name_str), #(#entry_log_params),*);
        }
    } else {
        quote! {
            log::#log_level!(#entry_log_msg, #func_name_str, #(#entry_log_params),*);
        }
    };

    // 6. 生成出口日志代码 (与之前相同)
    let exit_log = if is_method {
        match func_return_type {
            ReturnType::Default => quote! {
                log::#log_level!("Leaving {}::{}", std::any::type_name::<Self>(), #func_name_str);
            },
            _ => quote! {
                log::#log_level!("Leaving {}::{} with return value = {{:?}}", std::any::type_name::<Self>(), #func_name_str, &result);
            },
        }
    } else {
        match func_return_type {
            ReturnType::Default => quote! {
                log::#log_level!("Leaving {}", #func_name_str);
            },
            _ => quote! {
                log::#log_level!("Leaving {} with return value = {{:?}}", #func_name_str, &result);
            },
        }
    };
    
    // 7. 构建新的函数体 (与之前相同)
    let new_body = match func_return_type {
        ReturnType::Default => {
            quote! {
                {
                    #entry_log
                    let _ = #func_body;
                    #exit_log
                }
            }
        }
        _ => {
            quote! {
                {
                    #entry_log
                    let result = #func_body;
                    #exit_log
                    result
                }
            }
        }
    };

    // 8. 替换函数体
    func.block = syn::parse2(new_body).expect("Failed to parse new function body");

    // 9. 返回修改后的函数
    func.to_token_stream().into()
}

// `parse_log_level` 辅助函数保持不变
fn parse_log_level(args: &[NestedMeta]) -> Result<Ident, syn::Error> {
    if args.len() != 1 {
        return Err(syn::Error::new_spanned(&args[0], "Expected exactly one argument: `level = \"...\"`"));
    }

    if let NestedMeta::Meta(Meta::NameValue(nv)) = &args[0] {
        if nv.path.is_ident("level") {
            if let Lit::Str(lit_str) = &nv.lit {
                let level_str = lit_str.value().to_lowercase();
                match level_str.as_str() {
                    "error" | "warn" | "info" | "debug" | "trace" => {
                        return Ok(Ident::new(&level_str, lit_str.span()));
                    }
                    _ => return Err(syn::Error::new_spanned(&nv.lit, "Invalid log level. Must be one of: error, warn, info, debug, trace.")),
                }
            }
        }
    }

    Err(syn::Error::new_spanned(&args[0], "Invalid attribute format. Expected `level = \"...\"`"))
}
```

#### 2. 示例 Crate: `usage_example` (已更新)

`Cargo.toml` 文件保持不变。

**`usage_example/src/main.rs` (已更新)**
```rust
use std::fmt;
use trace_macro::trace;

// 定义一个自定义结构体，用于演示不同的格式化
struct Point {
    x: i32,
    y: i32,
}

// 为 Point 实现 Debug trait ({:?})
impl fmt::Debug for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Point")
         .field("x", &self.x)
         .field("y", &self.y)
         .finish()
    }
}

// 为 Point 实现 Display trait ({})
impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

// 演示自定义格式化的函数
#[trace(level = "info")]
fn process_point(
    // 使用 Display trait ({}) 格式化
    #[format("{}")]
    p: Point,
    // 使用默认的 Debug trait ({:?}) 格式化 (无属性)
    id: usize,
    // 使用十六进制格式化
    #[format("{:#X}")]
    flags: u32,
) {
    log::info!("--> Inside actual body of process_point");
}

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Trace)
        .init();

    println!("--- Calling function with custom format specifiers ---");
    let my_point = Point { x: 10, y: 20 };
    process_point(my_point, 123, 255);
}
```

#### 3. 如何运行

步骤与之前完全相同。
1.  `cd usage_example`
2.  `cargo run`

**预期输出**:
```text
--- Calling function with custom format specifiers ---
[INFO  usage_example] Entering process_point with p = (10, 20), id = 123, flags = 0xFF
[INFO  usage_example] --> Inside actual body of process_point
[INFO  usage_example] Leaving process_point
```
观察输出可以发现：
*   参数 `p` 被打印为 `(10, 20)`，这正是 `Display` trait (`{}`) 的输出。
*   参数 `id` 被打印为 `123`，这是 `Debug` trait (`{:?}`) 对 `usize` 的默认输出。
*   参数 `flags` 被打印为 `0xFF`，这正是十六进制格式化 (`{:#X}`) 的输出。

---

### 代码详解与设计决策 (新增部分)

#### 如何实现参数级格式化？

1.  **辅助属性 (`#[format("...")]`)**:
    *   我们引入了一个新的属性 `#[format("...")]`，它本身不是一个过程宏，而是一个普通的属性。它的唯一目的就是作为元数据，被我们的 `#[trace]` 宏读取。
    *   这种“宏消费另一个属性”的模式是 Rust 宏系统非常强大和灵活的体现。

2.  **解析并移除属性**:
    *   在 `trace` 宏的实现中，我们现在需要对函数的 `inputs` 进行**可变**遍历 (`iter_mut()`)。
    *   对于每个参数，我们遍历其上的所有属性 (`pat_type.attrs`)。
    *   我们检查属性的路径是否是 `format`。如果是，我们就解析它括号内的字符串字面量，并将其保存为这个参数的 `format_spec`。
    *   **关键一步**: 我们使用 `Vec::retain` 方法。这个方法会遍历属性列表，并只保留那些让闭包返回 `true` 的元素。在我们的闭包中，当找到 `format` 属性时，我们返回 `false`，从而**将其从函数的 AST (抽象语法树) 中移除**。
    *   **为什么必须移除？** 因为 `#[format]` 属性对于 `rustc` 编译器来说是未知的。如果我们不移除它，宏展开后的代码会包含一个编译器不认识的属性，从而导致编译错误 "error: unknown attribute `format`"。通过在宏内部消费并移除它，我们保证了最终生成的代码是干净和合法的。

3.  **默认行为**:
    *   为了方便使用，我们为 `format_spec` 设置了一个默认值 `"{::?}"`。
    *   这意味着，如果一个参数没有 `#[format]` 属性，它会自动回退到使用 `Debug` trait 进行打印，这与之前的行为保持一致，并且对于绝大多数类型都是有效的。

通过这种方式，我们以一种高度符合 Rust 语言习惯、类型安全且对用户友好的方式，成功地为宏增加了参数级别的配置能力。
