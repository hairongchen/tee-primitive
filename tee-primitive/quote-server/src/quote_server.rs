use clap::Parser;
use tonic::{transport::Server, Request, Response, Status};
use quote_server::get_quote_server::{GetQuote, GetQuoteServer};
use quote_server::{GetQuoteRequest, QuoteReply};

pub mod tee;
use tee::*;

pub mod quote_server {
    tonic::include_proto!("quoteserver");

    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("quote_server_descriptor");
}

pub struct CCNPGetQuote {
  local_tee: tee::TeeType,
}

impl CCNPGetQuote {
    fn new(local_tee: TeeType) -> Self {
        CCNPGetQuote {
            local_tee: local_tee,
        }
    }
}

#[tonic::async_trait]
impl GetQuote for CCNPGetQuote {
    async fn get_quote(
        &self,
        request: Request<GetQuoteRequest>,
    ) -> Result<Response<QuoteReply>, Status> {
        println!("Got get_quote request: {:?}", request);

        let msg;
        let result = get_quote(self.local_tee.clone(),request.into_inner().report_data);
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
    let service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(quote_server::FILE_DESCRIPTOR_SET)
        .build()
        .unwrap();

    let args = Cli::parse();
    let port = args.port;
    let addr = format!("0.0.0.0:{}", port).parse()
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let getquote = CCNPGetQuote::new(
        {
            match tee::get_tee_type() {
                tee::TeeType::PLAIN => panic!("Not found any TEE device"),
                t => t,
            }
        }
    );

    Server::builder()
        .add_service(service)
        .add_service(GetQuoteServer::new(getquote))
        .serve(addr)
        .await?;

    Ok(())
}
