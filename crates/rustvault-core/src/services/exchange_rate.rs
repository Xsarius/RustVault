//! Exchange rate service — fetches daily rates from the ECB XML feed.

use rust_decimal::Decimal;
use std::str::FromStr;
use time::Date;
use time::macros::format_description;

use crate::error::CoreError;
use rustvault_db::repos::exchange_rate::UpsertRate;

const ECB_FEED_URL: &str =
    "https://www.ecb.europa.eu/stats/eurofxref/eurofxref-daily.xml";

/// Fetch today's rates from the ECB daily XML feed.
///
/// Returns a list of [`UpsertRate`] records ready for batch upsert, all with
/// `base_currency = "EUR"`.
///
/// # Errors
///
/// Returns [`CoreError::ExternalService`] if the HTTP request or XML parsing fails.
pub async fn fetch_ecb_rates() -> Result<Vec<UpsertRate>, CoreError> {
    let body = reqwest::get(ECB_FEED_URL)
        .await
        .map_err(|e| CoreError::ExternalService(format!("ECB feed request failed: {e}")))?
        .text()
        .await
        .map_err(|e| CoreError::ExternalService(format!("ECB feed body read failed: {e}")))?;

    parse_ecb_xml(&body)
        .map_err(|e| CoreError::ExternalService(format!("ECB XML parse failed: {e}")))
}

/// Parse the ECB eurofxref XML into a list of upsertable rate records.
fn parse_ecb_xml(xml: &str) -> Result<Vec<UpsertRate>, String> {
    // The ECB feed structure:
    //   <gesmes:Envelope>
    //     <Cube>
    //       <Cube time="YYYY-MM-DD">
    //         <Cube currency="USD" rate="1.08"/>
    //         ...
    //       </Cube>
    //     </Cube>
    //   </gesmes:Envelope>

    let mut rates = Vec::new();
    let mut current_date: Option<Date> = None;

    // Simple line-based parser — avoids pulling in a full XML crate just for this feed.
    for line in xml.lines() {
        let line = line.trim();

        // Date line: <Cube time="2024-03-12">
        if line.starts_with("<Cube time=") {
            if let Some(date_str) = extract_attr(line, "time") {
                let format = format_description!("[year]-[month]-[day]");
                current_date = Date::parse(&date_str, &format).ok();
            }
            continue;
        }

        // Rate line: <Cube currency="USD" rate="1.0832"/>
        if line.starts_with("<Cube currency=") {
            if let (Some(date), Some(currency), Some(rate_str)) = (
                current_date,
                extract_attr(line, "currency"),
                extract_attr(line, "rate"),
            ) {
                if let Ok(rate) = Decimal::from_str(&rate_str) {
                    rates.push(UpsertRate {
                        base_currency: "EUR".into(),
                        target_currency: currency,
                        rate,
                        date,
                        source: "ecb".into(),
                    });
                }
            }
        }
    }

    if rates.is_empty() {
        return Err("no rates parsed from ECB feed".into());
    }

    Ok(rates)
}

/// Extract the value of an XML attribute from a single-line tag string.
///
/// E.g. given `<Cube currency="USD" rate="1.08"/>` and key `"currency"`,
/// returns `Some("USD")`.
fn extract_attr(line: &str, key: &str) -> Option<String> {
    let needle = format!("{key}=\"");
    let start = line.find(&needle)? + needle.len();
    let end = line[start..].find('"')? + start;
    Some(line[start..end].to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ecb_xml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<gesmes:Envelope xmlns:gesmes="http://www.gesmes.org/xml/2002-08-01"
    xmlns="http://www.ecb.int/vocabulary/2002-08-01/eurofxref">
    <Cube>
        <Cube time="2026-03-12">
            <Cube currency="USD" rate="1.0832"/>
            <Cube currency="PLN" rate="4.2100"/>
        </Cube>
    </Cube>
</gesmes:Envelope>"#;

        let rates = parse_ecb_xml(xml).expect("should parse");
        assert_eq!(rates.len(), 2);
        assert_eq!(rates[0].base_currency, "EUR");
        assert_eq!(rates[0].target_currency, "USD");
        assert_eq!(rates[0].rate.to_string(), "1.0832");
    }
}
