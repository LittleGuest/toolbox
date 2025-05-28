use cores::Result;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use tauri::{AppHandle, State};

mod cores;

// #[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
// #[serde(rename_all = "camelCase")]
// pub enum Driver {
//     Mysql,
//     Postgres,
//     Sqlite,
// }
//
// #[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
// #[serde(rename_all = "camelCase")]
// pub struct DatabaseConnect {
//     id: u64,
//     driver: String,
//     name: String,
//     host: String,
//     port: Option<u16>,
//     username: Option<String>,
//     password: Option<String>,
//     database: Option<String>,
// }

// #[tauri::command]
// pub async fn database_connects<'r>(
//     app: AppHandle,
//     db_pool: State<'r, DbPool>,
// ) -> Result<Vec<DatabaseConnect>> {
//     // let store = app.store("store.bin").unwrap();
//     // if let Some(connects) = store.get("database_connects") {
//     //     let mut connects = serde_json::from_value::<Vec<Connect>>(connects).unwrap();
//     //     connects
//     // } else {
//     //     vec![]
//     // }
//     // let res = connects(&*db_pool.0.lock().unwrap()).await;
//     todo!()
// }
//
// #[tauri::command]
// pub fn add_database_connect(connect: DatabaseConnect, app: AppHandle) {
//     // let store = app.store("store.bin").unwrap();
//     // if let Some(connects) = store.get("database_connects") {
//     //     let mut connects = serde_json::from_value::<Vec<Connect>>(connects).unwrap();
//     //     let con = connects.iter().enumerate().find(|(index, con)| {
//     //         con.driver == connect.driver
//     //             && con.host == connect.host
//     //             && con.port == connect.port
//     //             && con.user == connect.user
//     //     });
//     //     println!("--------------------{:?}==={:?}", con, connect);
//     //     if let Some((index, con)) = con {
//     //         println!("====================");
//     //         connects.remove(index);
//     //     }
//     //     connects.push(connect);
//     // } else {
//     //     let mut connects = Vec::new();
//     //     connects.push(connect);
//     //     store.set("database_connects", serde_json::to_value(connects).unwrap());
//     // }
//     todo!()
// }
//
// /*
// 连接信息保存到sqlite中？
//
// 1、新增连接信息
// 2、修改连接信息
// 3、删除连接信息
// 4、查询连接信息
// 5、获取表结构树
//  */
