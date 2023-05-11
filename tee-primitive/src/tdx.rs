use anyhow::*;
use std::path::Path;
use tdx_attest_rs;

pub fn is_tdx_platform() -> bool {
    Path::new("/dev/tdx-attest").exists()
}

#[derive(Debug, Default)]
pub struct TdxEvidence {}

pub fn tdx_get_quote(report_data: String) -> Result<String> {

        //println!("runtime data: {}", report_data);
        let mut report_data_bin = base64::decode(report_data.clone())?;
        //println!("runtime data size: {}",  report_data_bin.len());
        println!("runtime data decoded: {:?}",  std::str::from_utf8(&report_data_bin));

        if report_data_bin.len() != 48 {
            return Err(anyhow!(
                "TDX Evidence: runtime data should be SHA384 base64 String"
            ));
        }
        report_data_bin.extend([0; 16]);

        let tdx_report_data = tdx_attest_rs::tdx_report_data_t {
            d: report_data_bin.as_slice().try_into()?,
        };

        let quote = match tdx_attest_rs::tdx_att_get_quote(Some(&tdx_report_data), None, None, 0) {
            (tdx_attest_rs::tdx_attest_error_t::TDX_ATTEST_SUCCESS, Some(q)) => base64::encode(q),
            (error_code, _) => {
                return Err(anyhow!(
                    "TDX Attester: Failed to get TD quote. Error code: {:?}",
                    error_code
                ));
            }
        };

        serde_json::to_string(&quote)
            .map_err(|e| anyhow!("Serialize TDX quote failed: {:?}", e))
}

pub fn tdx_get_ima() -> Result<String> {
    const IMA_PATH: &str = "/sys/kernel/security/ima/ascii_runtime_measurements";

    let ima_eventlog = match std::fs::read(IMA_PATH) {
        Result::Ok(el) => Some(base64::encode(el)),
        Result::Err(e) => {
            log::warn!("Read IMA Eventlog failed: {:?}", e);
            None
        }
    };
    serde_json::to_string(&ima_eventlog)
    .map_err(|e| anyhow!("Serialize IMA Eventlog failed: {:?}", e))    
}

pub fn tdx_get_ccel() -> Result<String> {
    const CCEL_PATH: &str = "/sys/firmware/acpi/tables/data/CCEL";

    let cc_eventlog = match std::fs::read(CCEL_PATH) {
        Result::Ok(el) => Some(base64::encode(el)),
        Result::Err(e) => {
            log::warn!("Read CC Eventlog failed: {:?}", e);
            None
        }
    };
    serde_json::to_string(&cc_eventlog)
    .map_err(|e| anyhow!("Serialize IMA Eventlog  failed: {:?}", e))    
}
