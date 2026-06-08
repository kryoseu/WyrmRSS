use actix_web::{HttpRequest, HttpResponse, Responder};

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
