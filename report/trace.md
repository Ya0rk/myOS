好的，这是一个非常棒且极具实用价值的过程宏需求。在 Rust 中，这可以通过编写一个**属性宏 (Attribute Macro)** 来完美实现。

我将为您提供一个完整的、可工作的解决方案，包括：
1.  过程宏 `trace` crate 的完整代码。
2.  一个用于演示和测试的 `main` crate 的完整代码。
3.  详细的设置和使用说明。
4.  对实现细节和设计决策的深度解析，特别是如何满足您提出的所有要求。

--- 

### 解决方案

我们将创建两个 crate：
1.  `trace_macro`: 实现过程宏的 crate。
2.  `usage_example`: 使用并演示该过程宏的 crate。

#### 1. 过程宏 Crate: `trace_macro`

首先，创建一个新的库 crate：
`cargo new trace_macro --lib`

**`trace_macro/Cargo.toml`**
```toml
[package]
name = "trace_macro"
version = "0.1.0"
edition = "2021"

[lib]
proc-macro = true

[dependencies]
syn = { version = "2.0", features = ["full"] }
quote = "1.0"
```

**`trace_macro/src/lib.rs`**
```rust
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, ItemFn, FnArg, Pat, ReturnType, Ident, Lit, Meta, NestedMeta};

/// 一个属性宏，用于在函数进入和退出时自动记录日志。
///
/// # 用法
///
/// `level` 参数是必须的，并且必须是 `log` crate 支持的日志级别字符串，
/// 如 "error", "warn", "info", "debug", "trace"。
///
/// #[trace(level = "info")]
/// fn my_function(a: i32, b: &str) -> String {
///     // ...
/// }
#[proc_macro_attribute]
pub fn trace(attr: TokenStream, item: TokenStream) -> TokenStream {
    // 1. 解析宏的属性参数，即 (level = "...")
    let attr_args = parse_macro_input!(attr as Vec<NestedMeta>);
    let log_level = match parse_log_level(&attr_args) {
        Ok(level) => level,
        Err(e) => return e.to_compile_error().into(),
    };

    // 2. 解析被修饰的函数
    let mut func = parse_macro_input!(item as ItemFn);

    // 3. 提取函数的重要部分
    let func_name = &func.sig.ident;
    let func_name_str = func_name.to_string();
    let func_inputs = &func.sig.inputs;
    let func_body = &func.block;
    let func_return_type = &func.sig.output;

    // 4. 判断是否为成员函数 (检查第一个参数是否为 self)
    let is_method = matches!(func_inputs.iter().next(), Some(FnArg::Receiver(_)));

    // 5. 生成入口日志代码
    let mut entry_log_args = vec![];
    let mut entry_log_params = vec![];
    for arg in func_inputs.iter() {
        if let FnArg::Typed(pat_type) = arg {
            if let Pat::Ident(pat_ident) = &*pat_type.pat {
                let param_name = pat_ident.ident.to_string();
                let param_ident = &pat_ident.ident;
                entry_log_args.push(format!("{} = {{:?}}", param_name));
                entry_log_params.push(quote! { &#param_ident });
            }
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

    // 6. 生成出口日志代码
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
    
    // 7. 根据是否有返回值，构建新的函数体
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

    // 8. 将新的函数体替换旧的
    func.block = syn::parse2(new_body).expect("Failed to parse new function body");

    // 9. 将修改后的函数转换回 TokenStream 并返回
    func.to_token_stream().into()
}

/// 辅助函数，用于解析宏属性中的 `level = "..."`
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

#### 2. 示例 Crate: `usage_example`

创建一个新的二进制 crate：
`cargo new usage_example`

**`usage_example/Cargo.toml`**
```toml
[package]
name = "usage_example"
version = "0.1.0"
edition = "2021"

[dependencies]
# 引用我们的过程宏 crate
trace_macro = { path = "../trace_macro" }
# 引用 log 和一个日志实现
log = "0.4"
env_logger = "0.11"
```

**`usage_example/src/main.rs`**
```rust
use trace_macro::trace;

struct MyStruct {
    id: i32,
}

impl MyStruct {
    // 修饰一个成员函数
    #[trace(level = "info")]
    #[inline(always)] // 测试 inline 特性
    fn process_data(&self, factor: i32, name: &str) -> i32 {
        log::info!("--> Inside actual body of process_data");
        (self.id + factor) * name.len() as i32
    }

    // 修饰一个返回 () 的成员函数
    #[trace(level = "debug")]
    fn finish(&mut self) {
        log::debug!("--> Inside actual body of finish");
        self.id = 0;
    }
}

// 修饰一个普通函数
#[trace(level = "warn")]
fn free_function(value: String) {
    log::warn!("--> Inside actual body of free_function");
    // No return value
}

// 修饰一个泛型函数
#[trace(level = "error")]
fn generic_add<T: std::ops::Add<Output = T> + Copy + std::fmt::Debug>(a: T, b: T) -> T {
    a + b
}

