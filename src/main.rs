use goose::prelude::*;
//use kbs_types::{Attestation, Challenge, ErrorInformation, Request, Response, Tee};
use anyhow::*;
use kbs_types::{Attestation, Request, Tee};
use std::fs;
use std::result::Result::Ok;

const KBS_PROTOCOL_VERSION: &str = "0.1.0";

#[tokio::main]
async fn main() -> Result<()> {
    let _ = GooseAttack::initialize()?
        .register_scenario(scenario!("kbs_client_rcar").register_transaction(transaction!(rcar)))
        .execute()
        .await
        .map_err(|e| anyhow!("GooseAttack initialize failed: {}", e));

    Ok(())
}

async fn rcar(user: &mut GooseUser) -> TransactionResult {
    let request = Request {
        version: String::from(KBS_PROTOCOL_VERSION),
        tee: Tee::Tdx,
        extra_params: String::new(),
    };

    let request_builder = user
        .get_request_builder(&GooseMethod::Post, "/kbs/v0/auth")?
        .header("Content-Type", "application/json")
        .json(&request);

    let goose_request = GooseRequest::builder()
        .name("auth")
        .set_request_builder(request_builder)
        .expect_status_code(200)
        .build();

    let _ = user.request(goose_request).await?;

    let data = fs::read_to_string("./attest.req.json").expect("failed to load ATTEST_REQ");
    let attestation: Attestation = serde_json::from_str(&data).unwrap();

    let request_builder = user
        .get_request_builder(&GooseMethod::Post, "/kbs/v0/attest")?
        .header("Content-Type", "application/json")
        .json(&attestation);

    let goose_request = GooseRequest::builder()
        .name("attest")
        .set_request_builder(request_builder)
        .expect_status_code(200)
        .build();

    let goose = user.request(goose_request).await?;
    match goose.response {
        Ok(response) => {
            let headers = &response.headers().clone();
            if !response.status().is_success() {
                match response.text().await {
                    Ok(html) => {
                        let _ = user.log_debug(
                            "error loading /",
                            Some(&goose.request),
                            Some(headers),
                            Some(&html),
                        );
                    }
                    Err(e) => {
                        let _ = user.log_debug(
                            &format!("error loading /: {}", e),
                            Some(&goose.request),
                            Some(headers),
                            None,
                        );
                    }
                }
            }
        }
        Err(_e) => {
            let _ = user.log_debug(
                "no response from server when loading /",
                Some(&goose.request),
                None,
                None,
            );
        }
    }

    Ok(())
}
