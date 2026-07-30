use serde::{Deserialize, Serialize};
use whitebase_runner::{F64Value, Runner, ScalarF64BackendObservation, ScalarF64ObservationReport};

/// フロントエンドから受け取る`f64`スカラー加算の観測設定です。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScalarF64Request {
    pub lhs: String,
    pub rhs: String,
}

/// `f64`値の表示用情報です。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct F64ValueDto {
    pub value: f64,
    pub decimal: String,
    pub bits: String,
}

/// 1バックエンド分の`f64`スカラー加算結果です。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScalarF64BackendResultDto {
    pub backend: String,
    pub result: F64ValueDto,
    pub matches_reference_bits: bool,
}

/// フロントエンドへ返す`f64`スカラー加算の観測レポートです。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScalarF64ObservationDto {
    pub lhs_input: String,
    pub rhs_input: String,
    pub lhs: F64ValueDto,
    pub rhs: F64ValueDto,
    pub decimal_reference: String,
    pub reference: F64ValueDto,
    pub results: Vec<ScalarF64BackendResultDto>,
    pub all_backends_match: bool,
}

/// 対応している各バックエンドで`f64`スカラー加算を実行します。
#[tauri::command]
pub async fn observe_add_scalar_f64(
    request: ScalarF64Request,
) -> Result<ScalarF64ObservationDto, String> {
    tauri::async_runtime::spawn_blocking(move || run_observation(request))
        .await
        .map_err(|error| format!("scalar f64 observation task failed: {error}"))?
}

fn run_observation(request: ScalarF64Request) -> Result<ScalarF64ObservationDto, String> {
    Runner::new()
        .observe_add_scalar_f64(&request.lhs, &request.rhs)
        .map(Into::into)
        .map_err(|error| error.to_string())
}

impl From<ScalarF64ObservationReport> for ScalarF64ObservationDto {
    fn from(report: ScalarF64ObservationReport) -> Self {
        Self {
            lhs_input: report.lhs_input,
            rhs_input: report.rhs_input,
            lhs: report.lhs.into(),
            rhs: report.rhs.into(),
            decimal_reference: report.decimal_reference,
            reference: report.reference.into(),
            results: report.results.into_iter().map(Into::into).collect(),
            all_backends_match: report.all_backends_match,
        }
    }
}

impl From<ScalarF64BackendObservation> for ScalarF64BackendResultDto {
    fn from(result: ScalarF64BackendObservation) -> Self {
        Self {
            backend: result.backend.display_name().to_owned(),
            result: result.result.into(),
            matches_reference_bits: result.matches_reference_bits,
        }
    }
}

impl From<F64Value> for F64ValueDto {
    fn from(value: F64Value) -> Self {
        Self {
            value: value.value,
            decimal: format!("{:.17}", value.value),
            bits: format!("0x{:016x}", value.bits),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observation_generates_decimal_reference_from_inputs() {
        let report = run_observation(ScalarF64Request {
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
        let error = run_observation(ScalarF64Request {
            lhs: "NaN".to_owned(),
            rhs: "0.2".to_owned(),
        })
        .expect_err("NaN must be rejected");

        assert!(error.contains("invalid lhs value"));
    }
}
