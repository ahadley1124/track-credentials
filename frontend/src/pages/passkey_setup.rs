use yew::prelude::*;
use yew_router::prelude::*;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen::prelude::*;
use web_sys::*;
use gloo_console::log;
use gloo_net::http::Request;
use serde::Deserialize;
use serde_json::Value;
use base64::{Engine as _, engine::general_purpose};
use crate::Route;

#[derive(Debug, Deserialize)]
struct ChallengeResponse {
    #[serde(rename = "publicKey")]
    public_key: PublicKeyOptions,
}

#[derive(Debug, Deserialize)]
struct PublicKeyOptions {
    challenge: String,
    rp: RelyingParty,
    user: UserInfo,
    #[serde(rename = "pubKeyCredParams")]
    pub_key_cred_params: Vec<Value>,
    #[serde(rename = "authenticatorSelection")]
    authenticator_selection: Option<Value>,
    timeout: Option<u32>,
    attestation: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RelyingParty {
    name: String,
    id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UserInfo {
    id: String,
    name: String,
    #[serde(rename = "displayName")]
    display_name: String,
}

#[function_component(PasskeySetup)]
pub fn passkey_setup() -> Html {
    let navigator = use_navigator().unwrap();
    let error = use_state(|| None::<String>);
    let loading = use_state(|| false);

    let setup_passkey = {
        let error = error.clone();
        let loading = loading.clone();
        let navigator = navigator.clone();

        Callback::from(move |_| {
            let error = error.clone();
            let loading = loading.clone();
            let navigator = navigator.clone();
            loading.set(true);

            spawn_local(async move {
                match setup_passkey_flow().await {
                    Ok(_) => {
                        navigator.push(&Route::Home);
                    },
                    Err(e) => {
                        log!(&format!("Passkey setup error: {}", e));
                        error.set(Some(e));
                        loading.set(false);
                    }
                }
            });
        })
    };

    let skip_passkey = {
        let navigator = navigator.clone();

        Callback::from(move |_| {
            navigator.push(&Route::Home);
        })
    };

    html! {
        <div class="container">
            <div class="form-box">
                <h1>{"🔐 Setup Passkey"}</h1>
                <p class="subtitle">{"Secure your account with biometric authentication"}</p>
                
                <div class="info-box">
                    <h3>{"What is a Passkey?"}</h3>
                    <p>{"A passkey uses your device's biometric authentication (fingerprint, face recognition) or PIN to securely sign in without passwords."}</p>
                </div>
                
                <ul class="benefits">
                    <li>{"Faster and more secure than passwords"}</li>
                    <li>{"No need to remember complex passwords"}</li>
                    <li>{"Protection against phishing attacks"}</li>
                    <li>{"Works across all your devices"}</li>
                </ul>
                
                {if let Some(err) = (*error).clone() {
                    html! { <div class="error">{err}</div> }
                } else {
                    html! {}
                }}
                
                <div class="button-group">
                    <button 
                        class="btn-primary" 
                        onclick={setup_passkey}
                        disabled={*loading}
                    >
                        {if *loading { "Setting up..." } else { "Setup Passkey" }}
                    </button>
                    <button 
                        class="btn-secondary" 
                        onclick={skip_passkey}
                        disabled={*loading}
                    >
                        {"Skip for Now"}
                    </button>
                </div>
            </div>
        </div>
    }
}

async fn setup_passkey_flow() -> Result<(), String> {
    // Get challenge from server
    let response = Request::post("http://localhost:8000/api/passkey/register/start")
        .send()
        .await
        .map_err(|e| format!("Failed to get challenge: {}", e))?;
    
    if !response.ok() {
        return Err("Failed to start passkey registration".to_string());
    }
    
    let challenge_data: ChallengeResponse = response.json()
        .await
        .map_err(|e| format!("Failed to parse challenge: {}", e))?;
    
    // Create the credential
    let credential = create_credential(&challenge_data.public_key).await
        .map_err(|e| format!("Failed to create credential: {:?}", e))?;
    
    // Send credential to server
    let register_response = Request::post("http://localhost:8000/api/passkey/register/finish")
        .json(&credential)
        .map_err(|e| format!("Failed to prepare request: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Failed to register passkey: {}", e))?;
    
    if register_response.ok() {
        Ok(())
    } else {
        Err("Failed to register passkey with server".to_string())
    }
}

async fn create_credential(options: &PublicKeyOptions) -> Result<Value, JsValue> {
    let window = web_sys::window().ok_or("No window")?;
    let credentials = window.navigator().credentials();
    
    // Build the options object for create()
    let create_options = js_sys::Object::new();
    let public_key = js_sys::Object::new();
    
    // Challenge
    let challenge_bytes = general_purpose::STANDARD
        .decode(&options.challenge)
        .map_err(|_| JsValue::from_str("Failed to decode challenge"))?;
    let challenge_array = js_sys::Uint8Array::from(&challenge_bytes[..]);
    js_sys::Reflect::set(&public_key, &"challenge".into(), &challenge_array)?;
    
    // RP
    let rp = js_sys::Object::new();
    js_sys::Reflect::set(&rp, &"name".into(), &options.rp.name.clone().into())?;
    if let Some(id) = &options.rp.id {
        js_sys::Reflect::set(&rp, &"id".into(), &id.clone().into())?;
    }
    js_sys::Reflect::set(&public_key, &"rp".into(), &rp)?;
    
    // User
    let user = js_sys::Object::new();
    let user_id_bytes = general_purpose::STANDARD
        .decode(&options.user.id)
        .map_err(|_| JsValue::from_str("Failed to decode user ID"))?;
    let user_id_array = js_sys::Uint8Array::from(&user_id_bytes[..]);
    js_sys::Reflect::set(&user, &"id".into(), &user_id_array)?;
    js_sys::Reflect::set(&user, &"name".into(), &options.user.name.clone().into())?;
    js_sys::Reflect::set(&user, &"displayName".into(), &options.user.display_name.clone().into())?;
    js_sys::Reflect::set(&public_key, &"user".into(), &user)?;
    
    // pubKeyCredParams
    let params = js_sys::Array::new();
    for param in &options.pub_key_cred_params {
        let param_obj = serde_wasm_bindgen::to_value(param)?;
        params.push(&param_obj);
    }
    js_sys::Reflect::set(&public_key, &"pubKeyCredParams".into(), &params)?;
    
    // Optional fields
    if let Some(timeout) = options.timeout {
        js_sys::Reflect::set(&public_key, &"timeout".into(), &timeout.into())?;
    }
    
    if let Some(ref attestation) = options.attestation {
        js_sys::Reflect::set(&public_key, &"attestation".into(), &attestation.clone().into())?;
    }
    
    if let Some(ref auth_sel) = options.authenticator_selection {
        let auth_sel_val = serde_wasm_bindgen::to_value(auth_sel)?;
        js_sys::Reflect::set(&public_key, &"authenticatorSelection".into(), &auth_sel_val)?;
    }
    
    js_sys::Reflect::set(&create_options, &"publicKey".into(), &public_key)?;
    
    // Create the credential
    let promise = credentials.create_with_options(
        &create_options.unchecked_into()
    )?;
    
    let result = wasm_bindgen_futures::JsFuture::from(promise).await?;
    
    // Convert to JSON-serializable format
    let credential: PublicKeyCredential = result.unchecked_into();
    let response: AuthenticatorAttestationResponse = credential.response().unchecked_into();
    
    let credential_obj = js_sys::Object::new();
    js_sys::Reflect::set(&credential_obj, &"id".into(), &credential.id().into())?;
    js_sys::Reflect::set(&credential_obj, &"type".into(), &credential.type_().into())?;
    
    let raw_id_array = js_sys::Uint8Array::new(&credential.raw_id());
    let raw_id_vec: Vec<u8> = raw_id_array.to_vec();
    let raw_id_b64 = general_purpose::STANDARD.encode(&raw_id_vec);
    js_sys::Reflect::set(&credential_obj, &"rawId".into(), &raw_id_b64.into())?;
    
    let response_obj = js_sys::Object::new();
    
    let client_data_array = js_sys::Uint8Array::new(&response.client_data_json());
    let client_data_vec: Vec<u8> = client_data_array.to_vec();
    let client_data_b64 = general_purpose::STANDARD.encode(&client_data_vec);
    js_sys::Reflect::set(&response_obj, &"clientDataJSON".into(), &client_data_b64.into())?;
    
    let attestation_array = js_sys::Uint8Array::new(&response.attestation_object());
    let attestation_vec: Vec<u8> = attestation_array.to_vec();
    let attestation_b64 = general_purpose::STANDARD.encode(&attestation_vec);
    js_sys::Reflect::set(&response_obj, &"attestationObject".into(), &attestation_b64.into())?;
    
    js_sys::Reflect::set(&credential_obj, &"response".into(), &response_obj)?;
    
    Ok(serde_wasm_bindgen::from_value(credential_obj.into())?)
}
