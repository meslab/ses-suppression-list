use clap::Parser;
use log::debug;
use ses_suppression_list::sesv2;
use std::fs::File;
use std::io::Write;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[clap(
    version = "v0.0.1",
    author = "Anton Sidorov tonysidrock@gmail.com",
    about = "Exports ses suppression list"
)]
struct Args {
    #[clap(short, long, default_value = "eu-central-1")]
    region: String,

    #[clap(short, long, default_value = "suppressed_emails.csv")]
    output: String,

    #[clap(short, long, default_value = "default")]
    profile: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args = Args::parse();

    let sesv2_client = sesv2::initialize_client(&args.region, &args.profile).await;

    if let Ok(r) = sesv2::get_suppression_list(&sesv2_client).await {
        debug!("Result: {:?}", &r);
        let mut file = File::create(format!("./{}", &args.output)).unwrap();

        for (email,reason) in &r {
            writeln!(file, "{},{}", email, reason).unwrap();
        }
        println!("Total {} email addresses", r.len())
    }

    Ok(())
}
