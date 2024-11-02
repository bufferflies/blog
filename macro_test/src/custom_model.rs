extern crate proc_macro;

use darling::{FromDeriveInput, FromMeta, util::PathList};
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DataStruct, DeriveInput, Field, Fields, Ident, Path, parse_macro_input};

pub(crate) fn derive_custom_model_impl(input: TokenStream) -> TokenStream {
    // 将输入的 token 流解析为 `DeriveInput`
    let original_struct = parse_macro_input!(input as DeriveInput);

    // 从输入中解构出 data 和 ident 字段
    let DeriveInput { data, ident, .. } = original_struct.clone();

    if let Data::Struct(data_struct) = data {
        // 从这个数据结构中提取字段
        let DataStruct { fields, .. } = data_struct;

        // `darling` 在结构上提供了这个方法让我们方便地解析参数，
        // 并且还能为我们处理错误。
        let args = match CustomModelArgs::from_derive_input(&original_struct) {
            Ok(v) => v,
            Err(e) => {
                // 如果 darling 返回了一个错误，则生成一个
                // token 流，从而使编译器在正确的位置显示错误。
                return TokenStream::from(e.write_errors());
            }
        };

        // 从解析的参数中解构 `models` 字段。
        let CustomModelArgs { models } = args;

        // 创建一个新的输出
        let mut output = quote!();

        // 如果没有定义模型但使用了宏，则触发 panic 错误。
        if models.is_empty() {
            panic!("请使用 `model` 属性至少指定1个模型")
        }

        // 迭代所有定义的模型
        for model in models {
            // 根据目标结构的字段和 `model` 参数生成自定义模型。
            let generated_model = generate_custom_model(&fields, &model);

            // 扩展输出以包含生成的模型
            output.extend(quote!(#generated_model));
        }

        // 将输出转换为 TokenStream 并返回
        output.into()
    } else {
        // 如果目标不是命名结构，则触发 panic 错误
        panic!("DeriveCustomModel 只能用于命名结构")
    }
}

fn generate_custom_model(fields: &Fields, model: &CustomModel) -> proc_macro2::TokenStream {
    let CustomModel {
        name,
        fields: target_fields,
        extra_derives,
    } = model;

    // 创建用于作为输出的变量 new_fields
    let mut new_fields = quote!();

    // 遍历源结构体的所有字段
    for Field {
        // 该字段的标识符
        ident,
        // 该字段的属性
        attrs,
        // 该字段的可见性
        vis,
        // 分隔符 `:`
        colon_token,
        // 该字段的类型
        ty,
        ..
    } in fields
    {
        // 确保该字段有标识符，否则触发 panic 错误
        let Some(ident) = ident else {
            panic!("无法获取字段标识符")
        };

        // 尝试将字段标识符转换为 `Path`，这是由 `syn` 提供的一种类型。
        // 这样做是因为 `darling` 的 PathList 只是一个带有 Path
        // 的集合，并有一些附加方法。
        let path = match Path::from_string(&ident.clone().to_string()) {
            Ok(path) => path,
            Err(error) => panic!("无法将字段标识符转换为 path: {error:?}"),
        };

        // 如果目标字段列表不包含此字段，则跳过
        if !target_fields.contains(&path) {
            continue;
        }

        // 如果包含，则重构字段声明，并将其添加到 `new_fields` 输出中，
        // 以便我们可以在输出结构中使用它。
        new_fields.extend(quote! {
            #(#attrs)*
            #vis #ident #colon_token #ty,
        });
    }

    // 创建一个新的标识符，用于输出结构的名称
    let struct_ident = match Ident::from_string(name) {
        Ok(ident) => ident,
        Err(error) => panic!("{error:?}"),
    };

    // 创建一个 TokenStream，用于保存额外的派生声明
    let mut extra_derives_output = quote!();

    // 如果 extra_derives 不为空，则将其添加到输出中
    if !extra_derives.is_empty() {
        // 这种语法有点紧凑，但你应该已经知道如何理解它。
        extra_derives_output.extend(quote! {
            #(#extra_derives,)*
        })
    }

    // 构造最终的结构体，将所有生成的 TokenStream 组合在一起。
    quote! {
        #[derive(#extra_derives_output)]
        pub struct #struct_ident {
            #new_fields
        }
    }
}

// 为此结构派生 `FromDeriveInput`，该宏由 darling 提供，
// 能够自动添加将参数 token 解析到给定结构中的功能。
#[derive(FromDeriveInput, Clone)]
// 我们告诉 darling，我们正在查找使用 `custom_model`
// 属性定义的参数，并且我们只支持命名结构。
#[darling(attributes(custom_model), supports(struct_named))]
struct CustomModelArgs {
    // 指定生成派生模型的参数。
    // 通过为每个模型重复此属性，可以生成多个模型。
    #[darling(default, multiple, rename = "model")]
    pub models: Vec<CustomModel>,
}

#[derive(FromMeta, Clone)]
struct CustomModel {
    // 生成模型的名称。
    name: String,
    // 逗号分隔的字段标识符列表，
    // 这些字段将包含在生成的模型中。
    fields: PathList,
    // 应对生成的结构应用的额外的派生列表，例如 `Eq` 或 `Hash`。
    #[darling(default)]
    extra_derives: PathList,
}
