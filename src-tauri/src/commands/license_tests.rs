use super::*;

#[test]
fn license_ipc_responses_are_discriminated_and_never_contain_paths() {
    let responses = [
        serde_json::to_value(LicenseStatusResponse::Status { code: "active" }).unwrap(),
        serde_json::to_value(InstallationCodeResponse::Error { code: "machine_identity_unavailable" }).unwrap(),
        serde_json::to_value(LicenseFileSelection::Cancelled).unwrap(),
        serde_json::to_value(LicenseImportResponse::Cancelled).unwrap(),
    ];
    assert_eq!(responses[0], serde_json::json!({"kind":"status","code":"active"}));
    assert_eq!(responses[1], serde_json::json!({"kind":"error","code":"machine_identity_unavailable"}));
    for response in responses { assert!(!response.to_string().contains("path")); }
}