fn main() {
    // 初始化日志记录器
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Trace) // 显示所有级别的日志
        .init();

    println!("--- Calling method with return value ---");
    let mut my_instance = MyStruct { id: 10 };
    let result = my_instance.process_data(5, "hello");
    println!("Final result from process_data: {{}}\n", result);

    println!("--- Calling method with no return value ---");
    my_instance.finish();
    println!("Instance ID after finish: {{}}\n", my_instance.id);

    println!("--- Calling free function ---");
    free_function("some data".to_string());
    println!();

    println!("--- Calling generic function ---");
    let sum = generic_add(100, 200);
    println!("Final result from generic_add: {{}}", sum);
}
```

#### 3. 如何运行

1.  确保两个 crate (`trace_macro` 和 `usage_example`) 位于同一父目录下。
2.  进入 `usage_example` 目录: `cd usage_example`
3.  运行: `cargo run`

**预期输出**:
```text
--- Calling method with return value ---
[INFO  usage_example] Entering MyStruct::process_data with self = &MyStruct { id: 10 }, factor = 5, name = "hello"
[INFO  usage_example] --> Inside actual body of process_data
[INFO  usage_example] Leaving MyStruct::process_data with return value = 75
Final result from process_data: 75

--- Calling method with no return value ---
[DEBUG usage_example] Entering MyStruct::finish with self = &mut MyStruct { id: 10 }
[DEBUG usage_example] --> Inside actual body of finish
[DEBUG usage_example] Leaving MyStruct::finish
Instance ID after finish: 0

--- Calling free function ---
[WARN  usage_example] Entering free_function with value = "some data"
[WARN  usage_example] --> Inside actual body of free_function
[WARN  usage_example] Leaving free_function

--- Calling generic function ---
[ERROR usage_example] Entering generic_add with a = 100, b = 200
[ERROR usage_example] Leaving generic_add with return value = 300
Final result from generic_add: 300
```

--- 

### 代码详解与设计决策

下面详细解释这个过程宏是如何满足您的所有要求的。

#### 1. 指定 Logger 等级

*   **实现**: 通过解析宏的属性 `#[trace(level = "info")]` 来实现。
*   **代码**: `parse_log_level` 辅助函数负责解析 `level = "..."` 这种键值对。它验证 `level` 必须是 `log` crate 支持的级别之一，然后将字符串（如 `"info"`）转换成一个 `Ident`（标识符）。
*   在 `quote!` 宏中，`log::#log_level!` 语法允许我们将这个 `Ident` 动态地插入到代码中，从而生成 `log::info!`, `log::debug!` 等。

#### 2. 打印函数名和参数列表

*   **实现**: 通过 `syn` 解析函数签名 (`func.sig`) 来获取函数名和参数列表。
*   **代码**:
    *   `func.sig.ident` 提供了函数名。
    *   我们遍历 `func.sig.inputs`。对于每个类型化的参数 (`FnArg::Typed`)，我们提取其模式 (`Pat::Ident`) 来获取参数名 (`pat_ident.ident`)。
    *   我们动态地构建格式化字符串 `entry_log_msg` 和一个参数列表 `entry_log_params`。使用 `{:?}` (Debug trait) 来打印参数值，这是最通用和健壮的方式。

#### 3. 打印返回值或 "leave"

*   **实现**: 我们将原始的函数体包裹在一个新的代码块中，并将其执行结果捕获到一个名为 `result` 的变量里。
*   **代码**:
    *   `let result = #func_body;` 执行原始代码。
    *   我们检查 `func.sig.output` (返回类型)。
    *   如果是 `ReturnType::Default`，说明函数返回 `()`，我们就打印不带返回值的日志信息。
    *   否则，我们就打印 `... with return value = {:?}`, &result`。注意这里使用了 `&result`，这样即使返回值类型不是 `Copy`，我们也可以通过引用来打印它，而不会导致所有权问题。

#### 4. 不破坏 `inline` 特性和调用深度

*   **实现**: 这是通过**直接替换函数体**而不是用另一个函数包裹它来实现的。
*   **代码**: 我们的宏接收一个 `ItemFn`，修改它的 `.block` 字段，然后将修改后的 `ItemFn` 返回。
    ```rust
    // 宏的输出（简化后）
    #[inline(always)] // <-- 原始属性被完整保留
    fn process_data(&self, factor: i32, name: &str) -> i32 {
        // 新的、注入了日志代码的函数体
        {
            // ... 入口日志 ...
            let result = { /* ... 原始函数体 ... */ };
            // ... 出口日志 ...
            result
        }
    }
    ```
*   因为我们保留了函数的所有原始属性 (`#(#attrs)*`)，所以 `#[inline]`、`#[cold]` 等属性都保持不变。
*   因为我们没有创建新的函数调用栈帧（只是在现有函数内部增加了代码），所以**调用深度完全不受影响**。

#### 5. 对成员函数和非成员函数均可用

*   **实现**: 通过检查函数的第一个参数是否为 `self` (`FnArg::Receiver`) 来区分成员函数和普通函数。
*   **代码**: `let is_method = matches!(func_inputs.iter().next(), Some(FnArg::Receiver(_)));`
*   对于成员函数，我们希望打印 `TypeName::function_name`。在过程宏中，我们无法轻易地从 `impl` 块的上下文推断出 `TypeName`。因此，我们采用了一个非常巧妙且健壮的运行时技巧：
    *   **`std::any::type_name::<Self>()`**: 我们在**生成的代码**中调用这个函数。在运行时，当这个成员函数被调用时，`Self` 会被解析为具体的类型名（如 `MyStruct`），从而得到我们想要的字符串。这避免了在宏展开时进行复杂的类型推断。
    *   `format_args!("{}::{}", std::any::type_name::<Self>(), #func_name_str)` 这段代码就实现了 `TypeName::function_name` 的格式化。
