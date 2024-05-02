use aws_config::default_provider::credentials::DefaultCredentialsChain;
use aws_sdk_sesv2::config::Region;
use aws_sdk_sesv2::{Client, Config};
use log::{debug, info};
use std::{thread, time};

pub async fn initialize_client(region: &str, profile: &str) -> Client {
    let credentials_provider = DefaultCredentialsChain::builder()
        .profile_name(profile)
        .build()
        .await;
    let sesv2_config = Config::builder()
        .credentials_provider(credentials_provider)
        .region(Region::new(region.to_string()))
        .build();

    Client::from_conf(sesv2_config)
}

pub async fn get_suppression_list(
    sesv2_client: &Client,
    last_count_days: Option<u32>,
) -> Result<Vec<(String, String, String)>, Box<dyn std::error::Error>> {
    let mut sesv2_addresses_stream = sesv2_client
        .list_suppressed_destinations()
        .page_size(1000)
        .into_paginator()
        .send();

    let mut emails = Vec::new();

    while let Some(addresses) = sesv2_addresses_stream.next().await {
        debug!("Addresses: {:?}", addresses);

        for address in addresses.unwrap().suppressed_destination_summaries() {
            debug!("Address: {:?}", address);
            let now = chrono::Utc::now().naive_utc();
            let date = address.last_update_time().to_string();
            match last_count_days {
                None => {
                    let email = address.email_address().to_string();
                    let reason = address.reason().to_string();
                    emails.push((email, reason, date));
                }
                Some(last) => {
                    info!("Date: {:?}", date);
                    let time_date =
                        match chrono::NaiveDateTime::parse_from_str(&date, "%Y-%m-%dT%H:%M:%S.%fZ")
                        {
                            Ok(time) => time,
                            Err(_) => {
                                chrono::NaiveDateTime::parse_from_str(&date, "%Y-%m-%dT%H:%M:%SZ")?
                            }
                        };

                    let duration = now - time_date;
                    if duration.num_days() < last as i64 {
                        info!("Duration: {:?}", duration.num_days());
                        let email = address.email_address().to_string();
                        let reason = address.reason().to_string();
                        info!("Email: {}", &email);
                        emails.push((email, reason, date));
                    }
                }
            }
        }
        thread::sleep(time::Duration::from_millis(1000));
    }
    // Err("Address not found".into())
    Ok(emails)
}
