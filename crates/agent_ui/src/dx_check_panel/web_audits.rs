use serde_json::Value;

use super::{
    DxCheckPanelWebAudit,
    parser::{
        bool_from, bounded_panel_text, bounded_string_from, first_non_empty, u16_from, u32_from,
        u64_from,
    },
};

pub(super) fn web_audit_rows(receipt: &Value) -> Vec<DxCheckPanelWebAudit> {
    let mut rows = receipt
        .get("engine")
        .and_then(|engine| engine.get("web_audit_results"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .take(6)
        .filter_map(|result| {
            let url = bounded_string_from(result.get("url"))?;
            let label = bounded_string_from(result.get("target_id"))
                .or_else(|| bounded_string_from(result.get("id")))
                .unwrap_or_else(|| url.clone());
            Some(DxCheckPanelWebAudit {
                label,
                status: web_audit_status(result),
                detail: web_audit_detail(result),
                url,
                source: bounded_string_from(result.get("source")),
            })
        })
        .collect::<Vec<_>>();

    if let Some(row) = receipt
        .get("web")
        .and_then(|web| web.get("lighthouse"))
        .and_then(lighthouse_row)
    {
        rows.push(row);
    }

    rows.extend(
        receipt
            .get("web")
            .and_then(|web| web.get("lighthouse_runs"))
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(lighthouse_row),
    );

    rows.truncate(6);
    rows
}

fn lighthouse_row(result: &Value) -> Option<DxCheckPanelWebAudit> {
    let url = first_non_empty([
        bounded_string_from(result.get("requested_url")),
        bounded_string_from(result.get("url")),
        bounded_string_from(result.get("final_url")),
    ])?;
    let target = bounded_string_from(result.get("target_id"))
        .or_else(|| bounded_string_from(result.get("id")))
        .unwrap_or_else(|| "web".to_string());
    let label = if target.to_ascii_lowercase().contains("lighthouse") {
        target
    } else {
        format!("{target} Lighthouse")
    };

    Some(DxCheckPanelWebAudit {
        label,
        status: lighthouse_status(result),
        detail: lighthouse_detail(result),
        url,
        source: bounded_string_from(result.get("source_path"))
            .or_else(|| bounded_string_from(result.get("source"))),
    })
}

fn lighthouse_status(result: &Value) -> String {
    let status = bounded_string_from(result.get("status"));
    if matches!(status.as_deref(), Some("ready" | "warning" | "blocked")) {
        return status.unwrap();
    }

    let score = u32_from(result.get("total_score")).or_else(|| u32_from(result.get("score")));
    let max_score = u32_from(result.get("max_score")).unwrap_or(400);
    match score {
        Some(score) if score == max_score => "ready".to_string(),
        Some(score) if score >= max_score.saturating_mul(4) / 5 => "warning".to_string(),
        Some(_) => "blocked".to_string(),
        None => "unknown".to_string(),
    }
}

fn lighthouse_detail(result: &Value) -> String {
    let score = u32_from(result.get("total_score")).or_else(|| u32_from(result.get("score")));
    let max_score = u32_from(result.get("max_score")).unwrap_or(400);
    let score_label = score
        .map(|score| format!("{score}/{max_score}"))
        .unwrap_or_else(|| "score unknown".to_string());
    let categories = lighthouse_category_labels(result);

    if categories.is_empty() {
        return bounded_panel_text(&score_label).unwrap_or(score_label);
    }

    bounded_panel_text(&format!("{score_label} / {}", categories.join(" / ")))
        .unwrap_or(score_label)
}

fn lighthouse_category_labels(result: &Value) -> Vec<String> {
    let direct = [
        ("performance", "Performance"),
        ("accessibility", "Accessibility"),
        ("best_practices", "Best Practices"),
        ("seo", "SEO"),
    ]
    .into_iter()
    .filter_map(|(field, label)| {
        u32_from(result.get(field)).map(|score| format!("{label} {score}"))
    })
    .collect::<Vec<_>>();
    if !direct.is_empty() {
        return direct;
    }

    result
        .get("categories")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .take(4)
        .filter_map(|category| {
            let label = bounded_string_from(category.get("label"))
                .or_else(|| bounded_string_from(category.get("id")))?;
            let score = u32_from(category.get("score"))?;
            Some(format!("{label} {score}"))
        })
        .collect()
}

fn web_audit_status(result: &Value) -> String {
    let http_status = u16_from(result.get("status"));
    let metadata_missing = !bool_from(result.get("title_present")).unwrap_or(false)
        || !bool_from(result.get("description_present")).unwrap_or(false)
        || !bool_from(result.get("viewport_present")).unwrap_or(false);
    let header_count = u16_from(result.get("security_header_count")).unwrap_or(0);

    match http_status {
        Some(200..=399) if !metadata_missing && header_count >= 2 => "ready".to_string(),
        Some(200..=399) => "warning".to_string(),
        Some(_) => "blocked".to_string(),
        None => "unknown".to_string(),
    }
}

fn web_audit_detail(result: &Value) -> String {
    let status = u16_from(result.get("status"))
        .map(|status| format!("HTTP {status}"))
        .unwrap_or_else(|| "HTTP unknown".to_string());
    let bytes = u64_from(result.get("html_bytes"))
        .map(format_bytes)
        .unwrap_or_else(|| "size unknown".to_string());
    let headers = u16_from(result.get("security_header_count"))
        .map(|count| format!("{count} security headers"))
        .unwrap_or_else(|| "headers unknown".to_string());
    let title = present_label("title", bool_from(result.get("title_present")));
    let description = present_label("description", bool_from(result.get("description_present")));
    let viewport = present_label("viewport", bool_from(result.get("viewport_present")));

    bounded_panel_text(&format!(
        "{status} / {bytes} / {headers} / {title}, {description}, {viewport}"
    ))
    .unwrap_or_else(|| status)
}

fn present_label(name: &str, value: Option<bool>) -> String {
    match value {
        Some(true) => format!("{name} yes"),
        Some(false) => format!("{name} no"),
        None => format!("{name} unknown"),
    }
}

fn format_bytes(value: u64) -> String {
    if value >= 1_000_000 {
        format!("{:.1} MB", value as f64 / 1_000_000.0)
    } else if value >= 1_000 {
        format!("{:.1} KB", value as f64 / 1_000.0)
    } else {
        format!("{value} B")
    }
}
