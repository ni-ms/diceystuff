#[macro_use]
extern crate rocket;

use rand::Rng;
use rocket::http::{Status};
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::fs::{relative, FileServer};
use sha2::{Sha256, Digest};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::log::private::debug;
use rocket::{Request, Response};

#[derive(Serialize, Deserialize)]
struct DiceBet {
    number: String,
    amount: String,
    multiplier: String,
}

#[derive(Serialize, Deserialize)]
struct DiceResult {
    payout: f64,
    hash: String,
}

#[post("/", format = "json", data = "<dice_bet>")]
async fn dice(dice_bet: Json<DiceBet>) -> Result<Json<DiceResult>, Status> {
    let number = dice_bet.number.parse::<i32>().unwrap_or(0);
    let amount = dice_bet.amount.parse::<f64>().unwrap_or(0.0);
    let multiplier = dice_bet.multiplier.parse::<f64>().unwrap_or(0.0);

    let generated_number = rand::thread_rng().gen_range(1..=12);
    let payout = if generated_number == number {
        amount * multiplier
    } else {
        0.0
    };

    let mut hasher = Sha256::new();
    hasher.update(generated_number.to_string());
    let hash = format!("{:x}", hasher.finalize());

    let dice_result = DiceResult {
        payout,
        hash,
    };

    Ok(Json(dice_result))
}

pub struct Cors;

#[rocket::async_trait]
impl Fairing for Cors {
    fn info(&self) -> Info {
        Info {
            name: "Cross-Origin-Resource-Sharing Fairing",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, PATCH, PUT, DELETE, HEAD, OPTIONS, GET",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(Cors)
        .mount("/", routes![dice])
        .mount("/", FileServer::from(relative!("static")))
}
