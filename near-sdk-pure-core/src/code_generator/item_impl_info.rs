use crate::ItemImplInfo;
use syn::export::TokenStream2;
use syn::Ident;

impl ItemImplInfo {
    /// Generate the code that wraps
    pub fn wrapper_code(&self) -> TokenStream2 {
        let mut res = TokenStream2::new();
        for method in &self.methods {
            if method.is_public || self.is_trait_impl {
                res.extend(method.method_wrapper());
            }
        }
        res
    }

    #[cfg(target_arch = "wasm32")]
    pub fn marshall_code(&self) -> TokenStream2 {
        quote! {}
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn marshall_code(&self) -> TokenStream2 {
        use quote::{format_ident, quote, ToTokens};
        let orig_name = self.ty.clone().into_token_stream();
        let mut name = quote! {Contract};
        if let Ok(input) = syn::parse::<Ident>(orig_name.into()) {
            let new_name = format_ident!("{}Contract", input);
            name = quote! {#new_name};
        };
        let mut res = TokenStream2::new();
        for method in &self.methods {
            if method.is_public {
                res.extend(method.marshal_method());
            }
        }
        quote! {
         impl #name {
           #res
         }
        }
    }
}
// Rustfmt removes comas.
#[rustfmt::skip]
#[cfg(test)]
mod tests {
    use syn::{Type, ImplItemMethod, parse_quote};
    use quote::quote;
    use crate::info_extractor::ImplItemMethodInfo;


    #[test]
    fn trait_implt() {
        let impl_type: Type = syn::parse_str("Hello").unwrap();
        let mut method: ImplItemMethod = syn::parse_str("fn method(&self) { }").unwrap();
        let method_info = ImplItemMethodInfo::new(&mut method, impl_type).unwrap();
        let actual = method_info.method_wrapper();
        let expected = quote!(
            #[cfg(target_arch = "wasm32")]
            #[no_mangle]
            pub extern "C" fn method() {
                near_sdk_pure::env::set_blockchain_interface(::alloc::boxed::Box::new(near_blockchain::NearBlockchain {}));
                let contract: Hello = near_sdk_pure::env::state_read().unwrap_or_default();
                contract.method();
            }
        );
        assert_eq!(expected.to_string(), actual.to_string());
    }

    #[test]
    fn no_args_no_return_no_mut() {
        let impl_type: Type = syn::parse_str("Hello").unwrap();
        let mut method: ImplItemMethod = syn::parse_str("pub fn method(&self) { }").unwrap();
        let method_info = ImplItemMethodInfo::new(&mut method, impl_type).unwrap();
        let actual = method_info.method_wrapper();
        let expected = quote!(
            #[cfg(target_arch = "wasm32")]
            #[no_mangle]
            pub extern "C" fn method() {
                near_sdk_pure::env::set_blockchain_interface(::alloc::boxed::Box::new(near_blockchain::NearBlockchain {}));
                let contract: Hello = near_sdk_pure::env::state_read().unwrap_or_default();
                contract.method();
            }
        );
        assert_eq!(expected.to_string(), actual.to_string());
    }

    #[test]
    fn no_args_no_return_mut() {
        let impl_type: Type = syn::parse_str("Hello").unwrap();
        let mut method: ImplItemMethod = syn::parse_str("pub fn method(&mut self) { }").unwrap();
        let method_info = ImplItemMethodInfo::new(&mut method, impl_type).unwrap();
        let actual = method_info.method_wrapper();
        let expected = quote!(
            #[cfg(target_arch = "wasm32")]
            #[no_mangle]
            pub extern "C" fn method() {
                near_sdk_pure::env::set_blockchain_interface(::alloc::boxed::Box::new(near_blockchain::NearBlockchain {}));
                if near_sdk_pure::env::attached_deposit() != 0 {
                    near_sdk_pure::env::panic("Method method doesn't accept deposit".as_bytes());
                }
                let mut contract: Hello = near_sdk_pure::env::state_read().unwrap_or_default();
                contract.method();
                near_sdk_pure::env::state_write(&contract);
            }
        );
        assert_eq!(expected.to_string(), actual.to_string());
    }

