use clap::Parser;
use tonic::{transport::Server, Request, Response, Status};
use quote_server::get_quote_server::{GetQuote, GetQuoteServer};
use quote_server::{GetQuoteRequest, QuoteReply};

pub mod tee;
use tee::*;

pub mod quote_server {
    tonic::include_proto!("quoteserver");
}

#[derive(Debug, Default)]
pub struct CCNPGetQuote {}

#[tonic::async_trait]
impl GetQuote for CCNPGetQuote {
    async fn get_quote(
        &self,
        request: Request<GetQuoteRequest>,
    ) -> Result<Response<QuoteReply>, Status> {
        //println!("Got get_quote request: {:?}", request);
        let msg;
        let result = get_quote(request.into_inner().report_data);
        match result  {
            Ok(q) => msg = Response::new(quote_server::QuoteReply {quote: format!("{}", q),}),
            Err(e) => return Err(Status::internal(e.to_string())),
        }
        Ok(msg)
    }
}

#[derive(Parser)]
struct Cli {
    port: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    let port = args.port;
    let addr = format!("0.0.0.0:{}", port).parse()
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    let getquote = CCNPGetQuote::default();

    Server::builder()
        .add_service(GetQuoteServer::new(getquote))
        .serve(addr)
        .await?;
    Ok(())
}


