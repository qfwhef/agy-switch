use serde_json::Value;

const LOAD_PROJECT_ENDPOINTS: [&str; 2] = [
    "https://daily-cloudcode-pa.googleapis.com/v1internal:loadCodeAssist",
    "https://cloudcode-pa.googleapis.com/v1internal:loadCodeAssist",
];

/// Resolve the Antigravity project ID used by quota requests.
pub async fn fetch_project_id(access_token: &str) -> Result<String, String> {
    let request_body = serde_json::json!({
        "metadata": {
            "ideType": "ANTIGRAVITY"
        }
    });

    let client = crate::utils::http::get_standard_client()?;
    let mut last_error = String::new();

    for url in LOAD_PROJECT_ENDPOINTS {
        let response = client
            .post(url)
            .bearer_auth(access_token)
            .header(
                rquest::header::USER_AGENT,
                crate::constants::NATIVE_OAUTH_USER_AGENT.as_str(),
            )
            .header(rquest::header::CONTENT_TYPE, "application/json")
            .json(&request_body)
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                let data: Value = resp
                    .json()
                    .await
                    .map_err(|e| format!("解析响应失败: {}", e))?;
                return data
                    .get("cloudaicompanionProject")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned)
                    .ok_or_else(|| "账号无资格获取官方 cloudaicompanionProject".to_string());
            }
            Ok(resp) => {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                last_error = format!("loadCodeAssist [{}] 返回错误 {}: {}", url, status, body);
                if matches!(
                    status,
                    rquest::StatusCode::UNAUTHORIZED
                        | rquest::StatusCode::FORBIDDEN
                        | rquest::StatusCode::TOO_MANY_REQUESTS
                ) {
                    return Err(last_error);
                }
            }
            Err(error) => {
                last_error = format!("loadCodeAssist [{}] 请求失败: {}", url, error);
            }
        }
    }

    Err(if last_error.is_empty() {
        "loadCodeAssist 所有端点均不可用".to_string()
    } else {
        last_error
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_project_endpoints_prefer_daily_and_exclude_sandbox() {
        assert_eq!(
            LOAD_PROJECT_ENDPOINTS[0],
            "https://daily-cloudcode-pa.googleapis.com/v1internal:loadCodeAssist"
        );
        assert!(LOAD_PROJECT_ENDPOINTS
            .iter()
            .all(|url| !url.contains(".sandbox.googleapis.com")));
    }
}
