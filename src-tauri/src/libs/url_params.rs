//! url params转json

use std::collections::HashMap;

use super::{ToolError, ToolResult};

/// url_params to json
pub fn parse(data: &str) -> ToolResult<String> {
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
            // if let Some(vv) = map.get_mut(k) {
            //     vv.push(v);
            // } else {
            //     map.insert(k, vec![v]);
            // }
            map.insert(k, v);
            map
        });

    serde_json::to_string(&map).map_err(|e| ToolError::UrlParamsErr(e.to_string()))
}
