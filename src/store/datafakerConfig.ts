import Database from "@tauri-apps/plugin-sql";

export type DatafakerConfig = {
  datasourceId: number;
  rowCount: number;
  nodes: unknown[];
  edges: unknown[];
};

const loadDatabase = async () => {
  return await Database.load("sqlite:toolbox.db");
};

const ensureTable = async () => {
  const db = await loadDatabase();
  await db.execute(`
    CREATE TABLE IF NOT EXISTS datafaker_config (
      datasource_id INTEGER PRIMARY KEY,
      row_count INTEGER NOT NULL,
      config_json TEXT NOT NULL,
      updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
    )
  `);
  return db;
};

export const saveDatafakerConfigApi = async (config: DatafakerConfig) => {
  const db = await ensureTable();
  await db.execute(
    `
    INSERT INTO datafaker_config (datasource_id, row_count, config_json, updated_at)
    VALUES ($1, $2, $3, CURRENT_TIMESTAMP)
    ON CONFLICT(datasource_id) DO UPDATE SET
      row_count = excluded.row_count,
      config_json = excluded.config_json,
      updated_at = CURRENT_TIMESTAMP
    `,
    [config.datasourceId, config.rowCount, JSON.stringify(config)]
  );
};

export const loadDatafakerConfigApi = async (
  datasourceId: number
): Promise<DatafakerConfig | null> => {
  const db = await ensureTable();
  const rows = await db.select<{ config_json: string }[]>(
    "SELECT config_json FROM datafaker_config WHERE datasource_id = $1",
    [datasourceId]
  );
  if (!rows.length) {
    return null;
  }
  return JSON.parse(rows[0].config_json);
};