    #[test]
    fn arg_no_return_no_mut() {
        let impl_type: Type = syn::parse_str("Hello").unwrap();
        let mut method: ImplItemMethod = syn::parse_str("pub fn method(&self, k: u64) { }").unwrap();
        let method_info = ImplItemMethodInfo::new(&mut method, impl_type).unwrap();
        let actual = method_info.method_wrapper();
        let expected = quote!(
            #[cfg(target_arch = "wasm32")]
            #[no_mangle]
            pub extern "C" fn method() {
                near_sdk_pure::env::set_blockchain_interface(::alloc::boxed::Box::new(near_blockchain::NearBlockchain {}));
                #[derive(near_sdk_pure :: serde :: Deserialize)]
                #[serde(crate = "near_sdk_pure::serde")]
                struct Input {
                    k: u64,
                }
                let Input { k, }: Input = near_sdk_pure::serde_json::from_slice(
                    &near_sdk_pure::env::input().expect("Expected input since method has arguments.")
                )
                .expect("Failed to deserialize input from JSON.");
                let contract: Hello = near_sdk_pure::env::state_read().unwrap_or_default();
                contract.method(k, );
            }
        );
        assert_eq!(expected.to_string(), actual.to_string());
    }

    #[test]
    fn args_no_return_mut() {
        let impl_type: Type = syn::parse_str("Hello").unwrap();
        let mut method: ImplItemMethod =
            syn::parse_str("pub fn method(&mut self, k: u64, m: Bar) { }").unwrap();
        let method_info = ImplItemMethodInfo::new(&mut method, impl_type).unwrap();
        let actual = method_info.method_wrapper();
        let expected = quote!(
                #[cfg(target_arch = "wasm32")]
                #[no_mangle]
                pub extern "C" fn method() {
                    near_sdk_pure::env::set_blockchain_interface(::alloc::boxed::Box::new(near_blockchain::NearBlockchain {}));
                    if near_sdk_pure::env::attached_deposit() != 0 {
                        near_sdk_pure::env::panic("Method method doesn't accept deposit".as_bytes());
                    }
                    #[derive(near_sdk_pure :: serde :: Deserialize)]
                    #[serde(crate = "near_sdk_pure::serde")]
                    struct Input {
                        k: u64,
                        m: Bar,
                    }
                    let Input { k, m, }: Input = near_sdk_pure::serde_json::from_slice(
                        &near_sdk_pure::env::input().expect("Expected input since method has arguments.")
                    )
                    .expect("Failed to deserialize input from JSON.");
                    let mut contract: Hello = near_sdk_pure::env::state_read().unwrap_or_default();
                    contract.method(k, m, );
                    near_sdk_pure::env::state_write(&contract);
                }
        );
        assert_eq!(expected.to_string(), actual.to_string());
    }

    #[test]
    fn args_return_mut() {
        let impl_type: Type = syn::parse_str("Hello").unwrap();
        let mut method: ImplItemMethod =
            syn::parse_str("pub fn method(&mut self, k: u64, m: Bar) -> Option<u64> { }").unwrap();
        let method_info = ImplItemMethodInfo::new(&mut method, impl_type).unwrap();
        let actual = method_info.method_wrapper();
        let expected = quote!(
                #[cfg(target_arch = "wasm32")]
                #[no_mangle]
                pub extern "C" fn method() {
                    near_sdk_pure::env::set_blockchain_interface(::alloc::boxed::Box::new(near_blockchain::NearBlockchain {}));
                    if near_sdk_pure::env::attached_deposit() != 0 {
                        near_sdk_pure::env::panic("Method method doesn't accept deposit".as_bytes());
                    }
                    #[derive(near_sdk_pure :: serde :: Deserialize)]
                    #[serde(crate = "near_sdk_pure::serde")]
                    struct Input {
                        k: u64,
                        m: Bar,
                    }
                    let Input { k, m, }: Input = near_sdk_pure::serde_json::from_slice(
                        &near_sdk_pure::env::input().expect("Expected input since method has arguments.")
                    )
                    .expect("Failed to deserialize input from JSON.");
                    let mut contract: Hello = near_sdk_pure::env::state_read().unwrap_or_default();
                    let result = contract.method(k, m, );
                    let result =
                        near_sdk_pure::serde_json::to_vec(&result).expect("Failed to serialize the return value using JSON.");
                    near_sdk_pure::env::value_return(&result);
                    near_sdk_pure::env::state_write(&contract);
                }
        );
        assert_eq!(expected.to_string(), actual.to_string());
    }

