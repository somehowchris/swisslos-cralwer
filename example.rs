use chrono::{Datelike, Weekday};
use swisslos_crawler::SwissLottoClient;

#[tokio::main]
async fn main() {
    let client = SwissLottoClient::default();

    println!("{:?}", client.get_latest_draw().await);

    let today = chrono::offset::Local::today();
    let mon =
        chrono::NaiveDate::from_isoywd(today.year(), today.iso_week().week() - 1, Weekday::Thu);
    println!("{:?}", client.get_previous_draw(mon).await);
}
