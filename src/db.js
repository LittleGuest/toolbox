import Database from "@tauri-apps/plugin-sql";

const loadDatabase = async () => {
	return await Database.load("sqlite:toolbox.db");
};

const datasourceInfosApi = async () => {
	const db = await loadDatabase();
	return await db.select("select * from datasource_info");
};

const saveDatasourceInfoApi = async (connect) => {
	const db = await loadDatabase();
	await db.execute(
		"insert into datasource_info (driver, name, host, port, database, username, password) VALUES ($1, $2, $3, $4, $5, $6, $7)",
		[
			connect.driver,
			connect.name,
			connect.host,
			connect.port,
			connect.database,
			connect.username,
			connect.password,
		],
	);
};

const updateDatasourceInfoApi = async (connect) => {
	const db = await loadDatabase();
	await db.execute(
		"update datasource_info set driver=$1, name=$2, host=$3, port=$4, database=$5, username=$6, password=$7 where id=$8",
		[
			connect.driver,
			connect.name,
			connect.host,
			connect.port,
			connect.database,
			connect.username,
			connect.password,
			connect.id,
		],
	);
};

const deleteDatasourceInfoApi = async (id) => {
	const db = await loadDatabase();
	await db.execute("delete from datasource_info where id=$1", [id]);
};

export {
	datasourceInfosApi,
	saveDatasourceInfoApi,
	updateDatasourceInfoApi,
	deleteDatasourceInfoApi,
};