    #[test]
    fn args_return_ref() {
        let impl_type: Type = syn::parse_str("Hello").unwrap();
        let mut method: ImplItemMethod =
            syn::parse_str("pub fn method(&self) -> &Option<u64> { }").unwrap();
        let method_info = ImplItemMethodInfo::new(&mut method, impl_type).unwrap();
        let actual = method_info.method_wrapper();
        let expected = quote!(
            #[cfg(target_arch = "wasm32")]
            #[no_mangle]
            pub extern "C" fn method() {
                near_sdk_pure::env::set_blockchain_interface(::alloc::boxed::Box::new(near_blockchain::NearBlockchain {}));
                let contract: Hello = near_sdk_pure::env::state_read().unwrap_or_default();
                let result = contract.method();
                let result =
                    near_sdk_pure::serde_json::to_vec(&result).expect("Failed to serialize the return value using JSON.");
                near_sdk_pure::env::value_return(&result);
            }
        );
        assert_eq!(expected.to_string(), actual.to_string());
    }

    #[test]
    fn arg_ref() {
        let impl_type: Type = syn::parse_str("Hello").unwrap();
        let mut method: ImplItemMethod = syn::parse_str("pub fn method(&self, k: &u64) { }").unwrap();
        let method_info = ImplItemMethodInfo::new(&mut method, impl_type).unwrap();
        let actual = method_info.method_wrapper();
        let expected = quote!(
                #[cfg(target_arch = "wasm32")]
                #[no_mangle]
                pub extern "C" fn method() {
                    near_sdk_pure::env::set_blockchain_interface(::alloc::boxed::Box::new(near_blockchain::NearBlockchain {}));
                    #[derive(near_sdk_pure :: serde :: Deserialize)]
                    #[serde(crate = "near_sdk_pure::serde")]
                    struct Input {
                        k: u64,
                    }
                    let Input { k, }: Input = near_sdk_pure::serde_json::from_slice(
                        &near_sdk_pure::env::input().expect("Expected input since method has arguments.")
                    )
                    .expect("Failed to deserialize input from JSON.");
                    let contract: Hello = near_sdk_pure::env::state_read().unwrap_or_default();
                    contract.method(&k, );
                }
        );
        assert_eq!(expected.to_string(), actual.to_string());
    }

    #[test]
    fn arg_mut_ref() {
        let impl_type: Type = syn::parse_str("Hello").unwrap();
        let mut method: ImplItemMethod =
            syn::parse_str("pub fn method(&self, k: &mut u64) { }").unwrap();
        let method_info = ImplItemMethodInfo::new(&mut method, impl_type).unwrap();
        let actual = method_info.method_wrapper();
        let expected = quote!(
            #[cfg(target_arch = "wasm32")]
            #[no_mangle]
            pub extern "C" fn method() {
                near_sdk_pure::env::set_blockchain_interface(::alloc::boxed::Box::new(near_blockchain::NearBlockchain {}));
                #[derive(near_sdk_pure :: serde :: Deserialize)]
                #[serde(crate = "near_sdk_pure::serde")]
                struct Input {
                    k: u64,
                }
                let Input { mut k, }: Input = near_sdk_pure::serde_json::from_slice(
                    &near_sdk_pure::env::input().expect("Expected input since method has arguments.")
                )
                .expect("Failed to deserialize input from JSON.");
                let contract: Hello = near_sdk_pure::env::state_read().unwrap_or_default();
                contract.method(&mut k, );
            }
        );
        assert_eq!(expected.to_string(), actual.to_string());
    }

