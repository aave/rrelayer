use crate::wallet::WalletError;
use crate::yaml::GcpSecretManagerProviderConfig;
use google_cloud_auth::credentials::service_account;
use google_cloud_secretmanager_v1::client::SecretManagerService;
use std::path::Path;

pub async fn get_gcp_secret(
    project_path: &Path,
    config: &GcpSecretManagerProviderConfig,
) -> Result<String, WalletError> {
    let key_path = project_path.join(&config.service_account_key_path);

    let service_account_key = tokio::fs::read_to_string(&key_path).await.map_err(|e| {
        WalletError::ConfigurationError {
            message: format!("Failed to read service account key: {}", e),
        }
    })?;
    let service_account_key: serde_json::Value = serde_json::from_str(&service_account_key)
        .map_err(|e| WalletError::ConfigurationError {
            message: format!("Failed to read service account key: {}", e),
        })?;

    let project_id =
        service_account_key.get("project_id").and_then(|v| v.as_str()).map(str::to_string).ok_or(
            WalletError::ConfigurationError {
                message: "No project_id found in service account key".to_string(),
            },
        )?;

    let credentials = service_account::Builder::new(service_account_key).build().map_err(|e| {
        WalletError::AuthenticationError {
            message: format!("Failed to create authenticator: {}", e),
        }
    })?;

    let client =
        SecretManagerService::builder().with_credentials(credentials).build().await.map_err(
            |e| WalletError::ConfigurationError {
                message: format!("Failed to create Secret Manager client: {}", e),
            },
        )?;

    let version = config.version.as_deref().unwrap_or("latest");
    let secret_path = format!("projects/{}/secrets/{}/versions/{}", project_id, config.id, version);

    let result =
        client.access_secret_version().set_name(secret_path).send().await.map_err(|e| {
            WalletError::ApiError { message: format!("Failed to access secret: {}", e) }
        })?;

    let secret_data = result
        .payload
        .map(|payload| payload.data)
        .filter(|data| !data.is_empty())
        .ok_or(WalletError::ApiError { message: "No secret data found".to_string() })?;

    let secret_string = String::from_utf8(secret_data.to_vec())?;

    let secret_key = config.key.clone();
    let secret_json: serde_json::Value = serde_json::from_str(&secret_string)?;

    let key_value = secret_json.get(&secret_key).and_then(|v| v.as_str()).ok_or(
        WalletError::ConfigurationError {
            message: format!("Key '{}' not found in secret or is not a string", secret_key),
        },
    )?;

    Ok(key_value.to_string())
}
