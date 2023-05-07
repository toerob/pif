
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Song {
    pub id: i64,
    #[serde(rename = "type")]
    pub type_field: String,
    pub title: String,
    pub artist: Artist,
    pub chords_present: bool,
    pub tab_types: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Artist {
    pub id: i64,
    #[serde(rename = "type")]
    pub type_field: String,
    pub name_without_the_prefix: String,
    pub use_the_prefix: bool,
    pub name: String,
}