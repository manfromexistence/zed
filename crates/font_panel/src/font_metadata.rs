use db::kvp::KeyValueStore;
use gpui::{App, AppContext as _, Context, SharedString};
use serde::{Deserialize, Serialize};

pub(crate) const DX_SYSTEM_FONT_METADATA_KEY: &str = "dx_font_panel_system_fonts";
pub(crate) const DX_SYSTEM_FONT_METADATA_SCHEMA_VERSION: u32 = 1;
pub(crate) const MAX_SYSTEM_FONT_METADATA_ENTRIES: usize = 4096;
pub(crate) const MAX_SYSTEM_FONT_METADATA_NAME_CHARS: usize = 160;
pub(crate) const MAX_SYSTEM_FONT_METADATA_JSON_BYTES: usize = 512 * 1024;

#[derive(Serialize, Deserialize)]
struct SerializedSystemFontMetadataArtifact {
    schema_version: u32,
    fonts: Vec<String>,
}

pub(crate) fn load_system_font_metadata(cx: &App) -> Option<Vec<SharedString>> {
    let json = KeyValueStore::global(cx)
        .read_kvp(DX_SYSTEM_FONT_METADATA_KEY)
        .ok()
        .flatten()?;
    parse_system_font_metadata(&json)
}

fn parse_system_font_metadata(json: &str) -> Option<Vec<SharedString>> {
    if json.len() > MAX_SYSTEM_FONT_METADATA_JSON_BYTES {
        return None;
    }

    let artifact = serde_json::from_str::<SerializedSystemFontMetadataArtifact>(json).ok()?;
    if artifact.schema_version != DX_SYSTEM_FONT_METADATA_SCHEMA_VERSION {
        return None;
    }

    let mut fonts = Vec::with_capacity(artifact.fonts.len().min(MAX_SYSTEM_FONT_METADATA_ENTRIES));
    fonts.extend(
        artifact
            .fonts
            .into_iter()
            .take(MAX_SYSTEM_FONT_METADATA_ENTRIES)
            .filter_map(|font| bounded_system_font_name(&font).map(SharedString::from)),
    );
    (!fonts.is_empty()).then_some(fonts)
}

pub(crate) fn persist_system_font_metadata<T: 'static>(
    fonts: &[SharedString],
    cx: &mut Context<T>,
) {
    let mut serialized_fonts =
        Vec::with_capacity(fonts.len().min(MAX_SYSTEM_FONT_METADATA_ENTRIES));
    serialized_fonts.extend(
        fonts
            .iter()
            .take(MAX_SYSTEM_FONT_METADATA_ENTRIES)
            .filter_map(|font| bounded_system_font_name(font.as_ref())),
    );
    if serialized_fonts.is_empty() {
        return;
    }

    let Ok(json) = serde_json::to_string(&SerializedSystemFontMetadataArtifact {
        schema_version: DX_SYSTEM_FONT_METADATA_SCHEMA_VERSION,
        fonts: serialized_fonts,
    }) else {
        return;
    };
    if json.len() > MAX_SYSTEM_FONT_METADATA_JSON_BYTES {
        return;
    }

    let kvp = KeyValueStore::global(cx);
    cx.background_spawn(async move {
        let _ = kvp
            .write_kvp(DX_SYSTEM_FONT_METADATA_KEY.to_string(), json)
            .await;
    })
    .detach();
}

fn bounded_system_font_name(font_name: &str) -> Option<String> {
    let font_name = font_name.trim();
    if font_name.is_empty() {
        return None;
    }

    let mut name = String::with_capacity(font_name.len().min(MAX_SYSTEM_FONT_METADATA_NAME_CHARS));
    for character in font_name.chars().take(MAX_SYSTEM_FONT_METADATA_NAME_CHARS) {
        if !character.is_control() {
            name.push(character);
        }
    }

    (!name.is_empty()).then_some(name)
}
