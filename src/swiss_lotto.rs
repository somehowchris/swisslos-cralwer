use chrono::{NaiveDate, Utc};
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

use super::Errors;

const SWISS_LOTTO_DRAW_URL: &str =
    "https://www.swisslos.ch/en/swisslotto/information/winning-numbers/winning-numbers.html";

lazy_static! {
    static ref FORMATTED_DATE_SELECTOR: Selector =
        Selector::parse("input#formattedFilterDate").unwrap();

    static ref NORMAL_NUMBER_SELECTOR: Selector = Selector::parse(
        ".filter-results .quotes__game .actual-numbers__numbers .actual-numbers__number___normal span"
    )
    .unwrap();

    static ref LUCKY_NUMBER_SELECTOR: Selector = Selector::parse(
        ".filter-results .quotes__game .actual-numbers__numbers .actual-numbers__number___lucky span"
    )
    .unwrap();

    static ref REPLAY_NUMBER_SELECTOR: Selector = Selector::parse(
        ".filter-results .quotes__game .actual-numbers__numbers .actual-numbers__number___replay span"
    )
    .unwrap();
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LottoDraw {
    pub date: NaiveDate,
    pub numbers: [u8; 6],
    pub lucky: u8,
    pub replay: u8,
}

impl Default for LottoDraw {
    fn default() -> Self {
        Self {
            date: Utc::today().naive_utc(),
            numbers: [0, 0, 0, 0, 0, 0],
            lucky: 0,
            replay: 0,
        }
    }
}

pub struct SwissLottoClient {
    client: Client,
}

impl Default for SwissLottoClient {
    fn default() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

impl SwissLottoClient {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

impl SwissLottoClient {
    pub async fn get_latest_draw(&self) -> Result<LottoDraw, Errors> {
        let res = self.client.get(SWISS_LOTTO_DRAW_URL).send().await?;

        self.parse_draw_from_html(&res.text().await?, None)
    }

    pub async fn get_draw_of_date(&self, date: NaiveDate) -> Result<LottoDraw, Errors> {
        let formatted_date = date.format("%d.%m.%Y").to_string();
        let res = self
            .client
            .post(SWISS_LOTTO_DRAW_URL)
            .form(&[
                ("formattedFilterDate", &formatted_date),
                ("filterDate", &formatted_date),
                ("currentDate", &Utc::today().format("%d.%m.%Y").to_string()),
            ])
            .send()
            .await?;

        self.parse_draw_from_html(&res.text().await?, Some(date))
    }

    pub async fn get_previous_draw(&self, date: NaiveDate) -> Result<LottoDraw, Errors> {
        let formatted_date = date.format("%d.%m.%Y").to_string();
        let res = self
            .client
            .post(SWISS_LOTTO_DRAW_URL)
            .form(&[
                ("formattedFilterDate", &formatted_date),
                ("filterDate", &formatted_date),
                ("currentDate", &Utc::today().format("%d.%m.%Y").to_string()),
            ])
            .send()
            .await?;

        self.parse_draw_from_html(&res.text().await?, None)
    }

    pub fn parse_draw_from_html(
        &self,
        html: &str,
        date: Option<NaiveDate>,
    ) -> Result<LottoDraw, Errors> {
        let document = Html::parse_document(html);

        let mut lotto_draw = LottoDraw::default();

        // Check date

        let formatted_dates = document
            .select(&FORMATTED_DATE_SELECTOR)
            .collect::<Vec<_>>();

        let formatted_date_count = formatted_dates.len() as u8;

        if formatted_date_count != 1 {
            return Err(Errors::UnexpectedParsingError(
                format!(
                    "Expected 1 date element in html response, found {}",
                    formatted_date_count
                ),
                html.to_string(),
            ));
        }

        let formatted_date_value = formatted_dates.last().unwrap().value().attr("value");

        if let Some(date_value) = formatted_date_value {
            lotto_draw.date = NaiveDate::parse_from_str(date_value, "%d.%m.%Y")?;
            if let Some(check_date_value) = date {
                if lotto_draw.date != check_date_value {
                    return Err(Errors::SuppliedDateHasNoDraw);
                }
            }
        }

        // Normal numbers

        let normal_numbers = document.select(&NORMAL_NUMBER_SELECTOR).collect::<Vec<_>>();

        let normal_numbers_count = normal_numbers.len();

        if normal_numbers_count != 6 {
            return Err(Errors::UnexpectedParsingError(
                format!(
                    "Expected 6 normal numbers in html response, found {}",
                    normal_numbers_count
                ),
                html.to_string(),
            ));
        }

        for (index, element) in normal_numbers.iter().enumerate() {
            lotto_draw.numbers[index] = element.inner_html().parse().unwrap();
        }

        // lucky number

        let lucky_numbers = document.select(&LUCKY_NUMBER_SELECTOR).collect::<Vec<_>>();

        let lucky_numbers_count = lucky_numbers.len();

        if lucky_numbers_count != 1 {
            return Err(Errors::UnexpectedParsingError(
                format!(
                    "Expected 1 lucky number in html response, found {}",
                    lucky_numbers_count
                ),
                html.to_string(),
            ));
        }

        lotto_draw.lucky = lucky_numbers.last().unwrap().inner_html().parse().unwrap();

        // replay number
        let replay_numbers = document.select(&REPLAY_NUMBER_SELECTOR).collect::<Vec<_>>();

        let replay_numbers_count = replay_numbers.len();

        if replay_numbers_count != 1 {
            return Err(Errors::UnexpectedParsingError(
                format!(
                    "Expected 1 replay number in html response, found {}",
                    replay_numbers_count
                ),
                html.to_string(),
            ));
        }

        lotto_draw.replay = replay_numbers.last().unwrap().inner_html().parse().unwrap();

        Ok(lotto_draw)
    }
}
