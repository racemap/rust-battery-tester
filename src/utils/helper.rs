use core::num::ParseIntError;
use std::string::ParseError;

pub fn read_request(request: &str, key: &str) -> Result<String, &'static str> {
    let pairs = request.split("&");

    for pair in pairs {
        let mut pair = pair.split("=");
        if let (Some(k), Some(v)) = (pair.next(), pair.next()) {
            if k == key {
                return Ok(v.to_owned());
            }
        }
    }
    return Err("did not find key");
}

pub fn parse_v(v_str: &mut str) -> Result<u16, ParseIntError> {
    let removed = v_str.replace(".", "");
    removed.parse::<u16>()
}
