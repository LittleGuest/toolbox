import Database from "@tauri-apps/plugin-sql";

type DatasourceInfo = {
     id?: number | null;
     driver?: string | null;
     name?: string | null;
     host?: string | null;
     port?: number | null;
     database?: string | null;
     username?: string | null;
     password?: string | null;
};

const normalizeDriver = (driver?: string | null) => {
     if (driver === "postgresql") {
          return "postgres";
     }
     return driver;
};

const normalizeDatasourceInfo = (connect: DatasourceInfo): DatasourceInfo => ({
     ...connect,
     driver: normalizeDriver(connect.driver),
});

// 加载数据库
const loadDatabase = async () => {
	return await Database.load("sqlite:toolbox.db");
};

// 获取数据源信息
const datasourceInfosApi = async () => {
	const db = await loadDatabase();
        const rows = await db.select<DatasourceInfo[]>("select * from datasource_info");
        return rows.map(normalizeDatasourceInfo);
};

// 数据源详情
const datasourceDetailApi = async (id: number) => {
	const db = await loadDatabase();
        const rows = await db.select<DatasourceInfo[]>("select * from datasource_info where id=$1", [id]);
        return rows.map(normalizeDatasourceInfo);
};

// 新增数据源
const saveDatasourceInfoApi = async (connect: DatasourceInfo) => {
	const db = await loadDatabase();
        const datasourceInfo = normalizeDatasourceInfo(connect);
	await db.execute(
		"insert into datasource_info (driver, name, host, port, database, username, password) VALUES ($1, $2, $3, $4, $5, $6, $7)",
		[
                        datasourceInfo.driver,
                        datasourceInfo.name,
                        datasourceInfo.host,
                        datasourceInfo.port,
                        datasourceInfo.database,
                        datasourceInfo.username,
                        datasourceInfo.password,
		],
	);
};

// 更新数据源
const updateDatasourceInfoApi = async (connect: DatasourceInfo) => {
	const db = await loadDatabase();
        const datasourceInfo = normalizeDatasourceInfo(connect);
	await db.execute(
		"update datasource_info set driver=$1, name=$2, host=$3, port=$4, database=$5, username=$6, password=$7 where id=$8",
		[
                        datasourceInfo.driver,
                        datasourceInfo.name,
                        datasourceInfo.host,
                        datasourceInfo.port,
                        datasourceInfo.database,
                        datasourceInfo.username,
                        datasourceInfo.password,
                        datasourceInfo.id,
		],
	);
};

// 删除数据源
const deleteDatasourceInfoApi = async (id: number) => {
	const db = await loadDatabase();
	await db.execute("delete from datasource_info where id=$1", [id]);
};


export {
	datasourceInfosApi,
	datasourceDetailApi,
	saveDatasourceInfoApi,
	updateDatasourceInfoApi,
	deleteDatasourceInfoApi,
};
