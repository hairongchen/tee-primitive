use anyhow::*;
use std::path::Path;
use tdx_attest_rs;

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

pub fn get_tdx_quote(report_data: String) -> Result<String> {

    let mut report_data_bin = base64::decode(report_data.clone())?;
    //println!("runtime data decoded: {:?}",  std::str::from_utf8(&report_data_bin));

    if report_data_bin.len() != 48 {
        return Err(anyhow!(
            "get_tdx_quote: runtime data should be SHA384 base64 String of 48 bytes"
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

pub fn get_quote(report_data: String) -> Result<String> {
    match get_tee_type(){
        TeeType::TDX => return get_tdx_quote(report_data),
        TeeType::TPM => return get_tpm_quote(),
        TeeType::SEV => return get_sev_quote(),
        TeeType::PLAIN => return Err(anyhow!("get_quote: No TEE env found!")),
    }
}

#[test]
fn tdx_report_data_size_8() {
    // "YWJjZGVmZw==" is base64 of "abcdefg", 8 bytes
    let result = get_tdx_quote("YWJjZGVmZw==".to_string());
    assert!(result.is_err());
}

#[test]
fn tdx_report_data_size_0() {
    let result = get_tdx_quote("".to_string());
    assert!(result.is_err());
}

#[test]
fn tdx_report_data_size_48() {
    // "MTIzNDU2NzgxMjM0NTY3ODEyMzQ1Njc4MTIzNDU2NzgxMjM0NTY3ODEyMzQ1Njc4" is base64 of "123456781234567812345678123456781234567812345678", 48 bytes
    let result = get_tdx_quote("MTIzNDU2NzgxMjM0NTY3ODEyMzQ1Njc4MTIzNDU2NzgxMjM0NTY3ODEyMzQ1Njc4".to_string());
    assert!(result.is_ok());
}

#[test]
fn tdx_report_data_not_base64_encoded() {
    let result = get_tdx_quote("123456781234567812345678123456781234567812345678".to_string());
    assert!(result.is_err());
}
