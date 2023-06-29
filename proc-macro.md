# 学习 proc-macro

## 资料
- [proc-macro-workshop](https://github.com/dtolnay/proc-macro-workshop)
- [The Little Book of Rust Macros](https://veykril.github.io/tlborm/introduction.html)
- [Rust 宏小册](https://zjp-cn.github.io/tlborm/introduction.html)

## 库
- [proc-macro](https://doc.rust-lang.org/stable/proc_macro/) 内部结构，仅能用于
    ```toml
    [lib]
    proc-macro = true
    ```
- [proc-macro2](https://docs.rs/proc-macro2/) 包装 proc-macro 使其可以作为依赖项
- [syn](https:://docs.rs/syn/) 解析输入
- [quote](https://docs.rs/quote/) 简化输出

## 调试
- [cargo expand](https://crates.io/crates/cargo-expand) 展示输出
- stderr 输出
    ```rust
    eprintln!("TOKENS: {}", tokens);
    ```
- stderr 输出 debug
    ```rust
    eprintln!("INPUT: {:#?}", syntax_tree);
    ```
    ```toml
    # syn 的 Debug 要下面的 feature
    [dependencies]
    syn = { version = "2", features = ["extra-traits"]}
    ```

## Derive 宏
- 使用
    ```rust
    #[derive(Builder)]
    pub struct Command {
    }
    ```
- 最简写法  
    `TokenStream` 过于底层, 甚至不区分关键字和标识符, `struct Abc` 是两个同样数据结构
    ```rust
    use proc_macro::TokenStream;

    #[proc_macro_derive(Builder)] // 名称
    // 不修改输入, 将输出追加到输入之后
    pub fn derive(input: TokenStream) -> TokenStream {
        TokenStream::new()
    }
    ```
- 通用写法
    ```rust
    use proc_macro2::*;
    use quote::*;
    use syn::*;

    #[proc_macro_derive(Builder)]
    pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
        // 解析成高级结构 区分 关键字 标识符 字面量 代码结构
        let input = parse_macro_input!(input as DeriveInput);

        // 分析输入，生成输出要使用的名称等
        let vis = &input.vis;
        let struct_name = &input.ident;
        let struct_builder_name = format_ident!("{struct_name}Builder");

        // 输出
        let expanded = quote! {
            #vis struct #struct_builder_name {
            }
            impl #struct_name {
            }
        }

        proc_macro::TokenStream::from(expanded)
    }
    ```
