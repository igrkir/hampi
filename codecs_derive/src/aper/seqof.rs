//! `APER` Code generation for ASN.1 OCTET STRING Type

use quote::quote;

use crate::attrs::TyCodecParams;

pub(super) fn generate_aper_decode_for_asn_sequence_of(
    ast: &syn::DeriveInput,
    params: &TyCodecParams,
) -> proc_macro::TokenStream {
    let name = &ast.ident;

    let ty = if let syn::Data::Struct(ref d) = &ast.data {
        match d.fields {
            syn::Fields::Unnamed(ref f) => {
                if f.unnamed.len() == 1 {
                    let first = f.unnamed.first().unwrap();
                    let inner_ty = get_inner_ty_for_vec(first);
                    Some(inner_ty)
                } else {
                    None
                }
            }
            _ => None,
        }
    } else {
        None
    };

    if ty.is_none() {
        return syn::Error::new_spanned(ast, format!("{} Should be a Unit Struct.", name))
            .to_compile_error()
            .into();
    }

    let sz_lb = if params.sz_lb.is_some() {
        let lb = params
            .sz_lb
            .as_ref()
            .unwrap()
            .value()
            .parse::<i128>()
            .unwrap();
        quote! {
            Some(#lb)
        }
    } else {
        quote! {
            None
        }
    };
    let sz_ub = if params.sz_ub.is_some() {
        let ub = params
            .sz_ub
            .as_ref()
            .unwrap()
            .value()
            .parse::<i128>()
            .unwrap();
        quote! {
            Some(#ub)
        }
    } else {
        quote! {
            None
        }
    };
    let sz_ext = if params.sz_ext.is_some() {
        let ext = params.sz_ext.as_ref();
        quote! {
            #ext
        }
    } else {
        quote! {
            false
        }
    };

    let tokens = quote! {

        impl asn_codecs::aper::AperCodec for #name {
            type Output = Self;

            fn decode(data: &mut asn_codecs::aper::AperCodecData) -> Result<Self::Output, asn_codecs::aper::AperCodecError> {
                let length = asn_codecs::aper::decode::decode_length_determinent(data, #sz_lb, #sz_ub, #sz_ext)?;

                let mut items = vec![];
                let mut count = 0;
                loop {
                    items.push(#ty::decode(data)?);
                    count += 1;
                    if count == length {
                        break;
                    }
                }

                Ok(Self(items))
            }
        }
    };

    tokens.into()
}

fn get_inner_ty_for_vec(field: &syn::Field) -> Option<syn::Ident> {
    if let syn::Type::Path(ref tp) = field.ty {
        let type_params = &tp.path.segments.iter().next().unwrap().arguments;
        match type_params {
            syn::PathArguments::AngleBracketed(params) => {
                let generic_args = params.args.iter().next().unwrap();
                if let syn::GenericArgument::Type(ty) = generic_args {
                    if let syn::Type::Path(tpinner) = ty {
                        Some(tpinner.path.segments.iter().next().unwrap().ident.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    } else {
        None
    }
}