    #[test]
    fn callback_args() {
        let impl_type: Type = syn::parse_str("Hello").unwrap();
        let mut method: ImplItemMethod = parse_quote! {
            #[private] pub fn method(&self, #[callback] x: &mut u64, y: String, #[callback] z: Vec<u8>) { }
        };
        let method_info = ImplItemMethodInfo::new(&mut method, impl_type).unwrap();
        let actual = method_info.method_wrapper();
        let expected = quote!(
            #[cfg(target_arch = "wasm32")]
            #[no_mangle]
            pub extern "C" fn method() {
                near_sdk_pure::env::set_blockchain_interface(::alloc::boxed::Box::new(near_blockchain::NearBlockchain {}));
                if env::current_account_id() != env::predecessor_account_id() {
                    near_sdk_pure::env::panic("Method method is private".as_bytes());
                }
                #[derive(near_sdk_pure :: serde :: Deserialize)]
                #[serde(crate = "near_sdk_pure::serde")]
                struct Input {
                    y: String,
                }
                let Input { y, }: Input = near_sdk_pure::serde_json::from_slice(
                    &near_sdk_pure::env::input().expect("Expected input since method has arguments.")
                )
                .expect("Failed to deserialize input from JSON.");
                let data: Vec<u8> = match near_sdk_pure::env::promise_result(0u64) {
                    near_sdk_pure::PromiseResult::Successful(x) => x,
                    _ => panic!("Callback computation {} was not successful", 0u64)
                };
                let mut x: u64 =
                    near_sdk_pure::serde_json::from_slice(&data).expect("Failed to deserialize callback using JSON");
                let data: Vec<u8> = match near_sdk_pure::env::promise_result(1u64) {
                    near_sdk_pure::PromiseResult::Successful(x) => x,
                    _ => panic!("Callback computation {} was not successful", 1u64)
                };
                let z: Vec<u8> =
                    near_sdk_pure::serde_json::from_slice(&data).expect("Failed to deserialize callback using JSON");
                let contract: Hello = near_sdk_pure::env::state_read().unwrap_or_default();
                contract.method(&mut x, y, z, );
            }
        );
        assert_eq!(expected.to_string(), actual.to_string());
    }

    #[test]
    fn callback_args_only() {
        let impl_type: Type = syn::parse_str("Hello").unwrap();
        let mut method: ImplItemMethod = parse_quote! {
            #[private] pub fn method(&self, #[callback] x: &mut u64, #[callback] y: String) { }
        };
        let method_info = ImplItemMethodInfo::new(&mut method, impl_type).unwrap();
        let actual = method_info.method_wrapper();
        let expected = quote!(
            #[cfg(target_arch = "wasm32")]
            #[no_mangle]
            pub extern "C" fn method() {
                near_sdk_pure::env::set_blockchain_interface(::alloc::boxed::Box::new(near_blockchain::NearBlockchain {}));
                if env::current_account_id() != env::predecessor_account_id() {
                    near_sdk_pure::env::panic("Method method is private".as_bytes());
                }
                let data: Vec<u8> = match near_sdk_pure::env::promise_result(0u64) {
                    near_sdk_pure::PromiseResult::Successful(x) => x,
                    _ => panic!("Callback computation {} was not successful", 0u64)
                };
                let mut x: u64 =
                    near_sdk_pure::serde_json::from_slice(&data).expect("Failed to deserialize callback using JSON");
                let data: Vec<u8> = match near_sdk_pure::env::promise_result(1u64) {
                    near_sdk_pure::PromiseResult::Successful(x) => x,
                    _ => panic!("Callback computation {} was not successful", 1u64)
                };
                let y: String =
                    near_sdk_pure::serde_json::from_slice(&data).expect("Failed to deserialize callback using JSON");
                let contract: Hello = near_sdk_pure::env::state_read().unwrap_or_default();
                contract.method(&mut x, y, );
            }
        );

        assert_eq!(expected.to_string(), actual.to_string());
    }

