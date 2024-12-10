use rocket::{
    response::{status, Redirect},
};

#[rocket::get("/")]
pub async fn serve() -> Result<Redirect, status::Custom<String>> {
    Ok(Redirect::to("https://cscl.pages.dev/#/"))
}

#[shuttle_runtime::main]
async fn main() -> shuttle_rocket::ShuttleRocket {
    let rocket = rocket::build().mount("/", rocket::routes![serve]);

    Ok(rocket.into())
}