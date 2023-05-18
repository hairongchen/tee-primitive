use anyhow::*;
use std::path::Path;
use tdx_attest_rs;

#[derive(Debug,Clone)]
pub enum TeeType {
    TDX,
    SEV,
    TPM,
    PLAIN
}

pub fn get_tee_type() -> TeeType {
    if Path::new("/dev/tdx-guest").exists() {
        return TeeType::TDX;
    }else if Path::new("/dev/tpm0").exists(){
        return TeeType::TPM;
    }else if Path::new("/dev/sev-guest").exists() || Path::new("/dev/sev").exists(){
        return TeeType::SEV;
    }else {
        return TeeType::PLAIN;
    }
}

pub fn get_tdx_quote(report_data: Option<String>) -> Result<String> {

    let tdx_report_data = match report_data {
        Some(_report_data) => {
            if _report_data.len() == 0{
                tdx_attest_rs::tdx_report_data_t { d: [0u8; 64usize] }
            } else {
                let mut _tdx_report_data = base64::decode(_report_data.clone())?;
                if _tdx_report_data.len() != 48 {
                    return Err(anyhow!(
                        "get_tdx_quote: runtime data should be SHA384 base64 String of 48 bytes"
                    ));
                }
                _tdx_report_data.extend([0; 16]);
                tdx_attest_rs::tdx_report_data_t {
                    d: _tdx_report_data.as_slice().try_into()?,
                }
            }
        },
        None =>  tdx_attest_rs::tdx_report_data_t { d: [0u8; 64usize] },
    };

    let quote = match tdx_attest_rs::tdx_att_get_quote(Some(&tdx_report_data), None, None, 0) {
        (tdx_attest_rs::tdx_attest_error_t::TDX_ATTEST_SUCCESS, Some(q)) => base64::encode(q),
        (error_code, _) => {
            return Err(anyhow!(
                "get_tdx_quote: {:?}",
                error_code
            ));
        }
    };

    serde_json::to_string(&quote).map_err(|e| anyhow!("get_tdx_quote: {:?}", e))
}

pub fn get_tpm_quote() -> Result<String> {
    Err(anyhow!("TPM to be supported!"))
}

pub fn get_sev_quote() -> Result<String> {
    Err(anyhow!("SEV to be supported!"))
}

pub fn get_quote(local_tee: TeeType, report_data: String) -> Result<String> {
    match local_tee {
        TeeType::TDX => return get_tdx_quote(Some(report_data)),
        TeeType::TPM => return get_tpm_quote(),
        TeeType::SEV => return get_sev_quote(),
        _ => return Err(anyhow!("Unexpected case!")),
    }
}

#[test]
fn tdx_report_data_size_8() {
    // "YWJjZGVmZw==" is base64 of "abcdefg", 8 bytes
    let result = get_tdx_quote(Some("YWJjZGVmZw==".to_string()));
    assert!(result.is_err());
}

#[test]
fn tdx_report_data_size_0() {
    //allow does not specify report data
    let result = get_tdx_quote(Some("".to_string()));
    assert!(result.is_ok());
}

#[test]
fn tdx_report_data_size_48() {
    // this one should be standard 48 bytes base64 encoded report data
    // "MTIzNDU2NzgxMjM0NTY3ODEyMzQ1Njc4MTIzNDU2NzgxMjM0NTY3ODEyMzQ1Njc4" is base64 of "123456781234567812345678123456781234567812345678", 48 bytes
    let result = get_tdx_quote(Some("MTIzNDU2NzgxMjM0NTY3ODEyMzQ1Njc4MTIzNDU2NzgxMjM0NTY3ODEyMzQ1Njc4".to_string()));
    assert!(result.is_ok());
}

#[test]
fn tdx_report_data_null() {
    // allow call get_tdx_quote with out specify report data
    let result = get_tdx_quote(None);
    assert!(result.is_ok());
}

#[test]
fn tdx_report_data_not_base64_encoded() {
    //does not allow not base64 encoded report data
    let result = get_tdx_quote(Some("123456781234567812345678123456781234567812345678".to_string()));
    assert!(result.is_err());
}