    #[test]
    fn callback_args_vec() {
        let impl_type: Type = syn::parse_str("Hello").unwrap();
        let mut method: ImplItemMethod = parse_quote! {
            #[private] pub fn method(&self, #[callback_vec] x: Vec<String>, y: String) { }
        };
        let method_info = ImplItemMethodInfo::new(&mut method, impl_type).unwrap();
        let actual = method_info.method_wrapper();
        let expected = quote!(
            #[cfg(target_arch = "wasm32")]
            #[no_mangle]
            pub extern "C" fn method() {
                near_sdk_pure::env::set_blockchain_interface(::alloc::boxed::Box::new(near_blockchain::NearBlockchain {}));
                if env::current_account_id() != env::predecessor_account_id() {
                    near_sdk_pure::env::panic("Method method is private".as_bytes());
                }
                #[derive(near_sdk_pure :: serde :: Deserialize)]
                #[serde(crate = "near_sdk_pure::serde")]
                struct Input {
                    y: String,
                }
                let Input { y, }: Input = near_sdk_pure::serde_json::from_slice(
                    &near_sdk_pure::env::input().expect("Expected input since method has arguments.")
                )
                .expect("Failed to deserialize input from JSON.");
                let x: Vec<String> = (0..near_sdk_pure::env::promise_results_count())
                    .map(|i| {
                        let data: Vec<u8> = match near_sdk_pure::env::promise_result(i) {
                            near_sdk_pure::PromiseResult::Successful(x) => x,
                            _ => panic!("Callback computation {} was not successful", i)
                        };
                        near_sdk_pure::serde_json::from_slice(&data).expect("Failed to deserialize callback using JSON")
                    })
                    .collect();
                let contract: Hello = near_sdk_pure::env::state_read().unwrap_or_default();
                contract.method(x, y, );
            }
        );
        assert_eq!(expected.to_string(), actual.to_string());
    }

    #[test]
    fn simple_init() {
        let impl_type: Type = syn::parse_str("Hello").unwrap();
        let mut method: ImplItemMethod = parse_quote! {
            #[init]
            pub fn method(k: &mut u64) -> Self { }
        };
        let method_info = ImplItemMethodInfo::new(&mut method, impl_type).unwrap();
        let actual = method_info.method_wrapper();
        let expected = quote!(
            #[cfg(target_arch = "wasm32")]
            #[no_mangle]
            pub extern "C" fn method() {
                near_sdk_pure::env::set_blockchain_interface(::alloc::boxed::Box::new(near_blockchain::NearBlockchain {}));
                if near_sdk_pure::env::attached_deposit() != 0 {
                    near_sdk_pure::env::panic("Method method doesn't accept deposit".as_bytes());
                }
                #[derive(near_sdk_pure :: serde :: Deserialize)]
                #[serde(crate = "near_sdk_pure::serde")]
                struct Input {
                    k: u64,
                }
                let Input { mut k, }: Input = near_sdk_pure::serde_json::from_slice(
                    &near_sdk_pure::env::input().expect("Expected input since method has arguments.")
                )
                .expect("Failed to deserialize input from JSON.");
                let contract = Hello::method(&mut k,);
                near_sdk_pure::env::state_write(&contract);
            }
        );
        assert_eq!(expected.to_string(), actual.to_string());
    }

