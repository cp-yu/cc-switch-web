#[cfg(feature = "desktop")]
pub type UiAppHandle = tauri::AppHandle;

#[cfg(not(feature = "desktop"))]
#[derive(Clone, Debug, Default)]
pub struct UiAppHandle;

#[cfg(feature = "desktop")]
pub fn spawn<F>(future: F)
where
    F: std::future::Future<Output = ()> + Send + 'static,
{
    tauri::async_runtime::spawn(future);
}

#[cfg(not(feature = "desktop"))]
pub fn spawn<F>(future: F)
where
    F: std::future::Future<Output = ()> + Send + 'static,
{
    tokio::spawn(future);
}
