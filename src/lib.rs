#![forbid(unsafe_code)]
#![deny(clippy::all)]

#[macro_use]
extern crate lazy_static;

use chrono::Utc;
use chrono::{NaiveDate, ParseError as ChronoParseError};
use reqwest::{Client, Error as ReqwestError};
use scraper::{Html, Selector};
use serde::{Serialize, Deserialize};

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

#[derive(Debug)]
pub enum Errors {
    ReqwestClientError(ReqwestError),
    ParserError,
    SuppliedDateHasNoDraw,
    UnexpectedParsingError(String, String),
    DateParsingError(ChronoParseError),
}

impl From<ReqwestError> for Errors {
    fn from(e: ReqwestError) -> Self {
        Self::ReqwestClientError(e)
    }
}

impl From<ChronoParseError> for Errors {
    fn from(e: ChronoParseError) -> Self {
        Self::DateParsingError(e)
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

        Self::parse_draw_from_html(&res.text().await?, None)
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

        Self::parse_draw_from_html(&res.text().await?, Some(date))
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

        Self::parse_draw_from_html(&res.text().await?, None)
    }

    fn parse_draw_from_html(html: &str, date: Option<NaiveDate>) -> Result<LottoDraw, Errors> {
        let document = Html::parse_document(html);

        let mut lotto_draw = LottoDraw::default();

        // Check date

        let formatted_date_count = document.select(&FORMATTED_DATE_SELECTOR).count() as u8;

        if formatted_date_count != 1 {
            return Err(Errors::UnexpectedParsingError(format!(
                "Expected 1 date element in html response, found {}",
                formatted_date_count
            ), html.to_string()));
        }

        let formatted_date_value = document
            .select(&FORMATTED_DATE_SELECTOR)
            .last()
            .unwrap()
            .value()
            .attr("value");

        if let Some(date_value) = formatted_date_value {
            lotto_draw.date = NaiveDate::parse_from_str(date_value, "%d.%m.%Y")?;
            
            if let Some(check_date_value) = date {
                if lotto_draw.date != check_date_value {
                    return Err(Errors::SuppliedDateHasNoDraw);
                }
            }
        }

        // Normal numbers

        let normal_numbers_count = document.select(&NORMAL_NUMBER_SELECTOR).count() as u8;

        if normal_numbers_count != 6 {
            return Err(Errors::UnexpectedParsingError(format!(
                "Expected 6 normal numbers in html response, found {}",
                normal_numbers_count
            ), html.to_string()));
        }

        for (index, element) in document.select(&NORMAL_NUMBER_SELECTOR).enumerate() {
            lotto_draw.numbers[index] = element.inner_html().parse::<u8>().unwrap();
        }

        // lucky number

        let lucky_numbers_count = document.select(&LUCKY_NUMBER_SELECTOR).count() as u8;

        if lucky_numbers_count != 1 {
            return Err(Errors::UnexpectedParsingError(format!(
                "Expected 1 lucky number in html response, found {}",
                lucky_numbers_count
            ), html.to_string()));
        }

        lotto_draw.lucky = document
            .select(&LUCKY_NUMBER_SELECTOR)
            .last()
            .unwrap()
            .inner_html()
            .parse::<u8>()
            .unwrap();

        // replay number

        let replay_numbers_count = document.select(&REPLAY_NUMBER_SELECTOR).count();

        if replay_numbers_count != 1 {
            return Err(Errors::UnexpectedParsingError(format!(
                "Expected 1 replay number in html response, found {}",
                replay_numbers_count
            ), html.to_string()));
        }

        lotto_draw.replay = document
            .select(&REPLAY_NUMBER_SELECTOR)
            .last()
            .unwrap()
            .inner_html()
            .parse::<u8>()
            .unwrap();

        Ok(lotto_draw)
    }
}
