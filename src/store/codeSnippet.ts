import Database from "@tauri-apps/plugin-sql";

const loadDatabase = async () => {
  return await Database.load("sqlite:toolbox.db");
};
const db = await loadDatabase();

const fetchCodeSnippetsApi = async () => {
  return await db.select("select * from code_snippet order by ctime desc");
};

const codeSnippetDetailApi = async (id) => {
  return await db.select("select * from code_snippet where id=$1", [id]);
};

const saveCodeSnippetApi = async (snippet) => {
  await db.execute(
    "insert into code_snippet (language, title, tags, code) VALUES ($1, $2, $3, $4)",
    [snippet.language, snippet.title, snippet.tags, snippet.code]
  );
};

const updateCodeSnippetApi = async (snippet) => {
  await db.execute(
    "update code_snippet set language=$1, title=$2, tags=$3, code=$4, utime=(strftime('%s','now')) where id=$5",
    [snippet.language, snippet.title, snippet.tags, snippet.code, snippet.id]
  );
};

const deleteCodeSnippetApi = async (id) => {
  await db.execute("delete from code_snippet where id=$1", [id]);
};

const fetchTagsApi = async () => {
  return await db.select("select distinct tags from code_snippet");
};

export {
  fetchCodeSnippetsApi,
  codeSnippetDetailApi,
  saveCodeSnippetApi,
  updateCodeSnippetApi,
  deleteCodeSnippetApi,
  fetchTagsApi
};
