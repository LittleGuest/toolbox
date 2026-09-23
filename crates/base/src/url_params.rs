use std::collections::HashMap;

use anyhow::{Error, Result};

pub fn parse(data: &str) -> Result<String> {
    if data.trim().is_empty() {
        return Ok(String::new());
    }

    let map = data
        .split('&')
        .map(|d| {
            let mut key = "";
            let mut value = "";

            let mut p = d.split('=');
            if let Some(k) = p.next() {
                key = k;
            }

            if let Some(v) = p.next() {
                value = v;
            }
            (key, value)
        })
        .fold(HashMap::<_, _>::new(), |mut map, (k, v)| {
            map.insert(k, v);
            map
        });

    serde_json::to_string(&map).map_err(|e| Error::msg(e.to_string()))
}
