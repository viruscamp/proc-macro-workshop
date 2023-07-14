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
在 struct enum union 定义前  
名字定义在 `#[proc_macro_derive(名字)]`, 通常与要实现的 trait 同名  
输出是**追加**到原定义后的，不会修改原定义
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

## 属性式
属性宏是附加到 items 的 属性。  
名字是函数名  
替换被标记的定义
- 使用
    ```rust
    #[show_streams(对应 attr)]
    fn invoke1() {} // 对应 item
    ```
- 定义
    ```rust
    #[proc_macro_attribute]
    pub fn show_streams(attr: proc_macro::TokenStream, item: proc_macro::TokenStream)
        -> proc_macro::TokenStream {
        quote! {

        }.into()
    }
    ```

## 函数式
名字是函数名
```rust
#[proc_macro]
pub fn seq(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    quote! {

    }.into()
}
```

## 卫生性 hygiene
不影响或不受其周围环境的影响
- 不受其周围环境的影响  
    不要在宏内直接使用 `Result<T,E>`, 外部context可能有 `type Result<T> = ::std::result::Result<T, MyError>`  
    考虑到 nostd 环境， 应该用 `::core::result::Result<T,E>`, `::core::option::Option<T>` 
- 不影响其周围环境  
    设计目的之外，没有额外的影响  
    `#derive(Builder) struct Xyz;`的设计目的是生成 `struct XyzBuilder`   
    辅助函数和类型不应该外部可见  
    假设有一个辅助函数 `fn __internal_foo() {}`, 当多次调用 `#derive(Builder) struct Xyz; #derive(Builder) struct Abc;`时，会有两个 `__internal_foo` 报错。  
    尽量不让辅助函数和辅助类型被导出， 无法避免的情况下 `fn __internal_foo_xyz() {} fn __internal_foo_abc() {}` 
- 不可避免的受周围环境的影响  
    `#derive(Builder)` 功能包括 field 为 `a: Vec<i32>` 生成 append 类型的函数  
    但宏内无法确认 `Vec` 确实是 `::std::vec::Vec`  
    未确认: 也许可以引入另一个 crate 其内部用 `trait BuilderField<T> {}` 其泛型默认实现， 以及针对 `Vec` 的特化实现，来实现部分功能  

## span
表示源码的位置，没有 warning 和 error 时，不起作用  
没有注意 span 时，很可能编译错误指向宏使用处 `#[show_streams] fn invoke1() {}`, 无法提供更有意义的消息  
在宏内部创建新的结构时，尽量提供来源的 span  
- 例一
```rust
errors.push(Error::new(value.span(), "invalid, must be an int which > 0"));
```
- 例二
```rust
let mut newid = format_ident!("{id}{replaceby}{postfix}");
newid.set_span(id.span());
```
- 例三
```rust
field_checks.push(quote_spanned ! { bits.span() =>
    const _: [u8; #bits as usize] = [0; #ty::BITS as usize];
});
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

### 编译期条件判断
编译期常数(i32/enum变体)  
--(const fn)--> 有限的编译期常数(true/false/0..8)  
--(一对一)--> 常数泛型类型/常数对应类型  
--(部分实现`判断条件trait`)--> `as 判断条件trait`  
```rust
// 最简单,只能判断正整数类型
const _: [(); #A as usize] = [(); #B as usize];

// 常数泛型类型， 推荐用法
struct StaticBool<const B: bool>;
// 常数对应类型， 不推荐
struct True;
struct False;

// 常数泛型类型 --> 常数对应类型
// 1. 此语法不稳定
impl StaticBool<true> {
    type Target = True;
}
// 2. 需要一个 trait 中转
trait StaticBoolTarget {
    type Target;
}
impl StaticBoolTarget for StaticBool<true> {
    type Target = True;
}
impl StaticBoolTarget for StaticBool<false> {
    type Target = False;
}

// 判断条件trait
trait ShouldAbc {
    const CHECK_CONST: () = ();
}
// 部分实现判断条件trait, 通常只实现 True 就可以
impl ShouldAbc for True {}
impl ShouldAbc for StaticBool<true> {}

// const _ 技巧
const _: () = {
    // 本质上是在报错时用 ShouldAbc 这样的 trait 名称来作为错误信息
    // 最好用 quote_spanned!{} 来提供一个错误来源

    // 推荐方案
    // 1. const fn 替代关联常量, 少一个 as， 报错多一段指向 check 函数
    //   函数必须是 const
    //   不能放在 trait 里面，因为无法标记 const
    //   只能提供类型， 不要求提供值， 即不能作为参数，也不能用常量赋值
    // 2. 直接用 [u8; 0]/StaticBool<true> 而不是再找到 ZeroMod8/True, 再少一个 as, 报错的类型不同
    fn check<T: ShouldAbc>() {}
    check::<StaticBool<{ UserType::SIZE > (UserType::Varian1 as usize) }> >();

    // static_assertion 类似方案，报错不明确，不推荐
    const _: [(); 0 - !{ UserType::SIZE > (UserType::Varian1 as usize) } as usize] = [];

    const _: () =
        <
            <
                StaticBool< // 常数泛型类型
                    { UserType::SIZE > (UserType::Varian1 as usize) } // 编译期常数(i32/enum变体) --> 有限的编译期常数
                > as StaticBoolTarget
            >::Target // 常数泛型类型 --> 常数对应类型, 需要 `as StaticBooleanTarget`
                as ShouldAbc // `as 判断条件trait`, 实际的编译器判断发生在此处
        >::CHECK_CONST; // 为构成合法语句, 还是要调用关联函数或取值关联常数
};
```

### `const _` 技巧
```rust
const _: () = {
    // 一个局部作用域， 可定义，可计算，
    // 定义的类型不会影响外部
    // 在此使用编译期条件判断
    // 内部调用的函数必须为 const fn
    trait Xyz {}
    struct Abc;
    fn f42() {}
};
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
- `quote_spanned! { id.span() => stmt }`
    将 stmt 的报错位置指向 id, 否则会指向宏使用处