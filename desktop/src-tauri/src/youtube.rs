use actix_web::{HttpResponse, web};

#[derive(serde::Deserialize)]
pub struct YoutubeEmbedQuery {
    v: String,
}

/// YouTube's embed player rejects the webview's `tauri://localhost` origin
/// (no valid HTTP referer), so a direct `<iframe src="youtube.com/embed/...">`
/// fails with "Error 153" on the WebKit-based webviews (macOS, Linux).  
///
/// See https://github.com/tauri-apps/tauri/issues/14422
///
/// Deliberately left off the api-key-gated scope in `backend.rs`.
pub async fn youtube_embed(query: web::Query<YoutubeEmbedQuery>) -> HttpResponse {
    let video_id = &query.v;

    if !validate_video_id(video_id) {
        return HttpResponse::BadRequest().body("invalid video id");
    }

    HttpResponse::Ok().content_type("text/html; charset=utf-8").body(format!(
        r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<style>
  * {{ margin: 0; padding: 0; box-sizing: border-box; }}
  html, body {{ width: 100%; height: 100%; overflow: hidden; background: #000; }}
  iframe {{ width: 100%; height: 100%; border: none; }}
</style>
</head>
<body>
<iframe
  src="https://www.youtube.com/embed/{video_id}"
  allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share"
  allowfullscreen>
</iframe>
</body>
</html>"#
    ))
}

fn validate_video_id(video_id: &str) -> bool {
    video_id.len() == 11
        && video_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}
