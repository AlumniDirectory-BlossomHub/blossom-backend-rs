use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{quote, ToTokens};
use syn::fold::Fold;
use syn::{parse_macro_input, parse_quote, Expr, ExprCall, Field, Fields, ItemStruct};

#[doc(hidden)]
struct FirstCallModifier {
    modified: bool,
}

#[doc(hidden)]
impl Fold for FirstCallModifier {
    // 处理普通函数调用
    fn fold_expr_call(&mut self, call: ExprCall) -> ExprCall {
        if self.modified {
            return call;
        }

        // 标记已修改并返回新调用
        self.modified = true;
        let mut new_call = call;
        new_call.args.insert(0, parse_quote! {&v});
        new_call
    }

    // 处理方法调用（确保优先遍历接收者）
    fn fold_expr_method_call(&mut self, mut call: syn::ExprMethodCall) -> syn::ExprMethodCall {
        if self.modified {
            return call;
        }

        // 优先处理接收者（可能包含目标调用）
        call.receiver = Box::new(self.fold_expr(*call.receiver));
        call
    }
}

/// Generate partial and full form
///
///
/// Examples:
/// ```
/// # use partial_form_codegen::generate_partial_form;
/// # use rocket::FromForm;
/// # use rocket::fs::TempFile;
///
/// generate_partial_form! {
///     #[derive(Debug, FromForm)]
///     struct UpdateProfileReq<'r> {
///         #[validate(len(2..32).or_else(msg!("Username length must be between 2 and 32 characters")))]
///         username: String,
///         avatar: TempFile<'r>,
///     }
///  }
/// ```
/// it will be expanded to
/// ```
/// # use partial_form_codegen::generate_partial_form;
/// # use rocket::FromForm;
/// # use rocket::fs::TempFile;
///  #[derive(Debug, FromForm)]
///  struct UpdateProfileReq<'r> {
///     #[field(validate=len(2..32).or_else(msg!("Username length must be between 2 and 32 characters")))]
///     username: String,
///     avatar: TempFile<'r>,
///  }
///  #[derive(Debug, FromForm)]
///  struct PartialUpdateProfileReq<'r> {
///     #[field(validate=(
///         |x:&Option<_>| match x {
///             Some(v)=>len(&v, 2..32).or_else(msg!("Username length must be between 2 and 32 characters")),
///             None=>Ok(()),
///         })( )
///     )]
///     avatar: Option<TempFile<'r>>,
///  }
/// ```
///
#[proc_macro]
pub fn generate_partial_form(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemStruct);
    let generics = input.generics.clone();
    let struct_name = &input.ident;
    let partial_struct_name = Ident::new(&format!("Partial{}", struct_name), struct_name.span());
    let struct_attrs = &input.attrs;

    let fields = match input.fields {
        Fields::Named(named_fields) => named_fields.named,
        _ => {
            return syn::Error::new_spanned(input, "Only named struct fields are supported")
                .to_compile_error()
                .into();
        }
    };

    let mut new_fields = vec![];
    let mut partial_fields = vec![];

    for field in fields {
        let Field {
            attrs,
            vis,
            ident,
            colon_token,
            ty,
            mutability: _,
        } = field;

        let mut new_attrs = vec![];
        let mut partial_attrs = Vec::new();

        for attr in attrs {
            if attr.path().is_ident("validate") {
                // 提取 validate(...) 里面的表达式
                let validate_expr: Expr = match attr.parse_args() {
                    Ok(expr) => expr,
                    Err(err) => return err.to_compile_error().into(),
                };

                new_attrs.push(quote! {
                    #[field(validate = #validate_expr)]
                });

                let mut modifier = FirstCallModifier { modified: false };
                let modified_expr = modifier.fold_expr(validate_expr);

                partial_attrs.push(quote! {
                    #[field(validate = (|x: &Option<_>| match x {
                        Some(v) => #modified_expr,
                        None => Ok(()),
                    })())]
                });
            } else {
                new_attrs.push(attr.to_token_stream());
                partial_attrs.push(attr.to_token_stream());
            }
        }

        new_fields.push(quote! {
            #(#new_attrs)*
            #vis #ident #colon_token #ty,
        });
        partial_fields.push(quote! {
            #(#partial_attrs)*
            #vis #ident #colon_token Option<#ty>,
        })
    }

    let expanded = quote! {
        #(#struct_attrs)*
        struct #struct_name #generics {
            #(#new_fields)*
        }

        #(#struct_attrs)*
        struct #partial_struct_name #generics {
            #(#partial_fields)*
        }
    };

    expanded.into()
}
