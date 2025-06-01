use std::{
    net::{Ipv4Addr, Ipv6Addr},
    str::FromStr,
};

use crate::{Error, Result};

pub fn ipv4_to_num(ip: &str) -> Result<u32> {
    let ip = Ipv4Addr::from_str(ip).map_err(|e| Error::E(e.to_string()))?;
    let ip = ip
        .octets()
        .iter()
        .map(|&ip| ip as u32)
        .collect::<Vec<u32>>();
    Ok(ip[0] << 24 | ip[1] << 16 | ip[2] << 8 | ip[3])
}

pub fn num_to_ipv4(ip: u32) -> Result<String> {
    let ip = Ipv4Addr::from(ip);
    Ok(ip.to_string())
}

pub fn ipv6_to_num(ip: &str) -> Result<u128> {
    let _ip = Ipv6Addr::from_str(ip).map_err(|e| Error::E(e.to_string()))?;
    todo!()
}

pub fn num_to_ipv6(ip: u128) -> Result<String> {
    let ip = Ipv6Addr::from(ip);
    Ok(ip.to_string())
}

pub fn ipv4_ipv6(ip: &str) -> Result<String> {
    todo!()
}

pub fn ipv6_ipv4(ip: &str) -> Result<String> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipv4_num() -> Result<()> {
        assert_eq!(3232235521, ipv4_to_num("192.168.0.1")?);
        assert_eq!(num_to_ipv4(3232235521)?, "192.168.0.1".to_string());
        Ok(())
    }

    #[test]
    fn test_ipv6_num() -> Result<()> {
        // assert_eq!(
        //     42541956123769884636017138956568135816,
        //     ipv6_to_num("2001:4860:4860:0000:0000:0000:0000:8888").unwrap()
        // );
        // assert_eq!(
        //     num_to_ipv6(42541956123769884636017138956568135816),
        //     "2001:4860:4860:0000:0000:0000:0000:8888".to_string()
        // );
        Ok(())
    }
}
