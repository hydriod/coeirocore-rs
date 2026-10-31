use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    pub speaker_name: String,
    pub speaker_uuid: String,
    pub styles: Vec<Style>,
    #[serde(default)]
    pub version: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Style {
    pub style_name: String,
    pub style_id: i32,
}
