use actix_web::{AsyncResponder, HttpMessage, HttpRequest, HttpResponse, Responder};
use crate::{
    actor::database::{BasicAuth, Register, TokenAuth},
    middleware::cond::HttpResponseBuilderExt,
    model::user::{Registration, User},
    state::PointercrateState,
};
use log::info;
use serde_json::json;
use tokio::prelude::future::Future;

pub fn register(req: &HttpRequest<PointercrateState>) -> impl Responder {
    info!("POST /api/v1/auth/register/");

    let state = req.state().clone();

    req.json()
        .from_err()
        .and_then(move |registration: Registration| state.database(Register(registration)))
        .map(|user: User| {
            HttpResponse::Created()
                .header("Location", "/auth/me/")
                .json_with_etag(user)
        }).responder()
}

pub fn login(req: &HttpRequest<PointercrateState>) -> impl Responder {
    info!("POST /api/v1/auth/");

    req.state()
        .database(BasicAuth(req.extensions_mut().remove().unwrap()))
        .map(|user: User| {
            HttpResponse::Ok().etag(&user).json(json!({
                "data": user,
                "token": user.generate_token()
            }))
        }).responder()
}

pub fn me(req: &HttpRequest<PointercrateState>) -> impl Responder {
    info!("GET /api/v1/auth/me/");

    req.state()
        .database(TokenAuth(req.extensions_mut().remove().unwrap()))
        .map(|user: User| HttpResponse::Ok().json_with_etag(user))
        .responder()
}
