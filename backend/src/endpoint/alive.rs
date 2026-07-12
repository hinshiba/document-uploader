#[tracing::instrument(level="info")]
pub async fn alive() -> String {
    "ok".to_owned()
}
