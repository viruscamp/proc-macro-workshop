# 学习 proc-macro

## 资料
- [proc-macro-workshop](https://github.com/dtolnay/proc-macro-workshop)
- [The Little Book of Rust Macros](https://veykril.github.io/tlborm/introduction.html)
- [Rust 宏小册](https://zjp-cn.github.io/tlborm/introduction.html)
- [Rust 学习笔记](https://zjp-cn.github.io/rust-note/index.html)

## 库
- [proc-macro](https://doc.rust-lang.org/stable/proc_macro/) 内部结构，仅能用于
    ```toml
    [lib]
    proc-macro = true
    ```
- [proc-macro2](https://docs.rs/proc-macro2/) 包装 proc-macro 使其可以作为依赖项
- [syn](https:://docs.rs/syn/) 解析输入
- [quote](https://docs.rs/quote/) 简化输出
- [parsel](https://docs.rs/parsel) syn 的高级封装库

## 调试
- [cargo expand](https://crates.io/crates/cargo-expand) 展示输出
- panic 输出
    ```rust
    pub fn my_macro(input: TokenStream) -> TokenStream {
    ...
    let out: TokenStream = .... ;
    panic!("{}", out);
    out
    }
    ```
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
        quote! {
            #vis struct #struct_builder_name {
            }
            impl #struct_name {
            }
        }.into()
    }
    ```
## Tips
### 模式匹配
- `dbg!(attr)` 获取语法树结构，稍微改动即可作为模式匹配
- 解构简化取值，不包括 enum 的可以用
    ```rust
        let Field { ident, ty, attrs, .. } = f;
        for Attribute { meta, .. } in attrs {}
    ```
- `#![feature(let_chains)]` 简化多层 if let
    ```rust
        if let Some(TokenTree::Ident(id)) = tokens_iter.next()
            && id == attr_id_bound
            && let Some(TokenTree::Punct(punct_eq)) = tokens_iter.next()
            && punct_eq.as_char() == '='
            && let Some(bound_val) = tokens_iter.next()                
    ```

### 建议结构
收集 修改 展开
```rust
let mut errors = vec![];
let mut debug_bounds = vec![];
let Mut field_methods = vec![];

// 收集 循环 attrs fields 等, 插入 errors debug_bounds 等

// 修改 在循环内修改 where_clause 等会有所有权问题
let where_clause = input.generics.make_where_clause();
where_clause.predicates.extend(debug_bounds.iter().filter_map(|s|{
    match syn::parse_str::<WherePredicate>(&s.value()) {
        Ok(wp) => Some(wp),
        Err(err) => { errors.push(err); None },
    }
}));

// 展开
let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
let errors = errors.iter().map(Error::to_compile_error);
let expand = quote! {
    #(#errors)*
    impl #impl_generics ::core::fmt::Debug for #struct_name #ty_generics
        #where_clause {
            #(#field_methods)*,
    }
};
```

### 生成的代码尽量使用无歧义的全名 
例如 `::core::option::Option`
### `TokenStream` 非常底层, 同时是输入和输出
- 用 `let mut iter = input.into_iter(); for tt in iter {}` 读取, 没有后退功能
- `TokenTree` 元素有
* `Punct(Punct)` 单字符符号 `+`, `,`, `$`
* `Literal(Literal)` 字面量 character (`'a'`), string (`"hello"`), number (`2.3`) 包含后缀 `3.3f64`
* `Ident(Ident)` 标识符 `let a: u32`内有3个标识符 包括关键字 `let` `for`, 包括 `true` `false` 关键字标识符 `r#let`
* `Group(Group)` 括号包裹的分组, `g.stream()` 获取内部的另一个 `TokenStream`
    * `( ... )` Parenthesis,
    * `{ ... }` Brace,
    * `[ ... ]` Bracket,
    * 没有 `<>` 尖括号
- 有 `.apeend` 和 `.extend` 方法用于在尾部追加
- 通常递归处理
```rust
let new_inner = process(g.stream());
let mut new_group = Group::new(g.delimiter(), group_inner);
new_group.set_span(g.span()); // 重要报错时保留来源位置
output.append(new_group) // 处理后的输出
```
### syn & quote
- quote 中动态字符串， 带""的字符串
    ```rust
    let field_name: Ident = format_ident!("abc");
    let field_name_str = LitStr::new(&field_name.to_string(), field_name.span());
    let fmt_str: LitStr = ...; // quote 中带引号展开
    // stringify!(#field_name) 是在生成之后，编译代码时展开的
    quote! {
        .field(#field_name_str, &format_args!(#fmt_str, &self.#field_name))
        .field(stringify!(#field_name), &format_args!(#fmt_str, &self.#field_name))
    }
    ```
- `Path` is `Ident`
    ```rust
        let attr_id_debug = format_ident!("builder");
        ...
        if let Meta::NameValue(MetaNameValue {
                path,
                ..
            }) = &attr.meta
            && path.is_ident(&attr_id_debug)
    ```
- `TypePath` 尤其是有 qself 的
    - `core::fmt::Debug`  qself=None path="core::fmt::Debug"
    - `<T::Value2 as Trait>::Value`  qself.ty="T::Value2" position=1 path="Trait::Value"
    - `<Vec<T>>::AssociatedItem<X>`  qself.ty="Vec<T>" position=0 path="AssociatedItem<X>"
- `Attribute::Meta`
    ```rust
    for Attribute { meta, .. } in attrs {
        //Meta::Path: `#[abc::def]`
        //Meta::List: `#[derive(Copy, Clone)]` `#[debug(bound = "T::Value: Debug")]`
        //Meta::NameValue: `#[path = "sys/windows.rs"]`
    ```