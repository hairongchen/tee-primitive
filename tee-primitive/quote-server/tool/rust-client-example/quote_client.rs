use quote_server::get_quote_client::GetQuoteClient;
use quote_server::GetQuoteRequest;

pub mod quote_server {
    tonic::include_proto!("quoteserver");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = GetQuoteClient::connect("http://0.0.0.0:6789").await?;

    //get Quote
    let request = tonic::Request::new(GetQuoteRequest {
        report_data: base64::encode("123456781234567812345678123456781234567812345678"),
    });
    let response = client.get_quote(request).await?;
    println!("RESPONSE={:?}", response);

    Ok(())
}