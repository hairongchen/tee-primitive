use tonic::{transport::Server, Request, Response, Status};

use tee_primitive::get_evidence_server::{GetEvidence, GetEvidenceServer};
use tee_primitive::{GetQuoteRequest, QuoteReply, GetImaRequest, ImaReply, GetCcelRequest, CcelReply};

pub mod tdx;
use tdx::*;

pub mod tee_primitive {
    tonic::include_proto!("teeprimitive");
}

#[derive(Debug, Default)]
pub struct TDXGetEvidence {}

#[tonic::async_trait]
impl GetEvidence for TDXGetEvidence {
    async fn get_quote(
        &self,
        request: Request<GetQuoteRequest>,
    ) -> Result<Response<QuoteReply>, Status> {
        println!("Got get_quote request: {:?}", request);

        let reply = tee_primitive::QuoteReply {
            quote: format!("{}", tdx_get_quote(request.into_inner().reportdata).unwrap()).into(),
        };

        Ok(Response::new(reply))
    }

    async fn get_ima(
        &self,
        request: Request<GetImaRequest>,
    ) -> Result<Response<ImaReply>, Status> {
        println!("Got get_ima request: {:?}", request);

        let reply = tee_primitive::ImaReply {
            ima: format!("{}",tdx_get_ima().unwrap()).into(),
        };

        Ok(Response::new(reply))
    }

    async fn get_ccel(
        &self,
        request: Request<GetCcelRequest>,
    ) -> Result<Response<CcelReply>, Status> {
        println!("Got get_ccel request: {:?}", request);

        let reply = tee_primitive::CcelReply {
            ccel: format!("{}",tdx_get_ccel().unwrap()).into(),
        };

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:6789".parse()?;
    let getevidence = TDXGetEvidence::default();

    Server::builder()
        .add_service(GetEvidenceServer::new(getevidence))
        .serve(addr)
        .await?;

    Ok(())
}
