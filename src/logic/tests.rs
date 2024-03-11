use super::*;
use quote::{quote, ToTokens};
use syn::{parse2, DeriveInput};

fn gen_test_case_2() -> DeriveInput {
    parse2(quote! {
        #[derive(Models, Clone)]
        #[view(UserUpdatables, omit(id), derives(Clone, core::fmt::Debug))]
        struct User {
            id: i32,
            display_name: String,
            bio: String,
            password: String,
        }
    })
    .unwrap()
}

fn get_view_args(attrs: Vec<Attribute>, index: usize) -> Vec<TokenTree> {
    let attr: Attribute = attrs
        .iter()
        .filter(|v| is_attribute(v, "view"))
        .collect::<Vec<&Attribute>>()[index]
        .clone();

    attr.meta
        .require_list()
        .unwrap()
        .to_owned()
        .tokens
        .into_iter()
        .collect::<Vec<TokenTree>>()[2..]
        .to_vec()
}

#[test]
pub fn should_extract_derives() {
    let ast = gen_test_case_2();
    let mut attr_args = get_view_args(ast.attrs, 0);

    let derives = take_path_group("derives", &mut attr_args).expect("Should of found derives");
    assert_eq!(derives[0].to_token_stream().to_string(), "Clone");
    assert_eq!(
        derives[1].to_token_stream().to_string(),
        "core :: fmt :: Debug"
    );
    assert_eq!(derives.len(), 2);
}
