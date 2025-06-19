use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    #[serde(rename = "AppBskyActorProfile")]
    pub app_bsky_actor_profile: AppBskyActorProfile,
    #[serde(rename = "AppBskyFeedGenerator")]
    pub app_bsky_feed_generator: AppBskyFeedGenerator,
    #[serde(rename = "AppBskyGraphList")]
    pub app_bsky_graph_list: AppBskyGraphList,
    #[serde(rename = "AppBskyEmbedImages")]
    pub app_bsky_embed_images: AppBskyEmbedImages,
    #[serde(rename = "AppBskyEmbedExternal")]
    pub app_bsky_embed_external: AppBskyEmbedExternal,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppBskyActorProfile {
    pub lexicon: i64,
    pub id: String,
    pub defs: Defs83,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Defs83 {
    pub main: Main78,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Main78 {
    #[serde(rename = "type")]
    pub type_field: String,
    pub description: String,
    pub key: String,
    pub record: Record4,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Record4 {
    #[serde(rename = "type")]
    pub type_field: String,
    pub properties: Properties150,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Properties150 {
    pub display_name: DisplayName4,
    pub description: Description4,
    pub avatar: Avatar4,
    pub banner: Banner2,
    pub labels: Labels8,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplayName4 {
    #[serde(rename = "type")]
    pub type_field: String,
    pub max_graphemes: i64,
    pub max_length: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Description4 {
    #[serde(rename = "type")]
    pub type_field: String,
    pub description: String,
    pub max_graphemes: i64,
    pub max_length: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Avatar4 {
    #[serde(rename = "type")]
    pub type_field: String,
    pub description: String,
    pub accept: Vec<String>,
    pub max_size: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Banner2 {
    #[serde(rename = "type")]
    pub type_field: String,
    pub description: String,
    pub accept: Vec<String>,
    pub max_size: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Labels8 {
    #[serde(rename = "type")]
    pub type_field: String,
    pub description: String,
    pub refs: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppBskyFeedGenerator {
    pub lexicon: i64,
    pub id: String,
    pub defs: Defs93,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Defs93 {
    pub main: Main87,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Main87 {
    #[serde(rename = "type")]
    pub type_field: String,
    pub description: String,
    pub key: String,
    pub record: Record11,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Record11 {
    #[serde(rename = "type")]
    pub type_field: String,
    pub required: Vec<String>,
    pub properties: Properties190,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Properties190 {
    pub did: Did39,
    pub display_name: DisplayName6,
    pub description: Description8,
    pub description_facets: DescriptionFacets2,
    pub avatar: Avatar6,
    pub accepts_interactions: AcceptsInteractions2,
    pub labels: Labels12,
    pub created_at: CreatedAt5,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Did39 {
    #[serde(rename = "type")]
    pub type_field: String,
    pub format: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplayName6 {
    #[serde(rename = "type")]
    pub type_field: String,
    pub max_graphemes: i64,
    pub max_length: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Description8 {
    #[serde(rename = "type")]
    pub type_field: String,
    pub max_graphemes: i64,
    pub max_length: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DescriptionFacets2 {
    #[serde(rename = "type")]
    pub type_field: String,
    pub items: Items63,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Items63 {
    #[serde(rename = "type")]
    pub type_field: String,
    #[serde(rename = "ref")]
    pub ref_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Avatar6 {
    #[serde(rename = "type")]
    pub type_field: String,
    pub accept: Vec<String>,
    pub max_size: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceptsInteractions2 {
    #[serde(rename = "type")]
    pub type_field: String,
    pub description: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Labels12 {
    #[serde(rename = "type")]
    pub type_field: String,
    pub description: String,
    pub refs: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatedAt5 {
    #[serde(rename = "type")]
    pub type_field: String,
    pub format: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppBskyGraphList {
    pub lexicon: i64,
    pub id: String,
    pub defs: Defs127,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Defs127 {
    pub main: Main120,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Main120 {
    #[serde(rename = "type")]
    pub type_field: String,
    pub description: String,
    pub key: String,
    pub record: Record18,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Record18 {
    #[serde(rename = "type")]
    pub type_field: String,
    pub required: Vec<String>,
    pub properties: Properties262,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Properties262 {
    pub purpose: Purpose3,
    pub name: Name10,
    pub description: Description10,
    pub description_facets: DescriptionFacets4,
    pub avatar: Avatar9,
    pub labels: Labels16,
    pub created_at: CreatedAt13,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Purpose3 {
    #[serde(rename = "type")]
    pub type_field: String,
    pub description: String,
    #[serde(rename = "ref")]
    pub ref_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Name10 {
    #[serde(rename = "type")]
    pub type_field: String,
    pub max_length: i64,
    pub min_length: i64,
    pub description: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Description10 {
    #[serde(rename = "type")]
    pub type_field: String,
    pub max_graphemes: i64,
    pub max_length: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DescriptionFacets4 {
    #[serde(rename = "type")]
    pub type_field: String,
    pub items: Items101,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Items101 {
    #[serde(rename = "type")]
    pub type_field: String,
    #[serde(rename = "ref")]
    pub ref_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Avatar9 {
    #[serde(rename = "type")]
    pub type_field: String,
    pub accept: Vec<String>,
    pub max_size: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Labels16 {
    #[serde(rename = "type")]
    pub type_field: String,
    pub refs: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatedAt13 {
    #[serde(rename = "type")]
    pub type_field: String,
    pub format: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppBskyEmbedImages {
    pub lexicon: i64,
    pub id: String,
    pub description: String,
    pub defs: Defs88,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Defs88 {
    pub main: Main83,
    pub image: Image,
    pub aspect_ratio: AspectRatio2,
    pub view: View2,
    pub view_image: ViewImage,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Main83 {
    #[serde(rename = "type")]
    pub type_field: String,
    pub required: Vec<String>,
    pub properties: Properties160,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Properties160 {
    pub images: Images,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Images {
    #[serde(rename = "type")]
    pub type_field: String,
    pub items: Items53,
    pub max_length: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Items53 {
    #[serde(rename = "type")]
    pub type_field: String,
    #[serde(rename = "ref")]
    pub ref_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    #[serde(rename = "type")]
    pub type_field: String,
    pub required: Vec<String>,
    pub properties: Properties161,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Properties161 {
    pub image: Image2,
    pub alt: Alt,
    pub aspect_ratio: AspectRatio,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Image2 {
    #[serde(rename = "type")]
    pub type_field: String,
    pub accept: Vec<String>,
    pub max_size: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Alt {
    #[serde(rename = "type")]
    pub type_field: String,
    pub description: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AspectRatio {
    #[serde(rename = "type")]
    pub type_field: String,
    #[serde(rename = "ref")]
    pub ref_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AspectRatio2 {
    #[serde(rename = "type")]
    pub type_field: String,
    pub description: String,
    pub required: Vec<String>,
    pub properties: Properties162,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Properties162 {
    pub width: Width,
    pub height: Height,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Width {
    #[serde(rename = "type")]
    pub type_field: String,
    pub minimum: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Height {
    #[serde(rename = "type")]
    pub type_field: String,
    pub minimum: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct View2 {
    #[serde(rename = "type")]
    pub type_field: String,
    pub required: Vec<String>,
    pub properties: Properties163,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Properties163 {
    pub images: Images2,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Images2 {
    #[serde(rename = "type")]
    pub type_field: String,
    pub items: Items54,
    pub max_length: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Items54 {
    #[serde(rename = "type")]
    pub type_field: String,
    #[serde(rename = "ref")]
    pub ref_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewImage {
    #[serde(rename = "type")]
    pub type_field: String,
    pub required: Vec<String>,
    pub properties: Properties164,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Properties164 {
    pub thumb: Thumb3,
    pub fullsize: Fullsize,
    pub alt: Alt2,
    pub aspect_ratio: AspectRatio3,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thumb3 {
    #[serde(rename = "type")]
    pub type_field: String,
    pub format: String,
    pub description: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Fullsize {
    #[serde(rename = "type")]
    pub type_field: String,
    pub format: String,
    pub description: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Alt2 {
    #[serde(rename = "type")]
    pub type_field: String,
    pub description: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AspectRatio3 {
    #[serde(rename = "type")]
    pub type_field: String,
    #[serde(rename = "ref")]
    pub ref_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppBskyEmbedExternal {
    pub lexicon: i64,
    pub id: String,
    pub defs: Defs87,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Defs87 {
    pub main: Main82,
    pub external: External2,
    pub view: View,
    pub view_external: ViewExternal,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Main82 {
    #[serde(rename = "type")]
    pub type_field: String,
    pub description: String,
    pub required: Vec<String>,
    pub properties: Properties156,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Properties156 {
    pub external: External,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct External {
    #[serde(rename = "type")]
    pub type_field: String,
    #[serde(rename = "ref")]
    pub ref_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct External2 {
    #[serde(rename = "type")]
    pub type_field: String,
    pub required: Vec<String>,
    pub properties: Properties157,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Properties157 {
    pub uri: Uri8,
    pub title: Title,
    pub description: Description5,
    pub thumb: Thumb,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Uri8 {
    #[serde(rename = "type")]
    pub type_field: String,
    pub format: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Title {
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Description5 {
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thumb {
    #[serde(rename = "type")]
    pub type_field: String,
    pub accept: Vec<String>,
    pub max_size: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    #[serde(rename = "type")]
    pub type_field: String,
    pub required: Vec<String>,
    pub properties: Properties158,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Properties158 {
    pub external: External3,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct External3 {
    #[serde(rename = "type")]
    pub type_field: String,
    #[serde(rename = "ref")]
    pub ref_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewExternal {
    #[serde(rename = "type")]
    pub type_field: String,
    pub required: Vec<String>,
    pub properties: Properties159,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Properties159 {
    pub uri: Uri9,
    pub title: Title2,
    pub description: Description6,
    pub thumb: Thumb2,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Uri9 {
    #[serde(rename = "type")]
    pub type_field: String,
    pub format: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Title2 {
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Description6 {
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thumb2 {
    #[serde(rename = "type")]
    pub type_field: String,
    pub format: String,
}
