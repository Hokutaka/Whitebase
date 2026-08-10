use whitebase_interface::scalar_f64::{
    execute_scalar_f64_observation, ScalarF64ObservationDto, ScalarF64Request,
};

/// 対応している各バックエンドで`f64`スカラー加算を実行します。
#[tauri::command]
pub async fn observe_add_scalar_f64(
    request: ScalarF64Request,
) -> Result<ScalarF64ObservationDto, String> {
    tauri::async_runtime::spawn_blocking(move || {
        execute_scalar_f64_observation(request).map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| format!("scalar f64 observation task failed: {error}"))?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observation_generates_decimal_reference_from_inputs() {
        let report = execute_scalar_f64_observation(ScalarF64Request {
            lhs: "0.1".to_owned(),
            rhs: "0.2".to_owned(),
        })
        .expect("observation must succeed");

        assert_eq!(report.decimal_reference, "0.3");
        assert_eq!(report.reference.decimal, "0.29999999999999999");
        assert_eq!(report.reference.bits, "0x3fd3333333333333");
    }

    #[test]
    fn observation_rejects_non_finite_values() {
        let error = execute_scalar_f64_observation(ScalarF64Request {
            lhs: "NaN".to_owned(),
            rhs: "0.2".to_owned(),
        })
        .expect_err("NaN must be rejected");

        assert!(error.to_string().contains("invalid lhs value"));
    }
}
