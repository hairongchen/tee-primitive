use tee_primitive::get_evidence_client::GetEvidenceClient;
use tee_primitive::{GetQuoteRequest, GetImaRequest, GetCcelRequest};

pub mod tee_primitive {
    tonic::include_proto!("teeprimitive");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = GetEvidenceClient::connect("http://0.0.0.0:6789").await?;

    //get Quote
    let request = tonic::Request::new(GetQuoteRequest {
        runtimedata: "abcdefgabcdefgabcdefgabcdefgabcdefgabcdefgabcdefgabcdefg".into(),
    });

    let response = client.get_quote(request).await?;

    println!("RESPONSE={:?}", response);

    //get IMA
    let request1 = tonic::Request::new(GetImaRequest {});

    let response1 = client.get_ima(request1).await?;

    println!("RESPONSE={:?}", response1);

    ////get CCEL
    let request2 = tonic::Request::new(GetCcelRequest {});

    let response2 = client.get_ccel(request2).await?;

    println!("RESPONSE={:?}", response2);

    Ok(())
}
