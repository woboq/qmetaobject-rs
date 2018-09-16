use proc_macro::TokenStream;
use syn;

pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    let crate_ = super::get_crate(&input);

    let values = if let syn::Data::Struct(ref data) = input.data {
        data.fields
            .iter()
            .filter_map(|field| {
                if let syn::Visibility::Public(_) = field.vis {
                    field.ident.clone()
                } else {
                    None
                }
            })
            .collect::<Vec<syn::Ident>>()
    } else {
        panic!("#[derive(SimpleListItem)] is only defined for structs");
    };

    if values.is_empty() {
        panic!("#[derive(SimpleListItem)] only expose public named member, and there are none")
    }

    let arms = values
        .iter()
        .enumerate()
        .map(|(i, ref ident)| {
            let i = i as i32;
            quote!{ #i => QMetaType::to_qvariant(&self.#ident), }
        })
        .collect::<Vec<_>>();

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    quote!(
        impl #impl_generics #crate_::listmodel::SimpleListItem for #name #ty_generics #where_clause {
            fn get(&self, idx : i32) -> QVariant {
                match idx {
                    #(#arms)*
                    _ => QVariant::default()
                }
            }
            fn names() -> Vec<QByteArray> {
                vec![ #(QByteArray::from(stringify!(#values))),* ]
            }
        }
    ).into()
}
