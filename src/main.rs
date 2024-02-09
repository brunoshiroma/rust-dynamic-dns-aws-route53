use aws_config::BehaviorVersion;
use clap::Parser;
use hyper::{Body, Uri};

use aws_sdk_route53 as route53;
use hyper_openssl::HttpsConnector;
use route53::{
    config::Region,
    types::{
        Change, ChangeAction, ChangeBatch, CidrRoutingConfig, ResourceRecord, ResourceRecordSet,
    },
};
use std::str;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    dev: bool,

    #[arg(short, long)]
    name: String,
}

#[::tokio::main]
async fn main() -> Result<(), route53::Error> {
    let args = Args::parse();
    let ssl = HttpsConnector::new().unwrap();

    let config = match args.dev {
        true => {
            ::aws_config::defaults(BehaviorVersion::latest())
                .endpoint_url("http://127.0.0.1:4566")
                .load()
                .await
        }
        _ => ::aws_config::load_from_env().await,
    };

    let http_client = hyper::Client::builder().build::<_, Body>(ssl);

    let ip_res = http_client
        .get(Uri::from_static(
            "https://dev-toolbelt.brunoshiroma.com/network/ip",
        ))
        .await;

    let ip = hyper::body::to_bytes(ip_res.unwrap()).await;
    let ip_vec = ip.unwrap().to_vec();
    let str_ip = str::from_utf8(&ip_vec).unwrap();
    print!("body {}", str_ip);

    let client =
        route53::Client::from_conf(aws_sdk_route53::config::Builder::from(&config).build());

    let hosted_zones = client.list_hosted_zones().send().await?;
    for hz in hosted_zones.hosted_zones().iter() {
        println!("ID {}", hz.name);
    }

    let record = ResourceRecordSet::builder()
        .resource_records(ResourceRecord::builder().value(str_ip).build().unwrap())
        .name("test")
        .r#type(route53::types::RrType::A)
        .build();

    let change = Change::builder()
        .action(ChangeAction::Upsert)
        .resource_record_set(record.unwrap())
        .build();

    let change_batch = ChangeBatch::builder().changes(change.unwrap()).build();

    let upsert_result = client
        .change_resource_record_sets()
        .hosted_zone_id("A")
        .change_batch(change_batch.unwrap())
        .send()
        .await?;

    print!("result {:?}", upsert_result);

    Ok(())
}
