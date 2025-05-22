use crate::Result;

#[tauri::command]
pub async fn fetch_api_config(url: &str) -> Result<String, &str> {
    dbg!(url);
    Ok(reqwest::get(url).await.unwrap().text().await.unwrap())
}
