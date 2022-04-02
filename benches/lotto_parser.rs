#![feature(test)]

extern crate test;

#[cfg(test)]
mod tests {
    use crate::*;
    use test::Bencher;
    use swisslos_crawler::SwissLottoClient;

    const WEBPAGE: &str = include_str!("./winning-numbers.html");

    #[bench]
    fn bench_pow(b: &mut Bencher) {
        let client = SwissLottoClient::default();
        b.iter(|| {
            client.parse_draw_from_html(&WEBPAGE, None).unwrap();
        });
    }
}