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
) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    let mut sesv2_addresses_stream = sesv2_client
        .list_suppressed_destinations()
        .page_size(1000)
        .into_paginator()
        .send();

    let mut emails = Vec::new();

    while let Some(addresses) = sesv2_addresses_stream.next().await {
        info!("Addresses: {:?}", addresses);

        for address in addresses.unwrap().suppressed_destination_summaries() {
            debug!("Address: {:?}", address);
            let email = address.email_address().to_string();
            let reason = address.reason().to_string();
            emails.push((email, reason));
        }
        thread::sleep(time::Duration::from_millis(1000));
    }
    // Err("Address not found".into())
    Ok(emails)
}
