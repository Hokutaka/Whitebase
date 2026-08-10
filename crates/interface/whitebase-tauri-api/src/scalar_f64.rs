use crate::CommandError;
use whitebase_interface::scalar_f64::{
    ScalarF64ObservationDto, ScalarF64Request, execute_scalar_f64_observation,
};

/// 対応している各バックエンドで`f64`スカラー加算を実行します。
#[tauri::command]
pub async fn observe_add_scalar_f64(
    request: ScalarF64Request,
) -> Result<ScalarF64ObservationDto, CommandError> {
    tauri::async_runtime::spawn_blocking(move || {
        execute_scalar_f64_observation(request).map_err(Into::into)
    })
    .await
    .map_err(|error| {
        CommandError::internal(
            "scalar_f64_observation_task_failed",
            format!("scalar f64 observation task failed: {error}"),
        )
    })?
}
