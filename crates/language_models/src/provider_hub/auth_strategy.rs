use convert_case::{Case, Casing};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ApiKeyHeaderStyle {
    #[default]
    Bearer,
    XApiKey,
    AuthorizationApiKey,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthCredential {
    ApiKey {
        header_style: ApiKeyHeaderStyle,
        value: String,
    },
    Basic {
        username: String,
        password: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AuthStrategy {
    #[default]
    ApiKey {
        header_style: ApiKeyHeaderStyle,
        env_var: Option<String>,
    },
    BasicAuth {
        username_env_var: Option<String>,
        password_env_var: Option<String>,
    },
    OAuth2AuthorizationCodePkce {
        authorization_url: String,
        token_url: String,
        client_id: Option<String>,
        scopes: Vec<String>,
    },
    OAuth2DeviceFlow {
        device_authorization_url: String,
        token_url: String,
        client_id: Option<String>,
        scopes: Vec<String>,
    },
    OAuth2ClientCredentials {
        token_url: String,
        client_id_env_var: Option<String>,
        client_secret_env_var: Option<String>,
        scopes: Vec<String>,
    },
    OAuth2Password {
        token_url: String,
        client_id_env_var: Option<String>,
        client_secret_env_var: Option<String>,
        username_env_var: Option<String>,
        password_env_var: Option<String>,
        scopes: Vec<String>,
    },
    GithubOAuth,
    AwsSigV4NamedProfile {
        service: String,
        region: String,
        profile_env_var: Option<String>,
    },
    AwsSigV4EnvironmentVars {
        service: String,
        region: String,
    },
    AwsSigV4InstanceProfile {
        service: String,
        region: String,
    },
    AwsSigV4EcsTaskRole {
        service: String,
        region: String,
    },
    AwsSsoIamIdentityCenter {
        service: String,
        region: String,
        start_url: String,
    },
    GcpServiceAccountJson {
        scopes: Vec<String>,
        credentials_env_var: Option<String>,
    },
    GcpApplicationDefaultCredentials {
        scopes: Vec<String>,
    },
    GcpWorkloadIdentityFederation {
        scopes: Vec<String>,
        audience: String,
        subject_token_type: String,
        token_url: String,
    },
    GeminiAdvancedSubscription,
    AzureClientSecretCredential {
        tenant_id_env_var: Option<String>,
        client_id_env_var: Option<String>,
        client_secret_env_var: Option<String>,
        scope: String,
    },
    AzureManagedIdentity {
        scope: String,
    },
    AzureCliCredential {
        scope: String,
    },
    AzureAIFoundry {
        scope: String,
    },
    ReqsignMultiCloud {
        service: String,
        region: Option<String>,
    },
    HmacSha256SignedUrl,
    VolcanoEngineToken,
    BaiduOAuth2,
    GitLabDuoToken,
    AmazonQToken,
}

impl AuthStrategy {
    pub fn is_api_key_like(&self) -> bool {
        matches!(self, Self::ApiKey { .. })
    }

    pub fn default_env_var_for_provider(provider_id: &str) -> String {
        format!("{}_API_KEY", provider_id.to_case(Case::UpperSnake))
    }

    pub fn env_var_name_for_provider(&self, provider_id: &str) -> Option<String> {
        match self {
            Self::ApiKey { env_var, .. } => env_var
                .clone()
                .or_else(|| Some(Self::default_env_var_for_provider(provider_id))),
            _ => None,
        }
    }
}
