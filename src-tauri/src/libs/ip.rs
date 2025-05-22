//! ipv4地址和十进制地址互相转换
//! 根据IP地址和掩码计算地址范围

use std::{
    net::{Ipv4Addr, Ipv6Addr},
    str::FromStr,
};

use crate::{Error, Result};

/// Ipv4地址转数字
pub fn ipv4_to_num(ip: &str) -> Result<u32> {
    let ip = Ipv4Addr::from_str(ip).map_err(|e| Error::IpErr(e.to_string()))?;
    let ip = ip
        .octets()
        .iter()
        .map(|&ip| ip as u32)
        .collect::<Vec<u32>>();
    Ok(ip[0] << 24 | ip[1] << 16 | ip[2] << 8 | ip[3])
}

/// 数字转Ipv4
pub fn num_to_ipv4(ip: u32) -> String {
    let ip = Ipv4Addr::from(ip);
    ip.to_string()
}

/// Ipv6地址转数字
pub fn ipv6_to_num(ip: &str) -> Result<u128> {
    let _ip = Ipv6Addr::from_str(ip).map_err(|e| Error::IpErr(e.to_string()))?;
    todo!()
}

/// 数字转Ipv6
pub fn num_to_ipv6(ip: u128) -> String {
    let ip = Ipv6Addr::from(ip);
    ip.to_string()
}

/// Ipv4转Ipv6
pub fn ipv4_ipv6(ip: &str) -> String {
    todo!()
}

/// Ipv6转Ipv4
pub fn ipv6_ipv4(ip: &str) -> String {
    todo!()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_ipv4_num() {
        assert_eq!(3232235521, ipv4_to_num("192.168.0.1").unwrap());
        assert_eq!(num_to_ipv4(3232235521), "192.168.0.1".to_string());
    }

    #[test]
    fn test_ipv6_num() {
        // assert_eq!(
        //     42541956123769884636017138956568135816,
        //     ipv6_to_num("2001:4860:4860:0000:0000:0000:0000:8888").unwrap()
        // );
        // assert_eq!(
        //     num_to_ipv6(42541956123769884636017138956568135816),
        //     "2001:4860:4860:0000:0000:0000:0000:8888".to_string()
        // );
    }
}
