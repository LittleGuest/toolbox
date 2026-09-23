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

const loadDatabase = async () => {
	return await Database.load("sqlite:toolbox.db");
};

const datasourceInfosApi = async () => {
	const db = await loadDatabase();
        const rows = await db.select<DatasourceInfo[]>("select * from datasource_info");
        return rows.map(normalizeDatasourceInfo);
};

const datasourceDetailApi = async (id: number) => {
	const db = await loadDatabase();
        const rows = await db.select<DatasourceInfo[]>("select * from datasource_info where id=$1", [id]);
        return rows.map(normalizeDatasourceInfo);
};

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
