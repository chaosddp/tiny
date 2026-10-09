use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;
use syn::{self, Data};

#[proc_macro_derive(IntoLuaTable)]
pub fn into_lua(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let id = ast.ident;

    let Data::Struct(s) = ast.data else {
        panic!("IntoLuaTable derive macro must be used in struct.")
    };

    let mut fields_ast = quote!();

    for (_idx, f) in s.fields.iter().enumerate() {
        let (field_id, _field_ty, _field_vis) = (&f.ident, &f.ty, &f.vis);

        match field_id {
            Some(named_id) => {
                let name = named_id.to_string();
                fields_ast.extend(quote! {
                    table.set(#name, self.#named_id)?;
                });
            }
            None => {
                // do not care about unnamed fields
            }
        }
    }

    quote! {
       impl IntoLua for #id {
            fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
                let table = lua.create_table()?;
                # fields_ast

                Ok(LuaValue::Table(table))
            }
       }
    }
    .into()
}