    #[test]
    fn simple_init_payable() {
        let impl_type: Type = syn::parse_str("Hello").unwrap();
        let mut method: ImplItemMethod = parse_quote! {
            #[init]
            #[payable]
            pub fn method(k: &mut u64) -> Self { }
        };
        let method_info = ImplItemMethodInfo::new(&mut method, impl_type).unwrap();
        let actual = method_info.method_wrapper();
        let expected = quote!(
            #[cfg(target_arch = "wasm32")]
            #[no_mangle]
            pub extern "C" fn method() {
                near_sdk_pure::env::set_blockchain_interface(::alloc::boxed::Box::new(near_blockchain::NearBlockchain {}));
                #[derive(near_sdk_pure :: serde :: Deserialize)]
                #[serde(crate = "near_sdk_pure::serde")]
                struct Input {
                    k: u64,
                }
                let Input { mut k, }: Input = near_sdk_pure::serde_json::from_slice(
                    &near_sdk_pure::env::input().expect("Expected input since method has arguments.")
                )
                .expect("Failed to deserialize input from JSON.");
                let contract = Hello::method(&mut k,);
                near_sdk_pure::env::state_write(&contract);
            }
        );
        assert_eq!(expected.to_string(), actual.to_string());
    }

    #[test]
    fn args_return_mut_borsh() {
        let impl_type: Type = syn::parse_str("Hello").unwrap();
        let mut method: ImplItemMethod = parse_quote! {
            #[result_serializer(borsh)]
            pub fn method(&mut self, #[serializer(borsh)] k: u64, #[serializer(borsh)]m: Bar) -> Option<u64> { }
        };
        let method_info = ImplItemMethodInfo::new(&mut method, impl_type).unwrap();
        let actual = method_info.method_wrapper();
        let expected = quote!(
            #[cfg(target_arch = "wasm32")]
            #[no_mangle]
            pub extern "C" fn method() {
                near_sdk_pure::env::set_blockchain_interface(::alloc::boxed::Box::new(near_blockchain::NearBlockchain {}));
                if near_sdk_pure::env::attached_deposit() != 0 {
                    near_sdk_pure::env::panic("Method method doesn't accept deposit".as_bytes());
                }
                #[derive(near_sdk_pure :: borsh :: BorshDeserialize)]
                struct Input {
                    k: u64,
                    m: Bar,
                }
                let Input { k, m, }: Input = near_sdk_pure::borsh::BorshDeserialize::try_from_slice(
                    &near_sdk_pure::env::input().expect("Expected input since method has arguments.")
                )
                .expect("Failed to deserialize input from Borsh.");
                let mut contract: Hello = near_sdk_pure::env::state_read().unwrap_or_default();
                let result = contract.method(k, m, );
                let result = near_sdk_pure::borsh::BorshSerialize::try_to_vec(&result)
                    .expect("Failed to serialize the return value using Borsh.");
                near_sdk_pure::env::value_return(&result);
                near_sdk_pure::env::state_write(&contract);
            }
        );
        assert_eq!(expected.to_string(), actual.to_string());
    }

    #[test]
    fn callback_args_mixed_serialization() {
        let impl_type: Type = syn::parse_str("Hello").unwrap();
        let mut method: ImplItemMethod = parse_quote! {
            #[private] pub fn method(&self, #[callback] #[serializer(borsh)] x: &mut u64, #[serializer(borsh)] y: String, #[callback] #[serializer(json)] z: Vec<u8>) { }
        };
        let method_info = ImplItemMethodInfo::new(&mut method, impl_type).unwrap();
        let actual = method_info.method_wrapper();
        let expected = quote!(
            #[cfg(target_arch = "wasm32")]
            #[no_mangle]
            pub extern "C" fn method() {
                near_sdk_pure::env::set_blockchain_interface(::alloc::boxed::Box::new(near_blockchain::NearBlockchain {}));
                if env::current_account_id() != env::predecessor_account_id() {
                    near_sdk_pure::env::panic("Method method is private".as_bytes());
                }
                #[derive(near_sdk_pure :: borsh :: BorshDeserialize)]
                struct Input {
                    y: String,
                }
                let Input { y, }: Input = near_sdk_pure::borsh::BorshDeserialize::try_from_slice(
                    &near_sdk_pure::env::input().expect("Expected input since method has arguments.")
                )
                .expect("Failed to deserialize input from Borsh.");
                let data: Vec<u8> = match near_sdk_pure::env::promise_result(0u64) {
                    near_sdk_pure::PromiseResult::Successful(x) => x,
                    _ => panic!("Callback computation {} was not successful", 0u64)
                };
                let mut x: u64 = near_sdk_pure::borsh::BorshDeserialize::try_from_slice(&data)
                    .expect("Failed to deserialize callback using Borsh");
                let data: Vec<u8> = match near_sdk_pure::env::promise_result(1u64) {
                    near_sdk_pure::PromiseResult::Successful(x) => x,
                    _ => panic!("Callback computation {} was not successful", 1u64)
                };
                let z: Vec<u8> =
                    near_sdk_pure::serde_json::from_slice(&data).expect("Failed to deserialize callback using JSON");
                let contract: Hello = near_sdk_pure::env::state_read().unwrap_or_default();
                contract.method(&mut x, y, z, );
            }
        );
        assert_eq!(expected.to_string(), actual.to_string());
    }

    #[test]
    fn no_args_no_return_mut_payable() {
        let impl_type: Type = syn::parse_str("Hello").unwrap();
        let mut method: ImplItemMethod = syn::parse_str("#[payable] pub fn method(&mut self) { }").unwrap();
        let method_info = ImplItemMethodInfo::new(&mut method, impl_type).unwrap();
        let actual = method_info.method_wrapper();
        let expected = quote!(
            #[cfg(target_arch = "wasm32")]
            #[no_mangle]
            pub extern "C" fn method() {
                near_sdk_pure::env::set_blockchain_interface(::alloc::boxed::Box::new(near_blockchain::NearBlockchain {}));
                let mut contract: Hello = near_sdk_pure::env::state_read().unwrap_or_default();
                contract.method();
                near_sdk_pure::env::state_write(&contract);
            }
        );
        assert_eq!(expected.to_string(), actual.to_string());
    }

    #[test]
    fn private_method() {
        let impl_type: Type = syn::parse_str("Hello").unwrap();
        let mut method: ImplItemMethod = syn::parse_str("#[private] pub fn private_method(&mut self) { }").unwrap();
        let method_info = ImplItemMethodInfo::new(&mut method, impl_type).unwrap();
        let actual = method_info.method_wrapper();
        let expected = quote!(
            #[cfg(target_arch = "wasm32")]
            #[no_mangle]
            pub extern "C" fn private_method() {
                near_sdk_pure::env::set_blockchain_interface(::alloc::boxed::Box::new(near_blockchain::NearBlockchain {}));
                if env::current_account_id() != env::predecessor_account_id() {
                    near_sdk_pure::env::panic("Method private_method is private".as_bytes());
                }
                if near_sdk_pure::env::attached_deposit() != 0 {
                    near_sdk_pure::env::panic("Method private_method doesn't accept deposit".as_bytes());
                }
                let mut contract: Hello = near_sdk_pure::env::state_read().unwrap_or_default();
                contract.private_method();
                near_sdk_pure::env::state_write(&contract);
            }
        );
        assert_eq!(expected.to_string(), actual.to_string());
    }

    #[test]
    fn marshall_one_arg() {
        let impl_type: Type = syn::parse_str("Hello").unwrap();
        let mut method: ImplItemMethod = syn::parse_str("pub fn method(&self, k: String) { }").unwrap();
        let method_info = ImplItemMethodInfo::new(&mut method, impl_type).unwrap();
        let actual = method_info.marshal_method();
        let expected = quote!(
                #[cfg(not(target_arch = "wasm32"))]
                pub fn method(&self, k: String,) -> near_sdk_pure::PendingContractTx {
                    let args = near_sdk_pure::serde_json::json!({ "k" : k });
                    near_sdk_pure::PendingContractTx::new(& self . account_id, "method", args, true)
                }
        );
        assert_eq!(expected.to_string(), actual.to_string());
    }
}
