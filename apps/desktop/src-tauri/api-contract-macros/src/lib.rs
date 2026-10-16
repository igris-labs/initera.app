use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, FnArg, ItemFn, Pat, ReturnType};

#[proc_macro_attribute]
pub fn command(attribute: TokenStream, item: TokenStream) -> TokenStream {
    let forced_mutation = attribute.to_string() == "mutation";
    if !attribute.is_empty() && !forced_mutation {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            "Expected #[command] or #[command(mutation)]",
        )
        .to_compile_error()
        .into();
    }
    let function = parse_macro_input!(item as ItemFn);
    let name = &function.sig.ident;
    let wrapper = format_ident!("{}_api", name);
    let input_name = format_ident!("{}Input", name);

    let mut fields = Vec::new();
    let mut arguments = Vec::new();
    let mut has_mutation = false;
    let mut shared_mutation = false;

    for argument in &function.sig.inputs {
        let FnArg::Typed(argument) = argument else {
            return syn::Error::new_spanned(argument, "API commands cannot have a receiver")
                .to_compile_error()
                .into();
        };
        let Pat::Ident(pattern) = argument.pat.as_ref() else {
            return syn::Error::new_spanned(argument, "API command arguments must be named")
                .to_compile_error()
                .into();
        };
        let ident = &pattern.ident;
        if ident == "app" {
            arguments.push(quote! { app.clone() });
        } else {
            let ty = &argument.ty;
            fields.push(quote! { #ident: #ty });
            arguments.push(quote! { input.#ident });
            has_mutation |= ident == "mutation";
            if ident == "mutation" {
                shared_mutation = quote!(#ty).to_string() == "MutationRequest";
            }
        }
    }

    let ReturnType::Type(_, result_type) = &function.sig.output else {
        return syn::Error::new_spanned(&function.sig, "API commands must return Result")
            .to_compile_error()
            .into();
    };
    let syn::Type::Path(path) = result_type.as_ref() else {
        return syn::Error::new_spanned(result_type, "API commands must return Result")
            .to_compile_error()
            .into();
    };
    let Some(segment) = path.path.segments.last() else {
        unreachable!();
    };
    let syn::PathArguments::AngleBracketed(generics) = &segment.arguments else {
        return syn::Error::new_spanned(result_type, "API commands must return Result")
            .to_compile_error()
            .into();
    };
    let Some(syn::GenericArgument::Type(data_type)) = generics.args.first() else {
        return syn::Error::new_spanned(result_type, "API commands must return Result<T, String>")
            .to_compile_error()
            .into();
    };

    let mutation_required = has_mutation || forced_mutation;
    let restart = matches!(
        name.to_string().as_str(),
        "storage_restore_backup" | "storage_finish_import_upload"
    )
    .then(|| quote! { crate::api_contract::restart_after_response(app); });
    let validate_payload = has_mutation.then(|| {
        quote! {
            request.validate_mutation_payload(
                &app,
                stringify!(#name),
                &serde_json::to_value(&request.input.mutation)
                    .map_err(|_| crate::api_contract::ApiError::from("Cannot encode mutation.".to_owned()))?
            )?;
        }
    });
    let bind_input = if shared_mutation {
        quote! {
            let mut input = request.input;
            input.mutation.expected_versions = expected_versions.clone().unwrap_or_default();
        }
    } else {
        quote! { let input = request.input; }
    };
    let invoke_command = if forced_mutation {
        quote! {
            let data = crate::api_contract::apply_side_effect(
                &app,
                stringify!(#name),
                mutation_id.as_deref().unwrap(),
                mutation_input.as_ref().unwrap(),
                || #name(#(#arguments),*),
            ).map_err(|error| crate::api_contract::enrich_command_error(
                &app,
                error,
                expected_versions.as_ref(),
            ))?;
        }
    } else {
        quote! {
            let data = #name(#(#arguments),*)
                .map_err(|error| crate::api_contract::command_error(
                    &app,
                    error,
                    expected_versions.as_ref(),
                ))?;
        }
    };

    quote! {
        #function

        #[allow(non_camel_case_types)]
        #[derive(serde::Deserialize, serde::Serialize)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        pub(crate) struct #input_name {
            #(#fields,)*
        }

        #[tauri::command]
        pub(crate) fn #wrapper(
            app: tauri::AppHandle,
            request: Option<serde_json::Value>,
        ) -> Result<crate::api_contract::ApiResponse<#data_type>, crate::api_contract::ApiError> {
            let request: crate::api_contract::ApiRequest<#input_name> =
                serde_json::from_value(request.ok_or_else(||
                    crate::api_contract::ApiError::validation("Request envelope is required."))?)
                .map_err(|_| crate::api_contract::ApiError::validation("Invalid request envelope or input."))?;
            request.validate(#mutation_required)?;
            #validate_payload
            let mutation_id = request.mutation_id().map(str::to_owned);
            let expected_versions = request.expected_versions().cloned();
            let mutation_input = mutation_id.as_ref().map(|_| serde_json::to_value(&request.input))
                .transpose()
                .map_err(|_| crate::api_contract::ApiError::validation("Cannot encode mutation input."))?;
            #bind_input
            let _change_capture = #mutation_required.then(crate::api_contract::begin_change_capture);
            #invoke_command
            let response = crate::api_contract::respond(
                &app,
                data,
                mutation_id.as_deref(),
                expected_versions.as_ref(),
                stringify!(#name),
                mutation_input.as_ref(),
            )?;
            #restart
            Ok(response)
        }
    }
    .into()
}
