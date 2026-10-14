use actix_web::{HttpRequest, HttpResponse, Responder};
use database::models::{post::Post, settings::Settings};
use serde::Serialize;

#[derive(Serialize, ts_rs::TS)]
#[ts(export)]
pub struct SettingsResponse {
    #[serde(flatten)]
    #[ts(flatten)]
    pub settings: Settings,
    pub run_as_demo: bool,
}

#[derive(Serialize, ts_rs::TS)]
#[ts(export)]
pub struct UpdatePostResponse {
    pub post: Post,
    pub feed_unread_count: i64,
}

pub struct XmlResponse {
    pub body: String,
}

impl Responder for XmlResponse {
    type Body = actix_web::body::BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        HttpResponse::Ok()
            .content_type("application/xml; charset=utf-8")
            .append_header(("Content-Disposition", "attachment"))
            .body(self.body)
    }
}
