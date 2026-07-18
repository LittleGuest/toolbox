use database::{DatasourceInfo, Driver};
use gpui::{prelude::FluentBuilder as _, *};
use gpui_component::{
    WindowExt,
    button::*,
    checkbox::Checkbox,
    input::{Input, InputState, NumberInput},
    scroll::ScrollableElement,
    select::{Select, SelectEvent, SelectGroup, SelectItem, SelectState, SearchableVec},
    *,
};
use rand::Rng;
use sqlx::AnyPool;

use crate::config_store;

const INSERT_BATCH_SIZE: usize = 200;
const TABLE_NODE_WIDTH: f32 = 200.0;
const TABLE_NODE_HEIGHT: f32 = 48.0;
const COLUMN_NODE_WIDTH: f32 = 200.0;
const GENERATOR_NODE_WIDTH: f32 = 120.0;
const FIELD_NODE_HEIGHT: f32 = 48.0;
const GENERATOR_OFFSET_X: f32 = 340.0;

#[derive(Clone, Copy, PartialEq)]
enum FakerView {
    ConnectionList,
    Generator,
}

pub struct FakeDataGenerator {
    // 视图状态
    active_view: FakerView,

    // 连接列表页相关
    conn_form: DatafakerDbForm,
    conn_driver: String,
    conn_driver_state: Entity<SelectState<Vec<String>>>,
    conn_edit_name: Option<String>,

    // 画布配置页相关
    current_datasource: Option<DatasourceInfo>,
    all_tables: Vec<TablePreview>,
    table_filter: String,
    run_logs: Vec<String>,
    show_run_log: bool,
    selected_column_name: Option<String>,
    generator_select_state: Entity<SelectState<SearchableVec<SelectGroup<GeneratorSelectItem>>>>,
    // 生成器配置抽屉状态（对齐 Tauri SimplePreview/Number/Date 等配置组件的通用字段）
    gen_locale_state: Entity<SelectState<Vec<String>>>,
    gen_default_value_state: Entity<InputState>,
    gen_default_percentage_state: Entity<InputState>,
    gen_null_percentage_state: Entity<InputState>,
    gen_preview_state: Entity<InputState>,
    gen_include_default: bool,
    gen_include_null: bool,
    gen_unique: bool,
    gen_forbidden_links: bool,
    // 生成器专属配置（Number: start/end, Date: startDate/endDate, Text: minLength/maxLength, Regex: pattern, Enum: values, Sequence: start/step）
    gen_number_start_state: Entity<InputState>,
    gen_number_end_state: Entity<InputState>,
    gen_date_start_state: Entity<InputState>,
    gen_date_end_state: Entity<InputState>,
    gen_text_min_len_state: Entity<InputState>,
    gen_text_max_len_state: Entity<InputState>,
    gen_regex_pattern_state: Entity<InputState>,
    gen_enum_values_state: Entity<InputState>,
    gen_seq_start_state: Entity<InputState>,
    gen_seq_step_state: Entity<InputState>,
    // 生成器配置抽屉是否打开（改用 render 内条件渲染，cx.notify() 可触发重渲染）
    show_gen_dialog: bool,

    // 共享状态
    driver: String,
    form: DatafakerDbForm,
    row_count_state: Entity<InputState>,
    table_filter_state: Entity<InputState>,
    driver_state: Entity<SelectState<Vec<String>>>,
    saved_datasources: Vec<DatasourceInfo>,
    tables: Vec<TablePreview>,
    selected_table: Option<String>,
    selected_canvas_node: Option<usize>,
    output: String,
    status: String,
    is_running: bool,
    canvas_nodes: Vec<CanvasNode>,
    canvas_viewport_x: f32,
    canvas_viewport_y: f32,
    canvas_zoom: f32,
    canvas_drag: Option<CanvasDrag>,
    canvas_name_state: Entity<InputState>,
    _subscriptions: Vec<Subscription>,
}

#[derive(Clone)]
struct CanvasNode {
    table: TablePreview,
    x: f32,
    y: f32,
}

enum CanvasDrag {
    Node {
        index: usize,
        offset_x: f32,
        offset_y: f32,
    },
    Pan {
        start_mouse_x: f32,
        start_mouse_y: f32,
        origin_vx: f32,
        origin_vy: f32,
    },
}

#[derive(Clone)]
struct TablePreview {
    schema: String,
    table_name: String,
    table_comment: String,
    columns: Vec<ColumnPreview>,
}

#[derive(Clone, Default)]
struct GeneratorConfig {
    locale: String,
    include_default: bool,
    default_value: String,
    default_percentage: f64,
    include_null: bool,
    null_percentage: f64,
    unique: bool,
    forbidden_links: bool,
    // 专属配置字段（按生成器类型使用）
    number_start: f64,
    number_end: f64,
    date_start: String,
    date_end: String,
    text_min_length: usize,
    text_max_length: usize,
    regex_pattern: String,
    enum_values: String,  // 逗号分隔
    seq_start: i64,
    seq_step: i64,
}

#[derive(Clone)]
struct ColumnPreview {
    name: String,
    column_type: String,
    comment: String,
    generator: String,
    config: GeneratorConfig,
}

struct DatafakerDbForm {
    name: Entity<InputState>,
    host: Entity<InputState>,
    port: Entity<InputState>,
    database: Entity<InputState>,
    username: Entity<InputState>,
    password: Entity<InputState>,
}

impl FakeDataGenerator {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let driver_items = vec![
            "PostgreSQL".to_string(),
            "MySQL".to_string(),
            "SQLite".to_string(),
        ];
        let driver_state = cx.new(|cx| {
            let mut state = SelectState::new(driver_items.clone(), None, window, cx);
            state.set_selected_value(&"PostgreSQL".to_string(), window, cx);
            state
        });
        let conn_driver_state = cx.new(|cx| {
            let mut state = SelectState::new(driver_items, None, window, cx);
            state.set_selected_value(&"PostgreSQL".to_string(), window, cx);
            state
        });
        let count_state = cx.new(|cx| InputState::new(window, cx).default_value("10"));
        let table_filter_state = cx.new(|cx| InputState::new(window, cx).placeholder("搜索表..."));
        let canvas_name_state =
            cx.new(|cx| InputState::new(window, cx).placeholder("画布配置名称"));
        let generator_select_items = build_generator_select_items();
        let generator_select_state = cx.new(|cx| {
            SelectState::new(generator_select_items, None, window, cx).searchable(true)
        });
        // 生成器配置抽屉的语言下拉（对齐 Tauri SimplePreview 的 localeOptions）
        let locale_items = vec![
            "简体中文".to_string(),
            "繁体中文".to_string(),
            "中文拼音".to_string(),
            "English".to_string(),
        ];
        let gen_locale_state = cx.new(|cx| {
            let mut state = SelectState::new(locale_items, None, window, cx);
            state.set_selected_value(&"简体中文".to_string(), window, cx);
            state
        });
        let gen_default_value_state =
            cx.new(|cx| InputState::new(window, cx).placeholder("请输入默认值"));
        let gen_default_percentage_state =
            cx.new(|cx| InputState::new(window, cx).default_value("5"));
        let gen_null_percentage_state =
            cx.new(|cx| InputState::new(window, cx).default_value("5"));
        let gen_preview_state = cx.new(|cx| InputState::new(window, cx).placeholder("预览值"));
        // 生成器专属配置输入状态
        let gen_number_start_state = cx.new(|cx| InputState::new(window, cx).default_value("0"));
        let gen_number_end_state = cx.new(|cx| InputState::new(window, cx).default_value("1000"));
        let gen_date_start_state = cx.new(|cx| InputState::new(window, cx).default_value("2000-01-01"));
        let gen_date_end_state = cx.new(|cx| InputState::new(window, cx).default_value("2026-01-01"));
        let gen_text_min_len_state = cx.new(|cx| InputState::new(window, cx).default_value("100"));
        let gen_text_max_len_state = cx.new(|cx| InputState::new(window, cx).default_value("10000"));
        let gen_regex_pattern_state = cx.new(|cx| InputState::new(window, cx).placeholder("正则表达式"));
        let gen_enum_values_state = cx.new(|cx| InputState::new(window, cx).placeholder("枚举值（逗号分隔）"));
        let gen_seq_start_state = cx.new(|cx| InputState::new(window, cx).default_value("1"));
        let gen_seq_step_state = cx.new(|cx| InputState::new(window, cx).default_value("1"));
        let form = DatafakerDbForm::new(window, cx);
        let conn_form = DatafakerDbForm::new(window, cx);

        let _subscriptions = vec![
            cx.subscribe_in(
                &driver_state,
                window,
                move |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                    if let SelectEvent::Confirm(Some(value)) = ev {
                        this.driver = value.clone();
                        cx.notify();
                    }
                },
            ),
            cx.subscribe_in(
                &conn_driver_state,
                window,
                move |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                    if let SelectEvent::Confirm(Some(value)) = ev {
                        this.conn_driver = value.clone();
                        cx.notify();
                    }
                },
            ),
            // 抽屉内切换生成器：触发面板重渲染以更新条件渲染（对齐 Tauri watch(datafakerValue, hydrateConfig)）
            cx.subscribe_in(
                &generator_select_state,
                window,
                move |this, _, _ev: &SelectEvent<SearchableVec<SelectGroup<GeneratorSelectItem>>>, window, cx| {
                    // 切换生成器后清空预览值
                    this.gen_preview_state.update(cx, |state, cx| {
                        state.set_value(String::new(), window, cx);
                    });
                    // 触发整个视图重渲染，使 gen_dialog_content 中的条件渲染重新求值
                    cx.notify();
                },
            ),
        ];

        let mut this = Self {
            active_view: FakerView::ConnectionList,
            conn_form,
            conn_driver: "PostgreSQL".to_string(),
            conn_driver_state,
            conn_edit_name: None,
            current_datasource: None,
            all_tables: Vec::new(),
            table_filter: String::new(),
            run_logs: Vec::new(),
            show_run_log: false,
            selected_column_name: None,
            generator_select_state,
            gen_locale_state,
            gen_default_value_state,
            gen_default_percentage_state,
            gen_null_percentage_state,
            gen_include_default: false,
            gen_include_null: false,
            gen_unique: false,
            gen_forbidden_links: false,
            gen_preview_state,
            gen_number_start_state,
            gen_number_end_state,
            gen_date_start_state,
            gen_date_end_state,
            gen_text_min_len_state,
            gen_text_max_len_state,
            gen_regex_pattern_state,
            gen_enum_values_state,
            gen_seq_start_state,
            gen_seq_step_state,
            show_gen_dialog: false,
            driver: "PostgreSQL".to_string(),
            form,
            row_count_state: count_state,
            table_filter_state,
            driver_state,
            saved_datasources: Vec::new(),
            tables: Vec::new(),
            selected_table: None,
            selected_canvas_node: None,
            output: String::new(),
            status: "请填写数据源并加载表结构。".to_string(),
            is_running: false,
            canvas_nodes: Vec::new(),
            canvas_viewport_x: 0.0,
            canvas_viewport_y: 0.0,
            canvas_zoom: 1.0,
            canvas_drag: None,
            canvas_name_state,
            _subscriptions,
        };

        // 初始化时从 SQLite 加载已保存连接列表
        this.refresh_datasources(cx);

        this
    }

    // ── 视图切换 ──

    fn go_to_generator(&mut self, name: String, window: &mut Window, cx: &mut Context<Self>) {
        let Some(ds) = self
            .saved_datasources
            .iter()
            .find(|d| d.name == name)
            .cloned()
        else {
            self.status = format!("连接 {name} 不存在。");
            cx.notify();
            return;
        };

        self.current_datasource = Some(ds.clone());
        self.active_view = FakerView::Generator;
        self.all_tables.clear();
        self.tables.clear();
        self.canvas_nodes.clear();
        self.canvas_viewport_x = 0.0;
        self.canvas_viewport_y = 0.0;
        self.canvas_zoom = 1.0;
        self.canvas_drag = None;
        self.run_logs.clear();
        self.show_run_log = false;
        self.table_filter.clear();
        self.table_filter_state.update(cx, |state, cx| {
            state.set_value(String::new(), window, cx);
        });
        self.status = format!("正在加载 {} 的表结构...", ds.name);
        cx.notify();

        // 加载数据源的表列表并匹配生成器
        self.load_datasource_tables(cx);

        // 自动加载已保存的画布配置（对齐 Tauri 的 init→loadConfig 行为）
        self.load_canvas(window, cx);
    }

    fn go_back(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.active_view = FakerView::ConnectionList;
        self.current_datasource = None;
        self.table_filter.clear();
        self.table_filter_state.update(cx, |state, cx| {
            state.set_value(String::new(), window, cx);
        });
        self.refresh_datasources(cx);
    }

    fn load_datasource_tables(&mut self, cx: &mut Context<Self>) {
        let Some(info) = self.current_datasource.clone() else {
            self.status = "请先选择数据源。".to_string();
            cx.notify();
            return;
        };
        self.is_running = true;
        self.status = "正在加载表结构并匹配生成器...".to_string();
        cx.notify();

        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            let result = load_table_previews(info).await;
            let _ = this.update(cx, |this, cx| {
                this.is_running = false;
                match result {
                    Ok(tables) => {
                        this.all_tables = tables.clone();
                        this.tables = tables.clone();
                        this.selected_table = tables
                            .first()
                            .map(|table| table_key(&table.schema, &table.table_name));
                        this.status = format!("已加载 {} 张表", this.tables.len());
                    }
                    Err(err) => {
                        this.status = format!("加载表结构失败：{err}");
                        this.tables.clear();
                        this.all_tables.clear();
                        this.selected_table = None;
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    // ── 连接列表页方法 ──

    fn open_conn_sheet(
        &mut self,
        edit_name: Option<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.conn_edit_name = edit_name.clone();

        if let Some(ref name) = edit_name {
            // 编辑模式：从已保存列表加载数据
            if let Some(ds) = self.saved_datasources.iter().find(|d| d.name == *name) {
                self.conn_driver = config_store::driver_label(&ds.driver);
                self.conn_driver_state.update(cx, |state, cx| {
                    state.set_selected_value(&self.conn_driver.clone(), window, cx);
                });
                self.conn_form.apply(ds, window, cx);
            }
        } else {
            // 新建模式：清空表单
            self.conn_driver = "PostgreSQL".to_string();
            self.conn_driver_state.update(cx, |state, cx| {
                state.set_selected_value(&"PostgreSQL".to_string(), window, cx);
            });
            self.conn_form.apply(
                &DatasourceInfo {
                    driver: Driver::Postgres,
                    name: String::new(),
                    host: String::new(),
                    port: None,
                    username: None,
                    password: None,
                    database: None,
                },
                window,
                cx,
            );
        }

        let weak = cx.entity().downgrade();
        let is_edit = edit_name.is_some();
        let driver_state = self.conn_driver_state.clone();
        let form_name = self.conn_form.name.clone();
        let form_host = self.conn_form.host.clone();
        let form_port = self.conn_form.port.clone();
        let form_database = self.conn_form.database.clone();
        let form_username = self.conn_form.username.clone();
        let form_password = self.conn_form.password.clone();

        window.open_sheet_at(Placement::Bottom, cx, move |sheet, _, cx| {
            sheet
                .overlay(true)
                .overlay_closable(true)
                .size(px(500.))
                .title(if is_edit {
                    "编辑"
                } else {
                    "添加"
                })
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .p_4()
                        .child(form_field("类型", Select::new(&driver_state)))
                        .child(form_field("连接名称", Input::new(&form_name)))
                        .child(form_field("主机", Input::new(&form_host)))
                        .child(form_field("端口", Input::new(&form_port)))
                        .child(form_field("数据库", Input::new(&form_database)))
                        .child(form_field("用户名", Input::new(&form_username)))
                        .child(form_field("密码", Input::new(&form_password).mask_toggle()))
                        .child(
                            div()
                                .flex()
                                .justify_end()
                                .gap_2()
                                .mt_2()
                                .child({
                                    let weak = weak.clone();
                                    Button::new("conn-sheet-ping")
                                        .label("测试连接")
                                        .on_click(move |_, _, cx| {
                                            if let Some(this) = weak.upgrade() {
                                                this.update(cx, |this, cx| {
                                                    this.ping_conn(cx);
                                                });
                                            }
                                        })
                                })
                                .child({
                                    let weak = weak.clone();
                                    Button::new("conn-sheet-save")
                                        .label("保存")
                                        .on_click(move |_, window, cx| {
                                            if let Some(this) = weak.upgrade() {
                                                this.update(cx, |this, cx| {
                                                    this.save_conn(window, cx);
                                                    window.close_sheet(cx);
                                                });
                                            }
                                        })
                                }),
                        ),
                )
        });
    }

    fn save_conn(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let info = self.conn_form.to_info(&self.conn_driver, cx);
        let Ok(info) = info else {
            self.status = info.unwrap_err();
            cx.notify();
            return;
        };

        let is_edit = self.conn_edit_name.is_some();
        let saved_info = info.clone();
        self.status = if is_edit {
            "正在更新连接...".to_string()
        } else {
            "正在保存连接...".to_string()
        };
        cx.notify();

        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            let result = config_store::save_datasource(info).await;
            let _ = this.update(cx, |this, cx| {
                this.status = match result {
                    Ok(_) => {
                        upsert_datasource(&mut this.saved_datasources, saved_info);
                        if is_edit {
                            "连接已更新".to_string()
                        } else {
                            "连接已保存".to_string()
                        }
                    }
                    Err(err) => format!("保存连接失败：{err}"),
                };
                cx.notify();
            });
        })
        .detach();

        self.conn_edit_name = None;
    }

    fn ping_conn(&mut self, cx: &mut Context<Self>) {
        let info = self.conn_form.to_info(&self.conn_driver, cx);
        let Ok(info) = info else {
            self.status = info.unwrap_err();
            cx.notify();
            return;
        };

        self.status = format!("正在测试连接 {}...", info.name);
        cx.notify();

        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            let result = database::database_ping(info.clone()).await;
            let _ = this.update(cx, |this, cx| {
                this.status = match result {
                    Ok(_) => format!("连接 {} 成功", info.name),
                    Err(err) => format!("连接失败：{err}"),
                };
                cx.notify();
            });
        })
        .detach();
    }

    fn delete_datasource_by_name(&mut self, name: String, cx: &mut Context<Self>) {
        self.status = format!("正在删除连接 {name}...");
        cx.notify();

        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            let result = config_store::delete_datasource(name.clone()).await;
            let _ = this.update(cx, |this, cx| {
                this.status = match result {
                    Ok(true) => {
                        this.saved_datasources.retain(|item| item.name != name);
                        format!("连接 {name} 已删除。")
                    }
                    Ok(false) => format!("未找到连接 {name}。"),
                    Err(err) => format!("删除连接失败：{err}"),
                };
                cx.notify();
            });
        })
        .detach();
    }

    // ── 画布配置页方法 ──

    fn add_table_to_canvas(&mut self, key: String, cx: &mut Context<Self>) {
        if self
            .canvas_nodes
            .iter()
            .any(|node| table_key(&node.table.schema, &node.table.table_name) == key)
        {
            self.status = "该表已在画布中。".to_string();
            cx.notify();
            return;
        }
        let Some(table) = self
            .tables
            .iter()
            .find(|table| table_key(&table.schema, &table.table_name) == key)
            .cloned()
        else {
            self.status = "表不存在，请重新加载表结构。".to_string();
            cx.notify();
            return;
        };
        let count = self.canvas_nodes.len() as f32;
        let x = 40.0 + (count % 3.0) * (TABLE_NODE_WIDTH + 200.0);
        let y = 40.0 + (count / 3.0).floor() * 200.0;
        self.canvas_nodes.push(CanvasNode { table, x, y });
        self.status = format!(
            "已添加表到画布，当前画布有 {} 张表。",
            self.canvas_nodes.len()
        );
        cx.notify();
    }

    fn run_canvas_config(&mut self, cx: &mut Context<Self>) {
        let Some(info) = self.current_datasource.clone() else {
            self.status = "请先选择数据源。".to_string();
            cx.notify();
            return;
        };
        if self.canvas_nodes.is_empty() {
            self.status = "画布为空，请先添加表到画布。".to_string();
            cx.notify();
            return;
        }
        let count = self
            .row_count_state
            .read(cx)
            .value()
            .parse::<usize>()
            .unwrap_or(10)
            .clamp(1, 100_000);
        let tables = self
            .canvas_nodes
            .iter()
            .map(|node| node.table.clone())
            .collect::<Vec<_>>();

        self.is_running = true;
        self.run_logs.clear();
        self.show_run_log = true;
        self.add_run_log("开始运行配置...".to_string(), cx);
        self.add_run_log(format!("表数量={}, 每张表行数={count}", tables.len()), cx);
        self.status = "正在写入假数据...".to_string();
        cx.notify();

        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            // 逐表处理并实时推送日志（对齐 Tauri 的 datafaker-run-log 事件推送）
            let table_count = tables.len();
            for (ti, table) in tables.iter().enumerate() {
                let table_label = format!("{}.{}", table.schema, table.table_name);
                let _ = this.update(cx, |this, cx| {
                    this.add_run_log(format!("[{}/{}] 开始处理表 {}", ti + 1, table_count, table_label), cx);
                    cx.notify();
                });

                let result = run_fake_data_insert_single(&info, table, count).await;
                let _ = this.update(cx, |this, cx| {
                    match result {
                        Ok(logs) => {
                            for line in logs {
                                this.add_run_log(line, cx);
                            }
                        }
                        Err(err) => {
                            this.add_run_log(format!("表 {} 处理失败: {err}", table_label), cx);
                        }
                    }
                    cx.notify();
                });
            }

            let _ = this.update(cx, |this, cx| {
                this.is_running = false;
                this.status = "假数据写入完成".to_string();
                this.add_run_log("所有表处理完成。".to_string(), cx);
                cx.notify();
            });
        })
        .detach();
    }

    fn add_run_log(&mut self, msg: String, cx: &mut Context<Self>) {
        self.run_logs.push(msg);
        cx.notify();
    }

    fn save_generator_config(&mut self, cx: &mut Context<Self>) {
        let name = match &self.current_datasource {
            Some(ds) => ds.name.clone(),
            None => {
                self.status = "请先选择数据源。".to_string();
                cx.notify();
                return;
            }
        };
        let nodes_json = serde_json::to_string(
            &self
                .canvas_nodes
                .iter()
                .map(|node| {
                    serde_json::json!({
                        "schema": node.table.schema,
                        "tableName": node.table.table_name,
                        "tableComment": node.table.table_comment,
                        "x": node.x,
                        "y": node.y,
                        "columns": node.table.columns.iter().map(|col| {
                            serde_json::json!({
                                "name": col.name,
                                "type": col.column_type,
                                "comment": col.comment,
                                "generator": col.generator,
                            })
                        }).collect::<Vec<_>>(),
                    })
                })
                .collect::<Vec<_>>(),
        )
        .unwrap_or_default();
        let edges_json = "[]".to_string();
        let name_clone = name.clone();
        self.status = format!("正在保存画布配置 {name}...");
        cx.notify();

        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            let result =
                config_store::save_datafaker_config(&name_clone, &nodes_json, &edges_json).await;
            let _ = this.update(cx, |this, cx| {
                this.status = match result {
                    Ok(_) => format!("画布配置 {name_clone} 已保存。"),
                    Err(err) => format!("保存画布配置失败：{err}"),
                };
                cx.notify();
            });
        })
        .detach();
    }

    // ── 保留的现有方法 ──

    fn datasource_info(&self, cx: &App) -> Result<DatasourceInfo, String> {
        self.form.to_info(&self.driver, cx)
    }

    fn save_datasource(&mut self, cx: &mut Context<Self>) {
        let Ok(info) = self.datasource_info(cx) else {
            self.status = "请先补全连接信息。".to_string();
            cx.notify();
            return;
        };
        let saved_info = info.clone();
        self.status = "正在保存连接...".to_string();
        cx.notify();

        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            let result = config_store::save_datasource(info).await;
            let _ = this.update(cx, |this, cx| {
                this.status = match result {
                    Ok(_) => {
                        upsert_datasource(&mut this.saved_datasources, saved_info);
                        "连接已保存到 SQLite。".to_string()
                    }
                    Err(err) => format!("保存连接失败：{err}"),
                };
                cx.notify();
            });
        })
        .detach();
    }

    fn refresh_datasources(&mut self, cx: &mut Context<Self>) {
        self.status = "正在刷新连接列表...".to_string();
        cx.notify();

        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            let result = config_store::load_datasources().await;
            let _ = this.update(cx, |this, cx| {
                match result {
                    Ok(datasources) => {
                        this.status = format!("已加载 {} 个已保存连接。", datasources.len());
                        this.saved_datasources = datasources;
                    }
                    Err(err) => {
                        this.status = format!("刷新连接列表失败：{err}");
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn open_datasource_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.status = "正在加载连接弹窗...".to_string();
        cx.notify();

        cx.spawn_in(window, async move |this: WeakEntity<Self>, cx| {
            let result = config_store::load_datasources().await;
            let _ = this.update_in(cx, |this, window, cx| {
                match result {
                    Ok(datasources) => {
                        this.status = format!("已加载 {} 个已保存连接。", datasources.len());
                        this.saved_datasources = datasources.clone();
                        this.show_datasource_dialog(datasources, window, cx);
                    }
                    Err(err) => {
                        this.status = format!("打开连接弹窗失败：{err}");
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn show_datasource_dialog(
        &mut self,
        datasources: Vec<DatasourceInfo>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let this = cx.entity().downgrade();
        window.open_dialog(cx, move |dialog, _, cx| {
            dialog
                .title(div().text_lg().font_semibold().child("选择数据库连接"))
                .width(px(640.))
                .child(datasource_dialog_content(
                    datasources.clone(),
                    this.clone(),
                    cx,
                ))
        });
    }

    fn apply_saved_datasource(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(info) = self.saved_datasources.get(index).cloned() else {
            self.status = "连接不存在，请刷新列表后重试。".to_string();
            cx.notify();
            return;
        };
        let name = info.name.clone();
        self.apply_datasource(info, window, cx);
        self.status = format!("已应用连接 {name}。");
        cx.notify();
    }

    fn delete_datasource(&mut self, cx: &mut Context<Self>) {
        let name = read_input(&self.form.name, cx);
        if name.trim().is_empty() {
            self.status = "请先填写要删除的连接名称。".to_string();
            cx.notify();
            return;
        }
        self.status = format!("正在删除连接 {name}...");
        cx.notify();

        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            let result = config_store::delete_datasource(name.clone()).await;
            let _ = this.update(cx, |this, cx| {
                this.status = match result {
                    Ok(true) => {
                        this.saved_datasources.retain(|item| item.name != name);
                        format!("连接 {name} 已删除。")
                    }
                    Ok(false) => format!("未找到连接 {name}。"),
                    Err(err) => format!("删除连接失败：{err}"),
                };
                cx.notify();
            });
        })
        .detach();
    }

    fn apply_datasource(
        &mut self,
        info: DatasourceInfo,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let driver = config_store::driver_label(&info.driver);
        self.driver = driver.clone();
        self.driver_state.update(cx, |state, cx| {
            state.set_selected_value(&driver, window, cx);
        });
        self.form.apply(&info, window, cx);
    }

    fn ping(&mut self, cx: &mut Context<Self>) {
        let Ok(info) = self.datasource_info(cx) else {
            self.status = "请先补全连接信息。".to_string();
            cx.notify();
            return;
        };
        self.is_running = true;
        self.status = "正在测试连接...".to_string();
        cx.notify();

        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            let result = database::database_ping(info).await;
            let _ = this.update(cx, |this, cx| {
                this.is_running = false;
                this.status = match result {
                    Ok(_) => "连接成功".to_string(),
                    Err(err) => format!("连接失败：{err}"),
                };
                cx.notify();
            });
        })
        .detach();
    }

    fn load_tables(&mut self, cx: &mut Context<Self>) {
        let Ok(info) = self.datasource_info(cx) else {
            self.status = "请先补全连接信息。".to_string();
            cx.notify();
            return;
        };
        self.is_running = true;
        self.status = "正在加载表结构并匹配生成器...".to_string();
        cx.notify();

        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            let result = load_table_previews(info).await;
            let _ = this.update(cx, |this, cx| {
                this.is_running = false;
                match result {
                    Ok(tables) => {
                        this.selected_table = tables
                            .first()
                            .map(|table| table_key(&table.schema, &table.table_name));
                        this.status = format!("已加载 {} 张表", tables.len());
                        this.tables = tables;
                    }
                    Err(err) => {
                        this.status = format!("加载表结构失败：{err}");
                        this.tables.clear();
                        this.selected_table = None;
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn select_table(&mut self, key: String, cx: &mut Context<Self>) {
        self.selected_table = Some(key);
        cx.notify();
    }

    fn open_generator_dialog_for_column(
        &mut self,
        column_name: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(index) = self.selected_canvas_node else {
            self.status = "请先选择表节点。".to_string();
            cx.notify();
            return;
        };
        let Some(node) = self.canvas_nodes.get(index) else {
            self.status = "表节点不存在。".to_string();
            cx.notify();
            return;
        };
        let key = table_key(&node.table.schema, &node.table.table_name);
        self.open_generator_dialog(key, column_name, window, cx);
    }

    fn open_generator_dialog(
        &mut self,
        key: String,
        column_name: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // 优先从 canvas_nodes 查找（保留用户配置的生成器），否则从 tables 查找
        let table = self
            .canvas_nodes
            .iter()
            .find(|node| table_key(&node.table.schema, &node.table.table_name) == key)
            .map(|node| node.table.clone())
            .or_else(|| {
                self.tables
                    .iter()
                    .find(|table| table_key(&table.schema, &table.table_name) == key)
                    .cloned()
            });
        let Some(table) = table else {
            self.status = "表不存在，请重新加载表结构。".to_string();
            cx.notify();
            return;
        };

        // 找到当前列及其生成器
        let column = table
            .columns
            .iter()
            .find(|c| c.name == column_name)
            .cloned();
        let Some(column) = column else {
            self.status = format!("列 {column_name} 不存在。");
            cx.notify();
            return;
        };

        self.selected_table = Some(key.clone());
        self.selected_column_name = Some(column_name.clone());

        // 初始化 Select：选中当前列的生成器
        let current_generator = column.generator.clone();
        self.generator_select_state.update(cx, |state, cx| {
            state.set_selected_value(&current_generator, window, cx);
        });

        // 初始化通用配置（对齐 Tauri SimplePreview 的 setConfig 行为）
        // 注意：仅首次打开时从 column.config 初始化，切换生成器后配置由用户控制
        let cfg = &column.config;
        let locale_label = match cfg.locale.as_str() {
            "zh_traditional" => "繁体中文",
            "zh_pinyin" => "中文拼音",
            "en_us" => "English",
            _ => "简体中文",
        };
        self.gen_locale_state.update(cx, |state, cx| {
            state.set_selected_value(&locale_label.to_string(), window, cx);
        });
        self.gen_default_value_state.update(cx, |state, cx| {
            state.set_value(cfg.default_value.clone(), window, cx);
        });
        self.gen_default_percentage_state.update(cx, |state, cx| {
            state.set_value(
                format!("{}", cfg.default_percentage.round() as i64),
                window,
                cx,
            );
        });
        self.gen_null_percentage_state.update(cx, |state, cx| {
            state.set_value(
                format!("{}", cfg.null_percentage.round() as i64),
                window,
                cx,
            );
        });
        self.gen_preview_state.update(cx, |state, cx| {
            state.set_value(String::new(), window, cx);
        });
        self.gen_include_default = cfg.include_default;
        self.gen_include_null = cfg.include_null;
        self.gen_unique = cfg.unique;
        self.gen_forbidden_links = cfg.forbidden_links;

        // 初始化专属配置字段
        self.gen_number_start_state.update(cx, |state, cx| {
            state.set_value(cfg.number_start.to_string(), window, cx);
        });
        self.gen_number_end_state.update(cx, |state, cx| {
            state.set_value(cfg.number_end.to_string(), window, cx);
        });
        self.gen_date_start_state.update(cx, |state, cx| {
            state.set_value(cfg.date_start.clone(), window, cx);
        });
        self.gen_date_end_state.update(cx, |state, cx| {
            state.set_value(cfg.date_end.clone(), window, cx);
        });
        self.gen_text_min_len_state.update(cx, |state, cx| {
            state.set_value(cfg.text_min_length.to_string(), window, cx);
        });
        self.gen_text_max_len_state.update(cx, |state, cx| {
            state.set_value(cfg.text_max_length.to_string(), window, cx);
        });
        self.gen_regex_pattern_state.update(cx, |state, cx| {
            state.set_value(cfg.regex_pattern.clone(), window, cx);
        });
        self.gen_enum_values_state.update(cx, |state, cx| {
            state.set_value(cfg.enum_values.clone(), window, cx);
        });
        self.gen_seq_start_state.update(cx, |state, cx| {
            state.set_value(cfg.seq_start.to_string(), window, cx);
        });
        self.gen_seq_step_state.update(cx, |state, cx| {
            state.set_value(cfg.seq_step.to_string(), window, cx);
        });

        self.show_gen_dialog = true;
        cx.notify();
    }

    fn close_gen_dialog(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.show_gen_dialog = false;
        cx.notify();
    }

    fn save_gen_dialog(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let key = self.selected_table.clone().unwrap_or_default();
        let column_name = self.selected_column_name.clone().unwrap_or_default();
        let generator = self.generator_select_state.read(cx).selected_value().cloned().unwrap_or_default();
        let locale_raw = self.gen_locale_state.read(cx).selected_value().cloned().unwrap_or_default();
        let locale_code = match locale_raw.as_str() {
            "繁体中文" => "zh_traditional",
            "中文拼音" => "zh_pinyin",
            "English" => "en_us",
            _ => "zh_cn",
        }.to_string();
        let default_value = self.gen_default_value_state.read(cx).value().to_string();
        let default_pct = self.gen_default_percentage_state.read(cx).value().parse::<f64>().unwrap_or(5.0);
        let null_pct = self.gen_null_percentage_state.read(cx).value().parse::<f64>().unwrap_or(5.0);
        let cfg = GeneratorConfig {
            locale: locale_code,
            include_default: self.gen_include_default,
            default_value: default_value.clone(),
            default_percentage: default_pct,
            include_null: self.gen_include_null,
            null_percentage: null_pct,
            unique: self.gen_unique,
            forbidden_links: self.gen_forbidden_links,
            number_start: self.gen_number_start_state.read(cx).value().parse().unwrap_or(0.0),
            number_end: self.gen_number_end_state.read(cx).value().parse().unwrap_or(1000.0),
            date_start: self.gen_date_start_state.read(cx).value().to_string(),
            date_end: self.gen_date_end_state.read(cx).value().to_string(),
            text_min_length: self.gen_text_min_len_state.read(cx).value().parse().unwrap_or(100),
            text_max_length: self.gen_text_max_len_state.read(cx).value().parse().unwrap_or(10000),
            regex_pattern: self.gen_regex_pattern_state.read(cx).value().to_string(),
            enum_values: self.gen_enum_values_state.read(cx).value().to_string(),
            seq_start: self.gen_seq_start_state.read(cx).value().parse().unwrap_or(1),
            seq_step: self.gen_seq_step_state.read(cx).value().parse().unwrap_or(1),
        };
        if !key.is_empty() && !column_name.is_empty() && !generator.is_empty() {
            self.set_column_generator_with_config(key, column_name, generator, cfg, cx);
        }
        self.show_gen_dialog = false;
        cx.notify();
    }

    fn gen_dialog_content(&self, _window: &mut Window, cx: &mut Context<Self>) -> Div {
        let label_w = px(140.0);
        let gen_val = self.generator_select_state.read(cx).selected_value().map(|v| v.to_string()).unwrap_or_default();

        div()
            .flex()
            .flex_col()
            .h_full()
            .child(
                div()
                    .flex_1()
                    .overflow_y_scrollbar()
                    .p_4()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_3()
                            // 生成器选择
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(div().w(label_w).text_sm().child("生成器"))
                                    .child(
                                        div().flex_1().child(
                                            Select::new(&self.generator_select_state)
                                                .search_placeholder("搜索生成器..."),
                                        ),
                                    ),
                            )
                            // ── 生成器专属配置（每次 render 重新求值，切换生成器后自动更新）──
                            .child({
                                let mut spec_div = div().flex().flex_col().gap_3();
                                match gen_val.as_str() {
                                    "number" => {
                                        spec_div = spec_div
                                            .child(form_field_entity("开始", &self.gen_number_start_state))
                                            .child(form_field_entity("结束", &self.gen_number_end_state));
                                    }
                                    "date" | "datetime" => {
                                        spec_div = spec_div
                                            .child(form_field_entity("开始日期", &self.gen_date_start_state))
                                            .child(form_field_entity("结束日期", &self.gen_date_end_state));
                                    }
                                    "text" => {
                                        spec_div = spec_div
                                            .child(form_field_entity("最小字符数", &self.gen_text_min_len_state))
                                            .child(form_field_entity("最大字符数", &self.gen_text_max_len_state));
                                    }
                                    "regex" => {
                                        spec_div = spec_div
                                            .child(form_field_entity("正则表达式", &self.gen_regex_pattern_state));
                                    }
                                    "enum" => {
                                        spec_div = spec_div
                                            .child(form_field_entity("枚举值", &self.gen_enum_values_state));
                                    }
                                    "sequence" => {
                                        spec_div = spec_div
                                            .child(form_field_entity("起始值", &self.gen_seq_start_state))
                                            .child(form_field_entity("步长", &self.gen_seq_step_state));
                                    }
                                    _ => {}
                                }
                                spec_div
                            })
                            // 语言选择
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(div().w(label_w).text_sm().child("语言"))
                                    .child(
                                        div().flex_1().child(Select::new(&self.gen_locale_state)),
                                    ),
                            )
                            // 预览
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(div().w(label_w).text_sm().child("预览"))
                                    .child(
                                        div()
                                            .flex_1()
                                            .flex()
                                            .items_center()
                                            .gap_2()
                                            .child(
                                                div()
                                                    .flex_1()
                                                    .child(Input::new(&self.gen_preview_state)),
                                            )
                                            .child(
                                                Button::new("refresh-preview")
                                                    .label("刷新")
                                                    .on_click(cx.listener(|this, _, window, cx| {
                                                        this.preview_generator(window, cx);
                                                    })),
                                            ),
                                    ),
                            )
                            // 包含默认值
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(div().w(label_w).text_sm().child("包含默认值"))
                                    .child(
                                        Checkbox::new(("cb-default", 1_usize))
                                            .checked(self.gen_include_default)
                                            .on_click(cx.listener(|this, checked: &bool, _, cx| {
                                                this.gen_include_default = *checked;
                                                cx.notify();
                                            })),
                                    ),
                            )
                            // 默认值输入
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(div().w(label_w))
                                    .child(
                                        div().flex_1().child(Input::new(&self.gen_default_value_state)),
                                    ),
                            )
                            // 默认值百分比
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(div().w(label_w))
                                    .child(
                                        div()
                                            .flex_1()
                                            .flex()
                                            .items_center()
                                            .gap_1()
                                            .child(div().w(px(120.)).child(
                                                NumberInput::new(&self.gen_default_percentage_state),
                                            ))
                                            .child(div().text_sm().child("%")),
                                    ),
                            )
                            // 包含NULL值
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(div().w(label_w).text_sm().child("包含NULL值"))
                                    .child(
                                        Checkbox::new(("cb-null", 2_usize))
                                            .checked(self.gen_include_null)
                                            .on_click(cx.listener(|this, checked: &bool, _, cx| {
                                                this.gen_include_null = *checked;
                                                cx.notify();
                                            })),
                                    ),
                            )
                            // NULL值百分比
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(div().w(label_w))
                                    .child(
                                        div()
                                            .flex_1()
                                            .flex()
                                            .items_center()
                                            .gap_1()
                                            .child(div().w(px(120.)).child(
                                                NumberInput::new(&self.gen_null_percentage_state),
                                            ))
                                            .child(div().text_sm().child("%")),
                                    ),
                            )
                            // 唯一值
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(div().w(label_w).text_sm().child("唯一值"))
                                    .child(
                                        Checkbox::new(("cb-unique", 3_usize))
                                            .checked(self.gen_unique)
                                            .on_click(cx.listener(|this, checked: &bool, _, cx| {
                                                this.gen_unique = *checked;
                                                cx.notify();
                                            })),
                                    ),
                            )
                            // 禁用字段之间数据链接
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(div().w(label_w).text_sm().child("禁用字段之间数据链接"))
                                    .child(
                                        Checkbox::new(("cb-forbidden", 4_usize))
                                            .checked(self.gen_forbidden_links)
                                            .on_click(cx.listener(|this, checked: &bool, _, cx| {
                                                this.gen_forbidden_links = *checked;
                                                cx.notify();
                                            })),
                                    ),
                            )
                            // 重置属性
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(div().w(label_w))
                                    .child(
                                        Button::new("reset-gen-config")
                                            .label("重置属性")
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.reset_generator_config(window, cx);
                                            })),
                                    ),
                            ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .justify_end()
                    .gap_2()
                    .p_4()
                    .border_t_1()
                    .border_color(cx.theme().border)
                    .child(
                        Button::new("cancel-gen-drawer")
                            .label("取消")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.close_gen_dialog(window, cx);
                            })),
                    )
                    .child(
                        Button::new("save-gen-drawer")
                            .primary()
                            .label("保存")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.save_gen_dialog(window, cx);
                            })),
                    ),
            )
    }

    /// 获取当前选中列的 column_type（用于对话框标题）
    fn get_selected_column_type(&self) -> String {
        let key = match &self.selected_table {
            Some(k) => k.clone(),
            None => return String::new(),
        };
        let column_name = match &self.selected_column_name {
            Some(n) => n.clone(),
            None => return String::new(),
        };
        self.canvas_nodes
            .iter()
            .find(|node| table_key(&node.table.schema, &node.table.table_name) == key)
            .and_then(|node| node.table.columns.iter().find(|c| c.name == column_name))
            .map(|c| c.column_type.clone())
            .or_else(|| {
                self.tables
                    .iter()
                    .find(|t| table_key(&t.schema, &t.table_name) == key)
                    .and_then(|t| t.columns.iter().find(|c| c.name == column_name))
                    .map(|c| c.column_type.clone())
            })
            .unwrap_or_default()
    }

    // 预览当前生成器输出（对齐 Tauri SimplePreview 的 preview 函数）
    fn preview_generator(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(column_name) = self.selected_column_name.clone() else {
            return;
        };
        let generator = self
            .generator_select_state
            .read(cx)
            .selected_value()
            .cloned()
            .unwrap_or_default();
        if generator.is_empty() {
            return;
        }
        let value = generate_column_value(&ColumnPreview {
            name: column_name,
            column_type: String::new(),
            comment: String::new(),
            generator: generator.clone(),
            config: GeneratorConfig::default(),
        });
        self.gen_preview_state.update(cx, |state, cx| {
            state.set_value(value, window, cx);
        });
        cx.notify();
    }

    // 重置生成器配置为默认值（对齐 Tauri SimplePreview 的 reset 函数）
    fn reset_generator_config(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.gen_locale_state.update(cx, |state, cx| {
            state.set_selected_value(&"简体中文".to_string(), window, cx);
        });
        self.gen_default_value_state.update(cx, |state, cx| {
            state.set_value(String::new(), window, cx);
        });
        self.gen_default_percentage_state.update(cx, |state, cx| {
            state.set_value("5".to_string(), window, cx);
        });
        self.gen_null_percentage_state.update(cx, |state, cx| {
            state.set_value("5".to_string(), window, cx);
        });
        self.gen_preview_state.update(cx, |state, cx| {
            state.set_value(String::new(), window, cx);
        });
        self.gen_include_default = false;
        self.gen_include_null = false;
        self.gen_unique = false;
        self.gen_forbidden_links = false;
        cx.notify();
    }

    fn set_column_generator(
        &mut self,
        table_key_value: String,
        column_name: String,
        generator: String,
        cx: &mut Context<Self>,
    ) {
        let cfg = GeneratorConfig::default();
        self.set_column_generator_with_config(table_key_value, column_name, generator, cfg, cx);
    }

    // 保存生成器及完整配置（对齐 Tauri saveChanges：写入 datafaker + config）
    fn set_column_generator_with_config(
        &mut self,
        table_key_value: String,
        column_name: String,
        generator: String,
        config: GeneratorConfig,
        cx: &mut Context<Self>,
    ) {
        let mut updated = false;
        for table in &mut self.tables {
            if table_key(&table.schema, &table.table_name) != table_key_value {
                continue;
            }
            for column in &mut table.columns {
                if column.name == column_name {
                    column.generator = generator.clone();
                    column.config = config.clone();
                    updated = true;
                    break;
                }
            }
        }
        // 同步更新 canvas_nodes 中的对应字段（确保修改生效）
        for node in &mut self.canvas_nodes {
            if table_key(&node.table.schema, &node.table.table_name) != table_key_value {
                continue;
            }
            for column in &mut node.table.columns {
                if column.name == column_name {
                    column.generator = generator.clone();
                    column.config = config.clone();
                    break;
                }
            }
        }

        self.status = if updated {
            format!("字段 {column_name} 已切换为生成器 {generator}。")
        } else {
            format!("字段 {column_name} 不存在，请重新加载表结构。")
        };
        cx.notify();
    }

    fn preview(&mut self, cx: &mut Context<Self>) {
        let count = self
            .row_count_state
            .read(cx)
            .value()
            .parse::<usize>()
            .unwrap_or(10)
            .clamp(1, 200);
        let Some(table) = self.current_table().cloned() else {
            self.status = "请先加载并选择表。".to_string();
            cx.notify();
            return;
        };

        let mut lines = Vec::new();
        lines.push(
            table
                .columns
                .iter()
                .map(|column| column.name.clone())
                .collect::<Vec<_>>()
                .join(","),
        );
        for _ in 0..count {
            lines.push(
                table
                    .columns
                    .iter()
                    .map(|column| csv_escape(&generate_column_value(column)))
                    .collect::<Vec<_>>()
                    .join(","),
            );
        }
        self.output = lines.join("\n");
        self.status = format!(
            "已预览 {}.{} 的 {} 行数据",
            table.schema, table.table_name, count
        );
        cx.notify();
    }

    fn run_insert(&mut self, selected_only: bool, cx: &mut Context<Self>) {
        let Ok(info) = self.datasource_info(cx) else {
            self.status = "请先补全连接信息。".to_string();
            cx.notify();
            return;
        };
        let count = self
            .row_count_state
            .read(cx)
            .value()
            .parse::<usize>()
            .unwrap_or(10)
            .clamp(1, 100_000);
        let tables = if selected_only {
            match self.current_table().cloned() {
                Some(table) => vec![table],
                None => {
                    self.status = "请先加载并选择表。".to_string();
                    cx.notify();
                    return;
                }
            }
        } else {
            self.tables.clone()
        };
        if tables.is_empty() {
            self.status = "请先加载表结构。".to_string();
            cx.notify();
            return;
        }

        self.is_running = true;
        self.output = format!(
            "开始写入假数据: 表数量={}, 每张表行数={count}\n",
            tables.len()
        );
        self.status = "正在写入假数据...".to_string();
        cx.notify();

        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            let result = run_fake_data_insert(info, tables, count).await;
            let _ = this.update(cx, |this, cx| {
                this.is_running = false;
                match result {
                    Ok(output) => {
                        this.output = output;
                        this.status = "假数据写入完成".to_string();
                    }
                    Err(err) => {
                        this.output.push_str(&format!("\n写入失败: {err}"));
                        this.status = format!("假数据写入失败：{err}");
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn current_table(&self) -> Option<&TablePreview> {
        let selected = self.selected_table.as_ref()?;
        self.tables
            .iter()
            .find(|table| table_key(&table.schema, &table.table_name) == *selected)
    }

    fn toggle_canvas_mode(&mut self, cx: &mut Context<Self>) {
        // 不再使用，保留空方法以防编译错误
        cx.notify();
    }

    fn add_to_canvas(&mut self, key: String, cx: &mut Context<Self>) {
        self.add_table_to_canvas(key, cx);
    }

    fn remove_from_canvas(&mut self, index: usize, cx: &mut Context<Self>) {
        if index < self.canvas_nodes.len() {
            self.canvas_nodes.remove(index);
            self.status = format!("已从画布移除表，剩余 {} 张。", self.canvas_nodes.len());
            cx.notify();
        }
    }

    fn clear_canvas(&mut self, cx: &mut Context<Self>) {
        self.canvas_nodes.clear();
        self.canvas_viewport_x = 0.0;
        self.canvas_viewport_y = 0.0;
        self.canvas_zoom = 1.0;
        self.canvas_drag = None;
        self.run_logs.clear();
        self.show_run_log = false;
        self.status = "画布已清空。".to_string();
        cx.notify();
    }

    fn canvas_node_mouse_down(
        &mut self,
        index: usize,
        event: &MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if event.button != MouseButton::Left {
            return;
        }
        if let Some(node) = self.canvas_nodes.get(index) {
            let mouse_x = f32::from(event.position.x);
            let mouse_y = f32::from(event.position.y);
            let node_screen_x = node.x * self.canvas_zoom + self.canvas_viewport_x;
            let node_screen_y = node.y * self.canvas_zoom + self.canvas_viewport_y;
            self.canvas_drag = Some(CanvasDrag::Node {
                index,
                offset_x: mouse_x - node_screen_x,
                offset_y: mouse_y - node_screen_y,
            });
            cx.notify();
        }
    }

    fn canvas_bg_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if event.button != MouseButton::Left {
            return;
        }
        self.canvas_drag = Some(CanvasDrag::Pan {
            start_mouse_x: f32::from(event.position.x),
            start_mouse_y: f32::from(event.position.y),
            origin_vx: self.canvas_viewport_x,
            origin_vy: self.canvas_viewport_y,
        });
        cx.notify();
    }

    fn canvas_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(drag) = self.canvas_drag.take() else {
            return;
        };
        match drag {
            CanvasDrag::Node {
                index,
                offset_x,
                offset_y,
            } => {
                let mouse_x = f32::from(event.position.x);
                let mouse_y = f32::from(event.position.y);
                let new_screen_x = mouse_x - offset_x;
                let new_screen_y = mouse_y - offset_y;
                let new_canvas_x = (new_screen_x - self.canvas_viewport_x) / self.canvas_zoom;
                let new_canvas_y = (new_screen_y - self.canvas_viewport_y) / self.canvas_zoom;

                // 计算位移增量（对齐 Tauri onNodeDrag：同步移动同表的所有子节点）
                let (delta_x, delta_y) = if let Some(node) = self.canvas_nodes.get(index) {
                    (new_canvas_x - node.x, new_canvas_y - node.y)
                } else {
                    (0.0, 0.0)
                };

                if let Some(node) = self.canvas_nodes.get_mut(index) {
                    node.x = new_canvas_x;
                    node.y = new_canvas_y;
                }
                self.canvas_drag = Some(CanvasDrag::Node {
                    index,
                    offset_x,
                    offset_y,
                });
                // 记录位移用于后续同步（已经直接设置好了主节点，子节点在渲染时自动跟随）
                let _ = delta_x;
                let _ = delta_y;
                cx.notify();
            }
            CanvasDrag::Pan {
                start_mouse_x,
                start_mouse_y,
                origin_vx,
                origin_vy,
            } => {
                let dx = f32::from(event.position.x) - start_mouse_x;
                let dy = f32::from(event.position.y) - start_mouse_y;
                self.canvas_viewport_x = origin_vx + dx;
                self.canvas_viewport_y = origin_vy + dy;
                self.canvas_drag = Some(CanvasDrag::Pan {
                    start_mouse_x,
                    start_mouse_y,
                    origin_vx,
                    origin_vy,
                });
                cx.notify();
            }
        }
    }

    fn canvas_mouse_up(
        &mut self,
        _event: &MouseUpEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.canvas_drag.is_some() {
            self.canvas_drag = None;
            cx.notify();
        }
    }

    fn canvas_scroll_wheel(
        &mut self,
        event: &ScrollWheelEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if event.modifiers.control || event.modifiers.platform {
            let delta = event.delta.pixel_delta(px(20.0));
            let zoom_delta = -f32::from(delta.y) / 200.0;
            let old_zoom = self.canvas_zoom;
            self.canvas_zoom = (self.canvas_zoom + zoom_delta).clamp(0.1, 5.0);
            if (self.canvas_zoom - old_zoom).abs() > 0.001 {
                // 以鼠标位置为中心缩放
                let mouse_x = f32::from(event.position.x);
                let mouse_y = f32::from(event.position.y);
                let ratio = self.canvas_zoom / old_zoom;
                self.canvas_viewport_x = mouse_x - ratio * (mouse_x - self.canvas_viewport_x);
                self.canvas_viewport_y = mouse_y - ratio * (mouse_y - self.canvas_viewport_y);
                cx.notify();
            }
        }
    }

    fn run_canvas_insert(&mut self, cx: &mut Context<Self>) {
        self.run_canvas_config(cx);
    }

    fn save_canvas(&mut self, cx: &mut Context<Self>) {
        self.save_generator_config(cx);
    }

    fn load_canvas(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // 对齐 Tauri：根据当前数据源名称直接加载配置，不弹出对话框
        let ds_name = match &self.current_datasource {
            Some(ds) => ds.name.clone(),
            None => return,
        };
        self.status = "正在加载画布配置...".to_string();
        cx.notify();

        cx.spawn_in(window, async move |this: WeakEntity<Self>, cx| {
            let result = config_store::load_datafaker_configs().await;
            let _ = this.update_in(cx, |this, window, cx| {
                match result {
                    Ok(configs) => {
                        if let Some(config) = configs.iter().find(|c| c.name == ds_name) {
                            this.apply_canvas_config(
                                config.nodes_json.clone(),
                                config.name.clone(),
                                window,
                                cx,
                            );
                            this.status = format!("已加载画布配置 {ds_name}。");
                        } else {
                            this.status = "未找到已保存的画布配置。".to_string();
                        }
                    }
                    Err(err) => {
                        this.status = format!("加载画布配置失败：{err}");
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn show_canvas_load_dialog(
        &mut self,
        configs: Vec<config_store::DatafakerConfigRecord>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let this = cx.entity().downgrade();
        window.open_dialog(cx, move |dialog, _, cx| {
            dialog
                .title(div().text_lg().font_semibold().child("选择画布配置"))
                .width(px(640.))
                .child(canvas_load_dialog_content(
                    configs.clone(),
                    this.clone(),
                    cx,
                ))
        });
    }

    fn apply_canvas_config(
        &mut self,
        nodes_json: String,
        name: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let nodes_value: Vec<serde_json::Value> =
            serde_json::from_str(&nodes_json).unwrap_or_default();
        let mut canvas_nodes = Vec::new();
        for node_value in nodes_value {
            let schema = node_value
                .get("schema")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .to_string();
            let table_name = node_value
                .get("tableName")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .to_string();
            let table_comment = node_value
                .get("tableComment")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .to_string();
            let x = node_value
                .get("x")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(40.0) as f32;
            let y = node_value
                .get("y")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(40.0) as f32;
            let columns = node_value
                .get("columns")
                .and_then(serde_json::Value::as_array)
                .cloned()
                .unwrap_or_default()
                .iter()
                .map(|col| ColumnPreview {
                    name: col
                        .get("name")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or_default()
                        .to_string(),
                    column_type: col
                        .get("type")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("text")
                        .to_string(),
                    comment: col
                        .get("comment")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or_default()
                        .to_string(),
                    generator: col
                        .get("generator")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("text")
                        .to_string(),
                    config: GeneratorConfig::default(),
                })
                .collect::<Vec<_>>();
            canvas_nodes.push(CanvasNode {
                table: TablePreview {
                    schema,
                    table_name,
                    table_comment,
                    columns,
                },
                x,
                y,
            });
        }
        self.canvas_nodes = canvas_nodes;
        self.canvas_name_state.update(cx, |state, cx| {
            state.set_value(name.clone(), window, cx);
        });
        self.status = format!(
            "已加载画布配置 {name}，共 {} 张表。",
            self.canvas_nodes.len()
        );
        cx.notify();
    }

    fn delete_canvas_config(&mut self, name: String, cx: &mut Context<Self>) {
        self.status = format!("正在删除画布配置 {name}...");
        cx.notify();

        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            let result = config_store::delete_datafaker_config(name.clone()).await;
            let _ = this.update(cx, |this, cx| {
                this.status = match result {
                    Ok(true) => format!("画布配置 {name} 已删除。"),
                    Ok(false) => format!("未找到画布配置 {name}。"),
                    Err(err) => format!("删除画布配置失败：{err}"),
                };
                cx.notify();
            });
        })
        .detach();
    }

    fn clear(&mut self) {
        self.output.clear();
        self.status = "预览结果已清空。".to_string();
    }

    fn copy(&mut self, cx: &mut Context<Self>) {
        cx.write_to_clipboard(ClipboardItem::new_string(self.output.clone()));
    }
}

// ── 渲染 ──

impl Render for FakeDataGenerator {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        match self.active_view {
            FakerView::ConnectionList => self.render_connection_list(cx),
            FakerView::Generator => self.render_generator(window, cx),
        }
    }
}

impl FakeDataGenerator {
    fn render_connection_list(&mut self, cx: &mut Context<Self>) -> Div {
        div()
            .size_full()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                div().flex().items_center().justify_between().child(
                    Button::new("new-conn")
                        .label("新建连接")
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.open_conn_sheet(None, window, cx);
                        })),
                ),
            )
            .child(self.render_datasource_table(cx))
            .child(
                div()
                    .text_sm()
                    .text_color(if self.is_running {
                        cx.theme().warning
                    } else {
                        cx.theme().muted_foreground
                    })
                    .child(self.status.clone()),
            )
    }

    fn render_datasource_table(&mut self, cx: &mut Context<Self>) -> Div {
        let header = div()
            .flex()
            .items_center()
            .py(px(8.))
            .px(px(8.))
            .border_b_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().muted)
            .child(
                div()
                    .w(px(120.))
                    .text_sm()
                    .font_semibold()
                    .child("连接名称"),
            )
            .child(div().w(px(140.)).text_sm().font_semibold().child("主机"))
            .child(div().w(px(60.)).text_sm().font_semibold().child("端口"))
            .child(div().flex_1().text_sm().font_semibold().child("数据库"))
            .child(div().w(px(200.)).text_sm().font_semibold().child("操作"));

        let mut rows_div = div().flex().flex_col();

        if self.saved_datasources.is_empty() {
            rows_div = rows_div.child(
                div()
                    .py(px(16.))
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("暂无连接，点击「新建连接」添加。"),
            );
        } else {
            for (i, ds) in self.saved_datasources.iter().enumerate() {
                let ds_name = ds.name.clone();
                let ds_host = ds.host.clone();
                let ds_port = ds.port.map(|p| p.to_string()).unwrap_or_default();
                let ds_database = ds.database.clone().unwrap_or_default();
                let config_name = ds.name.clone();
                let edit_name = ds.name.clone();
                let delete_name = ds.name.clone();

                rows_div = rows_div.child(
                    div()
                        .flex()
                        .items_center()
                        .py(px(6.))
                        .px(px(8.))
                        .border_b_1()
                        .border_color(cx.theme().border)
                        .child(div().w(px(120.)).text_sm().child(ds_name))
                        .child(div().w(px(140.)).text_sm().child(ds_host))
                        .child(div().w(px(60.)).text_sm().child(ds_port))
                        .child(div().flex_1().text_sm().child(ds_database))
                        .child(
                            div()
                                .w(px(200.))
                                .flex()
                                .gap_1()
                                .child(
                                    Button::new(("config", i))
                                        .ghost()
                                        .xsmall()
                                        .label("配置")
                                        .on_click(cx.listener(move |this, _, window, cx| {
                                            this.go_to_generator(config_name.clone(), window, cx);
                                        })),
                                )
                                .child(
                                    Button::new(("edit", i))
                                        .ghost()
                                        .xsmall()
                                        .label("编辑")
                                        .on_click(cx.listener(move |this, _, window, cx| {
                                            this.open_conn_sheet(
                                                Some(edit_name.clone()),
                                                window,
                                                cx,
                                            );
                                        })),
                                )
                                .child(
                                    Button::new(("delete", i))
                                        .ghost()
                                        .xsmall()
                                        .label("删除")
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            this.delete_datasource_by_name(delete_name.clone(), cx);
                                        })),
                                ),
                        ),
                );
            }
        }

        div()
            .border_1()
            .border_color(cx.theme().border)
            .rounded_lg()
            .overflow_hidden()
            .child(header)
            .child(rows_div)
    }

    fn render_generator(&mut self, window: &mut Window, cx: &mut Context<Self>) -> Div {
        let keyword = self.table_filter.to_lowercase();
        let filtered_tables: Vec<TablePreview> = self
            .tables
            .iter()
            .filter(|table| {
                if keyword.trim().is_empty() {
                    return true;
                }
                format!(
                    "{} {} {}",
                    table.schema, table.table_name, table.table_comment
                )
                .to_lowercase()
                .contains(keyword.trim())
            })
            .cloned()
            .collect();

        let ds_name = self
            .current_datasource
            .as_ref()
            .map(|ds| ds.name.clone())
            .unwrap_or_default();

        div()
            .size_full()
            .flex()
            .flex_row()
            .relative()
            // 左侧面板
            .child(
                div()
                    .w(px(300.))
                    .h_full()
                    .flex()
                    .flex_col()
                    .border_r_1()
                    .border_color(cx.theme().border)
                    .bg(cx.theme().background)
                    .child(
                        div()
                            .p_3()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(
                                Button::new("go-back")
                                    .label("返回上一页")
                                    .w_full()
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.go_back(window, cx);
                                    })),
                            )
                            .child(Input::new(&self.table_filter_state)),
                    )
                    .child(div().flex_1().overflow_y_scrollbar().p_3().pt_0().children(
                        if self.tables.is_empty() {
                            vec![
                                div()
                                    .id("no-tables")
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("暂无表结构"),
                            ]
                        } else if filtered_tables.is_empty() {
                            vec![
                                div()
                                    .id("no-match-tables")
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("没有匹配的表"),
                            ]
                        } else {
                            filtered_tables
                                .into_iter()
                                .enumerate()
                                .map(|(index, table)| {
                                    let key = table_key(&table.schema, &table.table_name);
                                    let table_name = table.table_name.clone();
                                    let table_comment = table.table_comment.clone();
                                    div()
                                        .id(("table-item", index))
                                        .pb_2()
                                        .border_b_1()
                                        .border_color(cx.theme().border)
                                        .cursor_pointer()
                                        .on_mouse_down(
                                            MouseButton::Left,
                                            cx.listener({
                                                let key = key.clone();
                                                move |this, _, _, cx| {
                                                    this.add_table_to_canvas(key.clone(), cx);
                                                }
                                            }),
                                        )
                                        .child(div().text_sm().font_weight(FontWeight::SEMIBOLD).child(table_name))
                                        .when(!table_comment.is_empty(), |el| {
                                            el.child(
                                                div()
                                                    .text_xs()
                                                    .text_color(cx.theme().muted_foreground)
                                                    .child(table_comment),
                                            )
                                        })
                                })
                                .collect::<Vec<_>>()
                        },
                    )),
            )
            // 右侧主区域
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .relative()
                    // 工具栏（浮动在画布上方）
                    .child(
                        div()
                            .absolute()
                            .top(px(12.))
                            .left(px(12.))
                            .flex()
                            .items_center()
                            .gap_2()
                            .p(px(8.))
                            .bg(cx.theme().background)
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded_md()
                            .shadow_sm()
                            .child(div().text_sm().child("每张表生成"))
                            .child(
                                div()
                                    .w(px(80.))
                                    .child(NumberInput::new(&self.row_count_state)),
                            )
                            .child(div().text_sm().child("行"))
                            .child(
                                Button::new("save-canvas")
                                    .label("保存配置")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.save_generator_config(cx);
                                    })),
                            )
                            .child(
                                Button::new("clear-canvas")
                                    .label("清空画布")
                                    .warning()
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        // 二次确认弹窗（对齐 Tauri n-popconfirm）
                                        let weak = cx.entity().downgrade();
                                        window.open_dialog(cx, move |dialog, _, _| {
                                            dialog
                                                .title(div().text_lg().font_semibold().child("确认清空"))
                                                .width(px(360.))
                                                .child(
                                                    div()
                                                        .p_4()
                                                        .child("清空后会移除所有表和列配置，确定要清空画布吗？")
                                                        .child(
                                                            div()
                                                                .flex()
                                                                .justify_end()
                                                                .gap_2()
                                                                .mt_4()
                                                                .child(
                                                                    Button::new("clear-cancel")
                                                                        .label("取消")
                                                                        .on_click(|_, window, cx| {
                                                                            window.close_dialog(cx);
                                                                        }),
                                                                )
                                                                .child({
                                                                    let weak = weak.clone();
                                                                    Button::new("clear-confirm")
                                                                        .primary()
                                                                        .label("确定")
                                                                        .on_click(move |_, window, cx| {
                                                                            if let Some(this) = weak.upgrade() {
                                                                                this.update(cx, |this, cx| {
                                                                                    this.clear_canvas(cx);
                                                                                });
                                                                            }
                                                                            window.close_dialog(cx);
                                                                        })
                                                                }),
                                                        ),
                                                )
                                        });
                                    })),
                            )
                            .child(
                                Button::new("run-canvas")
                                    .primary()
                                    .label("运行配置")
                                    .when(self.is_running, |btn| btn.disabled(true))
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.run_canvas_config(cx);
                                    })),
                            )
                            .when(self.run_logs.len() > 0 && !self.show_run_log, |el| {
                                el.child(
                                    Button::new("show-run-log")
                                        .label(format!("查看日志（{}）", self.run_logs.len()))
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.show_run_log = true;
                                            cx.notify();
                                        })),
                                )
                            }),
                    )
                    // 运行日志面板
                    .when(self.show_run_log && !self.run_logs.is_empty(), |el| {
                        el.child(
                            div()
                                .absolute()
                                .top(px(64.))
                                .left(px(12.))
                                .right(px(12.))
                                .max_w(px(960.))
                                .border_1()
                                .border_color(cx.theme().border)
                                .rounded_md()
                                .bg(cx.theme().background)
                                .shadow_sm()
                                .flex()
                                .flex_col()
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .px(px(12.))
                                        .py(px(8.))
                                        .border_b_1()
                                        .border_color(cx.theme().border)
                                        .child(div().text_sm().font_semibold().child("运行日志"))
                                        .child(
                                            Button::new("close-run-log")
                                                .ghost()
                                                .xsmall()
                                                .label("关闭")
                                                .on_click(cx.listener(|this, _, _, cx| {
                                                    this.show_run_log = false;
                                                    cx.notify();
                                                })),
                                        ),
                                )
                                .child(
                                    div()
                                        .max_h(px(200.))
                                        .overflow_y_scrollbar()
                                        .p(px(12.))
                                        .text_sm()
                                        .font_family("monospace")
                                        .child(self.run_logs.join("\n")),
                                ),
                        )
                    })
                    // 画布区域
                    .child(self.render_canvas_area(cx)),
            )
            // 生成器配置抽屉（条件渲染，每次 cx.notify() 重新求值）
            .when(self.show_gen_dialog, |el| {
                // 构建标题
                let column_name = self.selected_column_name.clone().unwrap_or_default();
                let column_type = self.get_selected_column_type();
                let title_text = format!("{} {} 生成器配置", column_name, column_type);
                el.child(
                    div()
                        .absolute()
                        .top_0()
                        .right_0()
                        .h_full()
                        .w(px(460.))
                        .flex()
                        .flex_col()
                        .bg(cx.theme().background)
                        .border_l_1()
                        .border_color(cx.theme().border)
                        .shadow_lg()
                        // 阻止鼠标事件穿透到画布（防止在抽屉内移动鼠标时画布跟着平移）
                        .on_mouse_down(MouseButton::Left, |_, _, cx| {
                            cx.stop_propagation();
                        })
                        .on_mouse_move(|_, _, cx| {
                            cx.stop_propagation();
                        })
                        .on_mouse_up(MouseButton::Left, |_, _, cx| {
                            cx.stop_propagation();
                        })
                        .on_scroll_wheel(|_, _, cx| {
                            cx.stop_propagation();
                        })
                        // 标题栏 + 关闭按钮
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .px_4()
                                .py_3()
                                .border_b_1()
                                .border_color(cx.theme().border)
                                .child(div().text_lg().font_semibold().child(title_text))
                                .child(
                                    Button::new("close-gen-panel")
                                        .xsmall()
                                        .ghost()
                                        .icon(Icon::new(IconName::Close))
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.close_gen_dialog(window, cx);
                                        })),
                                ),
                        )
                        // 面板内容
                        .child(self.gen_dialog_content(window, cx)),
                )
            })
    }

    fn render_canvas_area(&mut self, cx: &mut Context<Self>) -> Div {
        let dot_color = gpui::hsla(0.0, 0.0, 0.85, 1.0);
        let dot_size = 1.5_f32;

        // 捕获节点数据供连线绘制使用
        let nodes_for_lines = self.canvas_nodes.clone();
        let viewport_x = self.canvas_viewport_x;
        let viewport_y = self.canvas_viewport_y;
        let zoom = self.canvas_zoom;
        let line_color = gpui::hsla(0.6, 0.6, 0.55, 1.0);

        let mut canvas_div = div()
            .relative()
            .flex_1()
            .w_full()
            .border_1()
            .border_color(cx.theme().border)
            .overflow_hidden()
            .bg(gpui::white())
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, event, window, cx| {
                    this.canvas_bg_mouse_down(event, window, cx);
                }),
            )
            .on_mouse_move(cx.listener(|this, event, window, cx| {
                this.canvas_mouse_move(event, window, cx);
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, event, window, cx| {
                    this.canvas_mouse_up(event, window, cx);
                }),
            )
            .on_scroll_wheel(cx.listener(|this, event, window, cx| {
                this.canvas_scroll_wheel(event, window, cx);
            }));

        // 点阵网格背景 + 连线
        canvas_div = canvas_div.child(
            canvas(
                move |_, _, _| {},
                move |bounds: Bounds<Pixels>, _, window: &mut Window, _| {
                    window.paint_quad(fill(bounds, gpui::white()));
                    // 点阵网格（缩放适配）
                    let step = 20.0_f32 * zoom;
                    if step > 2.0 {
                        let start_x = viewport_x.rem_euclid(step);
                        let start_y = viewport_y.rem_euclid(step);
                        let mut x = start_x;
                        while x < f32::from(bounds.size.width) {
                            let mut y = start_y;
                            while y < f32::from(bounds.size.height) {
                                let dot_bounds = Bounds::new(
                                    bounds.origin
                                        + point(px(x - dot_size / 2.0), px(y - dot_size / 2.0)),
                                    size(px(dot_size), px(dot_size)),
                                );
                                window.paint_quad(fill(dot_bounds, dot_color));
                                y += step;
                            }
                            x += step;
                        }
                    }

                    // 连线：datafaker → column，带箭头
                    for node in &nodes_for_lines {
                        let sx = node.x * zoom + viewport_x;
                        let sy = node.y * zoom + viewport_y;
                        for i in 0..node.table.columns.len() {
                            let col_right_x = sx + COLUMN_NODE_WIDTH * zoom;
                            let field_center_y = sy + TABLE_NODE_HEIGHT * zoom
                                + i as f32 * FIELD_NODE_HEIGHT * zoom
                                + FIELD_NODE_HEIGHT * zoom / 2.0;
                            let gen_left_x = sx + GENERATOR_OFFSET_X * zoom;

                            let start = bounds.origin
                                + point(px(gen_left_x), px(field_center_y));
                            let end = bounds.origin
                                + point(px(col_right_x), px(field_center_y));

                            // 画线
                            let mut path = PathBuilder::stroke(px(1.5 * zoom));
                            path.move_to(start);
                            path.line_to(end);

                            // 箭头（终点在 column 右侧）
                            let dx = f32::from(end.x - start.x);
                            let dy = f32::from(end.y - start.y);
                            let len = (dx * dx + dy * dy).sqrt().max(0.001);
                            let ux = dx / len;
                            let uy = dy / len;
                            let arrow_len = 10.0_f32 * zoom;
                            let arrow_angle = 0.45f32;
                            let cos_a = arrow_angle.cos();
                            let sin_a = arrow_angle.sin();
                            let end_x = f32::from(end.x);
                            let end_y = f32::from(end.y);
                            let ax1 = end_x - arrow_len * (ux * cos_a + uy * sin_a);
                            let ay1 = end_y - arrow_len * (uy * cos_a - ux * sin_a);
                            let ax2 = end_x - arrow_len * (ux * cos_a - uy * sin_a);
                            let ay2 = end_y - arrow_len * (uy * cos_a + ux * sin_a);
                            path.move_to(end);
                            path.line_to(point(px(ax1), px(ay1)));
                            path.move_to(end);
                            path.line_to(point(px(ax2), px(ay2)));

                            if let Ok(p) = path.build() {
                                window.paint_path(p, line_color);
                            }
                        }
                    }
                },
            )
            .absolute()
            .size_full(),
        );

        if self.canvas_nodes.is_empty() {
            canvas_div = canvas_div.child(
                div()
                    .absolute()
                    .left(px(16.))
                    .top(px(16.))
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("画布为空，从左侧表列表添加表到画布。"),
            );
            return canvas_div;
        }

        // 缩放百分比提示
        let zoom_pct = format!("{:.0}%", self.canvas_zoom * 100.0);

        for (index, node) in self.canvas_nodes.iter().enumerate() {
            let screen_x = node.x * self.canvas_zoom + self.canvas_viewport_x;
            let screen_y = node.y * self.canvas_zoom + self.canvas_viewport_y;
            let title = node.table.table_name.clone();

            // ── Table 头节点 ──
            let header_div = div()
                .absolute()
                .left(px(screen_x))
                .top(px(screen_y))
                .w(px(TABLE_NODE_WIDTH * self.canvas_zoom))
                .h(px(TABLE_NODE_HEIGHT * self.canvas_zoom))
                .bg(cx.theme().muted)
                .border_1()
                .border_color(cx.theme().border)
                .rounded_md()
                .px_2()
                .flex()
                .items_center()
                .justify_between()
                .shadow_sm()
                .cursor_grab()
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, event, window, cx| {
                        cx.stop_propagation();
                        this.canvas_node_mouse_down(index, event, window, cx);
                    }),
                )
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(title),
                )
                .child(
                    div().flex().gap_1().child(
                        Button::new(("remove-canvas-node", index))
                            .xsmall()
                            .ghost()
                            .icon(Icon::new(IconName::Close))
                            .tooltip("删除表")
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.remove_from_canvas(index, cx);
                            })),
                    ),
                );
            canvas_div = canvas_div.child(header_div);

            // ── Column 节点列表 ──
            for (col_idx, column) in node.table.columns.iter().enumerate() {
                let col_screen_y = screen_y
                    + TABLE_NODE_HEIGHT * self.canvas_zoom
                    + col_idx as f32 * FIELD_NODE_HEIGHT * self.canvas_zoom;
                let col_div = div()
                    .absolute()
                    .left(px(screen_x))
                    .top(px(col_screen_y))
                    .w(px(COLUMN_NODE_WIDTH * self.canvas_zoom))
                    .h(px(FIELD_NODE_HEIGHT * self.canvas_zoom))
                    .bg(gpui::white())
                    .border_1()
                    .border_color(cx.theme().border)
                    .px_2()
                    .flex()
                    .items_center()
                    .text_xs()
                    .overflow_hidden()
                    .child(format!("{} [{}]", column.name, column.column_type));
                canvas_div = canvas_div.child(col_div);
            }

            // ── Datafaker 节点列表 ──
            for (col_idx, column) in node.table.columns.iter().enumerate() {
                let gen_screen_x = screen_x + GENERATOR_OFFSET_X * self.canvas_zoom;
                let gen_screen_y = screen_y
                    + TABLE_NODE_HEIGHT * self.canvas_zoom
                    + col_idx as f32 * FIELD_NODE_HEIGHT * self.canvas_zoom;
                let generator_name = column.generator.clone();
                let col_name = column.name.clone();
                let col_name_for_btn = col_name.clone();
                let gen_div = div()
                    .absolute()
                    .left(px(gen_screen_x))
                    .top(px(gen_screen_y))
                    .w(px(GENERATOR_NODE_WIDTH * self.canvas_zoom))
                    .h(px(FIELD_NODE_HEIGHT * self.canvas_zoom))
                    .bg(gpui::hsla(0.58, 0.15, 0.95, 1.0))
                    .border_1()
                    .border_color(cx.theme().border)
                    .px_2()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_1()
                    .text_xs()
                    .overflow_hidden()
                    .id(("gen-canvas-node", index * 1000 + col_idx))
                    .cursor_pointer()
                    // 阻止画布 pan 拖拽拦截节点的点击事件
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|_, _, _, cx| {
                            cx.stop_propagation();
                        }),
                    )
                    .on_click(
                        cx.listener(move |this, click_event: &ClickEvent, window, cx| {
                            if click_event.click_count() >= 2 {
                                this.selected_canvas_node = Some(index);
                                this.open_generator_dialog_for_column(col_name.clone(), window, cx);
                            }
                        }),
                    )
                    .child(
                        div()
                            .flex_1()
                            .overflow_hidden()
                            .text_ellipsis()
                            .child(generator_name),
                    )
                    .child(
                        Button::new(("gen-config-btn", index * 1000 + col_idx))
                            .xsmall()
                            .ghost()
                            .icon(Icon::new(IconName::Settings))
                            .tooltip("配置生成器")
                            .on_click(cx.listener(move |this, _, window, cx| {
                                cx.stop_propagation();
                                this.selected_canvas_node = Some(index);
                                this.open_generator_dialog_for_column(col_name_for_btn.clone(), window, cx);
                            })),
                    );
                canvas_div = canvas_div.child(gen_div);
            }
        }

        // 缩放百分比提示（右下角，MiniMap 上方）
        canvas_div = canvas_div.child(
            div()
                .absolute()
                .bottom(px(118.))
                .right(px(8.))
                .px_2()
                .py_1()
                .rounded_md()
                .bg(cx.theme().background)
                .border_1()
                .border_color(cx.theme().border)
                .shadow_sm()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(zoom_pct),
        );

        // MiniMap 缩略图（右下角，对齐 Tauri VueFlow 的 MiniMap）
        let minimap_nodes = self.canvas_nodes.clone();
        let minimap_viewport_x = self.canvas_viewport_x;
        let minimap_viewport_y = self.canvas_viewport_y;
        let minimap_zoom = self.canvas_zoom;
        let minimap_bg = cx.theme().background;
        let minimap_border = cx.theme().border;
        let minimap_muted = cx.theme().muted;
        canvas_div = canvas_div.child(
            div()
                .absolute()
                .bottom(px(8.))
                .right(px(8.))
                .w(px(120.))
                .h(px(100.))
                .border_1()
                .border_color(minimap_border)
                .rounded_md()
                .shadow_sm()
                .overflow_hidden()
                .child(
                    canvas(
                        move |_, _, _| minimap_nodes.clone(),
                        move |bounds: Bounds<Pixels>, nodes, window: &mut Window, _| {
                            window.paint_quad(fill(bounds, minimap_bg));
                            // 计算所有节点的边界
                            let mut min_x = f32::MAX;
                            let mut min_y = f32::MAX;
                            let mut max_x = f32::MIN;
                            let mut max_y = f32::MIN;
                            for node in &nodes {
                                let node_right = node.x + GENERATOR_OFFSET_X + GENERATOR_NODE_WIDTH;
                                let node_bottom = node.y + TABLE_NODE_HEIGHT
                                    + node.table.columns.len().max(1) as f32 * FIELD_NODE_HEIGHT;
                                min_x = min_x.min(node.x);
                                min_y = min_y.min(node.y);
                                max_x = max_x.max(node_right);
                                max_y = max_y.max(node_bottom);
                            }
                            if min_x == f32::MAX {
                                return;
                            }
                            // 增加边距
                            min_x -= 40.0;
                            min_y -= 40.0;
                            max_x += 40.0;
                            max_y += 40.0;
                            let content_w = (max_x - min_x).max(1.0);
                            let content_h = (max_y - min_y).max(1.0);
                            let scale_x = f32::from(bounds.size.width) / content_w;
                            let scale_y = f32::from(bounds.size.height) / content_h;
                            let scale = scale_x.min(scale_y).min(1.0);
                            let offset_x = (f32::from(bounds.size.width) - content_w * scale) / 2.0;
                            let offset_y = (f32::from(bounds.size.height) - content_h * scale) / 2.0;

                            // 绘制节点矩形
                            for node in &nodes {
                                let rx = (node.x - min_x) * scale + offset_x;
                                let ry = (node.y - min_y) * scale + offset_y;
                                let rw = TABLE_NODE_WIDTH * scale;
                                let rh = (TABLE_NODE_HEIGHT
                                    + node.table.columns.len().max(1) as f32 * FIELD_NODE_HEIGHT)
                                    * scale;
                                let node_bounds = Bounds::new(
                                    bounds.origin + point(px(rx), px(ry)),
                                    size(px(rw), px(rh)),
                                );
                                window.paint_quad(
                                    fill(node_bounds, minimap_muted)
                                        .border_widths(0.5)
                                        .border_color(minimap_border),
                                );
                            }

                            // 绘制当前视口矩形
                            let vp_left = (-minimap_viewport_x / minimap_zoom - min_x) * scale + offset_x;
                            let vp_top = (-minimap_viewport_y / minimap_zoom - min_y) * scale + offset_y;
                            // 视口宽高（假设画布大约 800x600，这里用 bounds 反推）
                            let vp_w = f32::from(bounds.size.width) / minimap_zoom * scale;
                            let vp_h = f32::from(bounds.size.height) / minimap_zoom * scale;
                            let vp_bounds = Bounds::new(
                                bounds.origin + point(px(vp_left), px(vp_top)),
                                size(px(vp_w), px(vp_h)),
                            );
                            window.paint_quad(
                                fill(vp_bounds, hsla(0.6, 0.6, 0.55, 0.1))
                                    .border_widths(1.0)
                                    .border_color(hsla(0.6, 0.6, 0.55, 0.8)),
                            );
                        },
                    )
                    .size_full(),
                ),
        );

        canvas_div
    }
}

// ── DatafakerDbForm ──

impl DatafakerDbForm {
    fn new(window: &mut Window, cx: &mut Context<FakeDataGenerator>) -> Self {
        Self {
            name: cx.new(|cx| {
                InputState::new(window, cx).placeholder("请输入连接名称")
            }),
            host: cx.new(|cx| {
                InputState::new(window, cx).placeholder("请输入主机名")
            }),
            port: cx.new(|cx| InputState::new(window, cx).placeholder("请输入端口")),
            database:
                cx.new(|cx| InputState::new(window, cx).placeholder("请输入数据库名")),
            username: cx.new(|cx| InputState::new(window, cx).placeholder("请输入用户名")),
            password: cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder("请输入密码")
                    .masked(true)
            }),
        }
    }

    fn to_info(&self, driver_label: &str, cx: &App) -> Result<DatasourceInfo, String> {
        let driver = match driver_label {
            "MySQL" => Driver::Mysql,
            "SQLite" => Driver::Sqlite,
            _ => Driver::Postgres,
        };
        let name = read_input(&self.name, cx);
        let host = read_input(&self.host, cx);
        let database = read_input(&self.database, cx);
        if name.trim().is_empty() {
            return Err("请输入连接名称。".to_string());
        }
        if driver != Driver::Sqlite && host.trim().is_empty() {
            return Err("请输入主机。".to_string());
        }
        if database.trim().is_empty() {
            return Err("请输入数据库名或 SQLite 文件路径。".to_string());
        }
        Ok(DatasourceInfo {
            driver,
            name,
            host,
            port: read_input(&self.port, cx).parse::<u16>().ok(),
            username: empty_to_none(read_input(&self.username, cx)),
            password: empty_to_none(read_input(&self.password, cx)),
            database: Some(database),
        })
    }

    fn apply(
        &self,
        info: &DatasourceInfo,
        window: &mut Window,
        cx: &mut Context<FakeDataGenerator>,
    ) {
        self.name.update(cx, |state, cx| {
            state.set_value(info.name.clone(), window, cx);
        });
        self.host.update(cx, |state, cx| {
            state.set_value(info.host.clone(), window, cx);
        });
        self.port.update(cx, |state, cx| {
            state.set_value(
                info.port.map(|port| port.to_string()).unwrap_or_default(),
                window,
                cx,
            );
        });
        self.database.update(cx, |state, cx| {
            state.set_value(info.database.clone().unwrap_or_default(), window, cx);
        });
        self.username.update(cx, |state, cx| {
            state.set_value(info.username.clone().unwrap_or_default(), window, cx);
        });
        self.password.update(cx, |state, cx| {
            state.set_value(info.password.clone().unwrap_or_default(), window, cx);
        });
    }
}

// ── 异步操作 ──

async fn load_table_previews(info: DatasourceInfo) -> Result<Vec<TablePreview>, String> {
    let tree = database::database_table_tree(info)
        .await
        .map_err(|err| err.to_string())?;
    let value = serde_json::to_value(tree).map_err(|err| err.to_string())?;
    let mut tables = Vec::new();
    for table in value.as_array().cloned().unwrap_or_default() {
        let schema = table
            .get("schema")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string();
        let table_name = table
            .get("tableName")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string();
        let table_comment = table
            .get("tableComment")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string();
        let mut columns = Vec::new();
        for column in table
            .get("children")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default()
        {
            let name = column
                .get("name")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .to_string();
            let column_type = column
                .get("type")
                .map(value_to_type_text)
                .unwrap_or_else(|| "text".to_string());
            let comment = column
                .get("comment")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .to_string();
            let generator =
                datafaker::datafaker_adapter(Some(name.clone()), Some(column_type.clone()))
                    .await
                    .map_err(|err| err.to_string())?;
            columns.push(ColumnPreview {
                name,
                column_type,
                comment,
                generator,
                config: GeneratorConfig::default(),
            });
        }
        tables.push(TablePreview {
            schema,
            table_name,
            table_comment,
            columns,
        });
    }
    Ok(tables)
}

async fn run_fake_data_insert(
    info: DatasourceInfo,
    tables: Vec<TablePreview>,
    row_count: usize,
) -> Result<String, String> {
    let pool = AnyPool::connect(&info.url())
        .await
        .map_err(|err| err.to_string())?;
    let mut logs = vec![format!(
        "开始运行假数据配置: 表数量={}, 每张表行数={row_count}",
        tables.len()
    )];
    let mut inserted_rows = 0usize;
    let mut failed_rows = 0usize;

    for table in tables {
        if table.columns.is_empty() {
            logs.push(format!(
                "跳过表 {}.{}: 未配置字段",
                table.schema, table.table_name
            ));
            continue;
        }

        logs.push(format!(
            "开始处理表 {}.{}，字段数量={}",
            table.schema,
            table.table_name,
            table.columns.len()
        ));
        let columns = table
            .columns
            .iter()
            .map(|column| quote_identifier(&info.driver, &column.name))
            .collect::<Vec<_>>();
        let mut row_index = 0usize;
        while row_index < row_count {
            let batch_size = INSERT_BATCH_SIZE.min(row_count - row_index);
            let sql = batch_insert_sql(&info.driver, &table, &columns, batch_size);
            match sqlx::query(&sql).execute(&pool).await {
                Ok(_) => {
                    inserted_rows += batch_size;
                    logs.push(format!(
                        "批量插入成功: {}.{} 第 {}-{} 行",
                        table.schema,
                        table.table_name,
                        row_index + 1,
                        row_index + batch_size
                    ));
                }
                Err(error) => {
                    logs.push(format!(
                        "批量插入失败: {}.{} 第 {}-{} 行: {}，回退逐行插入定位错误",
                        table.schema,
                        table.table_name,
                        row_index + 1,
                        row_index + batch_size,
                        error
                    ));
                    for offset in 0..batch_size {
                        let sql = insert_sql(&info.driver, &table, &columns);
                        match sqlx::query(&sql).execute(&pool).await {
                            Ok(_) => {
                                inserted_rows += 1;
                                logs.push(format!(
                                    "插入成功: {}.{} 第 {} 行",
                                    table.schema,
                                    table.table_name,
                                    row_index + offset + 1
                                ));
                            }
                            Err(error) => {
                                failed_rows += 1;
                                logs.push(format!(
                                    "插入失败: {}.{} 第 {} 行: {}",
                                    table.schema,
                                    table.table_name,
                                    row_index + offset + 1,
                                    error
                                ));
                                logs.push(format!("失败 SQL: {sql}"));
                                logs.push(format!(
                                    "运行结束: 成功插入 {inserted_rows} 行，失败 {failed_rows} 行"
                                ));
                                return Ok(logs.join("\n"));
                            }
                        }
                    }
                }
            }
            row_index += batch_size;
        }
        logs.push(format!(
            "{}.{}: 成功 {} 行，失败 0 行",
            table.schema, table.table_name, row_count
        ));
    }

    logs.push(format!(
        "运行结束: 成功插入 {inserted_rows} 行，失败 {failed_rows} 行"
    ));
    Ok(logs.join("\n"))
}

/// 单表假数据插入（用于逐表实时推送日志）
async fn run_fake_data_insert_single(
    info: &DatasourceInfo,
    table: &TablePreview,
    row_count: usize,
) -> Result<Vec<String>, String> {
    let pool = AnyPool::connect(&info.url())
        .await
        .map_err(|err| err.to_string())?;
    let mut logs = Vec::new();

    if table.columns.is_empty() {
        logs.push(format!("跳过表 {}.{}: 未配置字段", table.schema, table.table_name));
        return Ok(logs);
    }

    logs.push(format!(
        "表 {}.{} 字段数量={}",
        table.schema, table.table_name, table.columns.len()
    ));
    let columns = table
        .columns
        .iter()
        .map(|column| quote_identifier(&info.driver, &column.name))
        .collect::<Vec<_>>();
    let mut inserted_rows = 0usize;
    let mut failed_rows = 0usize;
    let mut row_index = 0usize;
    while row_index < row_count {
        let batch_size = INSERT_BATCH_SIZE.min(row_count - row_index);
        let sql = batch_insert_sql(&info.driver, table, &columns, batch_size);
        match sqlx::query(&sql).execute(&pool).await {
            Ok(_) => {
                inserted_rows += batch_size;
                logs.push(format!(
                    "批量插入成功: {}.{} 第 {}-{} 行",
                    table.schema, table.table_name, row_index + 1, row_index + batch_size
                ));
            }
            Err(error) => {
                logs.push(format!(
                    "批量插入失败: {}.{} 第 {}-{} 行: {}，回退逐行插入",
                    table.schema, table.table_name, row_index + 1, row_index + batch_size, error
                ));
                for offset in 0..batch_size {
                    let sql = insert_sql(&info.driver, table, &columns);
                    match sqlx::query(&sql).execute(&pool).await {
                        Ok(_) => {
                            inserted_rows += 1;
                        }
                        Err(error) => {
                            failed_rows += 1;
                            logs.push(format!(
                                "插入失败: {}.{} 第 {} 行: {}",
                                table.schema, table.table_name, row_index + offset + 1, error
                            ));
                        }
                    }
                }
            }
        }
        row_index += batch_size;
    }
    logs.push(format!(
        "{}.{}: 成功 {} 行，失败 {} 行",
        table.schema, table.table_name, inserted_rows, failed_rows
    ));
    Ok(logs)
}

// ── SQL 生成 ──

fn batch_insert_sql(
    driver: &Driver,
    table: &TablePreview,
    columns: &[String],
    batch_size: usize,
) -> String {
    let rows = (0..batch_size)
        .map(|_| {
            format!(
                "({})",
                table
                    .columns
                    .iter()
                    .map(|column| sql_value(generate_column_value(column)))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })
        .collect::<Vec<_>>();

    format!(
        "INSERT INTO {} ({}) VALUES {}",
        quote_table(driver, &table.schema, &table.table_name),
        columns.join(", "),
        rows.join(", ")
    )
}

fn insert_sql(driver: &Driver, table: &TablePreview, columns: &[String]) -> String {
    format!(
        "INSERT INTO {} ({}) VALUES ({})",
        quote_table(driver, &table.schema, &table.table_name),
        columns.join(", "),
        table
            .columns
            .iter()
            .map(|column| sql_value(generate_column_value(column)))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn quote_table(driver: &Driver, schema: &str, table: &str) -> String {
    if *driver == Driver::Sqlite || schema.trim().is_empty() {
        return quote_identifier(driver, table);
    }
    format!(
        "{}.{}",
        quote_identifier(driver, schema),
        quote_identifier(driver, table)
    )
}

fn quote_identifier(driver: &Driver, value: &str) -> String {
    match driver {
        Driver::Mysql => format!("`{}`", value.replace('`', "``")),
        Driver::Postgres | Driver::Sqlite => format!("\"{}\"", value.replace('"', "\"\"")),
    }
}

fn sql_value(value: String) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

// ── 辅助函数 ──

fn upsert_datasource(items: &mut Vec<DatasourceInfo>, info: DatasourceInfo) {
    items.retain(|item| item.name != info.name);
    items.insert(0, info);
}

fn form_field(label: &'static str, input: impl IntoElement) -> Div {
    div()
        .flex()
        .items_center()
        .gap_2()
        .child(div().w(px(72.0)).text_sm().child(label))
        .child(div().flex_1().child(input))
}

/// 表单行：label(140px) + Input(Entity<InputState>)（对齐抽屉内配置面板标签宽度）
fn form_field_entity(label: &str, state: &Entity<InputState>) -> Div {
    div()
        .flex()
        .items_center()
        .gap_2()
        .child(div().w(px(140.0)).text_sm().child(label.to_string()))
        .child(div().flex_1().child(Input::new(state)))
}

fn read_input(state: &Entity<InputState>, cx: &App) -> String {
    state.read(cx).value().to_string()
}

fn empty_to_none(value: String) -> Option<String> {
    if value.trim().is_empty() {
        None
    } else {
        Some(value)
    }
}

fn table_key(schema: &str, table_name: &str) -> String {
    format!("{schema}.{table_name}")
}

fn value_to_type_text(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(text) => text.clone(),
        serde_json::Value::Object(map) => map
            .keys()
            .next()
            .cloned()
            .unwrap_or_else(|| "text".to_string()),
        _ => value.to_string(),
    }
}

fn generate_column_value(column: &ColumnPreview) -> String {
    let value = generate_preview_value(&column.generator);
    match character_length_limit(&column.column_type) {
        Some(limit) => truncate_chars(&value, limit),
        None => value,
    }
}

fn character_length_limit(column_type: &str) -> Option<usize> {
    let lower = column_type.to_ascii_lowercase();
    if ![
        "character varying",
        "varchar",
        "char",
        "bpchar",
        "character",
    ]
    .iter()
    .any(|name| lower.contains(name))
    {
        return None;
    }

    let start = lower.find('(')?;
    let end = lower[start + 1..].find(')')? + start + 1;
    lower[start + 1..end].trim().parse::<usize>().ok()
}

fn truncate_chars(value: &str, limit: usize) -> String {
    value.chars().take(limit).collect()
}

fn generate_preview_value(generator: &str) -> String {
    let faker = datafaker::Faker::new();
    let mut rng = rand::rng();
    match generator {
        "name" => faker.name().full_name(),
        "email" => faker.internet().standard_generic_email(),
        "mobile" => faker.person().mobile(),
        "phone" => format!(
            "0{}-{}",
            rng.random_range(10..999),
            rng.random_range(1000000..99999999)
        ),
        "username" => faker.internet().username(),
        "password" => faker.person().strong_password(),
        "hostname" => faker.internet().domain(),
        "ip" => faker.internet().ipv4(),
        "mac" => faker.internet().mac(),
        "website" => faker.internet().static_url(),
        "uuid" => uuid::Uuid::new_v4().to_string(),
        "number" => rng.random_range(1..100000).to_string(),
        "money" => format!(
            "{:.2}",
            rng.random_range(1..100000) as f64 + rng.random::<f64>()
        ),
        "boolean" => rng.random_bool(0.5).to_string(),
        "date" => "2026-07-12".to_string(),
        "datetime" => "2026-07-12 12:00:00".to_string(),
        "time" => "12:00:00".to_string(),
        "timestamp" => "1783848000".to_string(),
        "address" => faker.address().full_address(),
        "city" => faker.address().city(),
        "company" => faker.name().last_name(),
        "text" => format!("测试文本{}", rng.random_range(1000..9999)),
        "json" => {
            serde_json::json!({"enabled": true, "score": rng.random_range(1..100)}).to_string()
        }
        "id_card" => faker.person().id_card(),
        "gender" => faker.person().gender(),
        "enum" => "active".to_string(),
        _ => format!("value_{}", rng.random_range(1000..9999)),
    }
}

fn csv_escape(value: &str) -> String {
    if value.contains([',', '"', '\n']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

// ── 弹窗内容 ──

fn canvas_load_dialog_content(
    configs: Vec<config_store::DatafakerConfigRecord>,
    target: WeakEntity<FakeDataGenerator>,
    cx: &mut App,
) -> Div {
    let mut content = div().flex().flex_col().gap_3();

    if configs.is_empty() {
        return content.child(div().text_sm().child("暂无画布配置，请先保存配置。"));
    }

    for (index, config) in configs.into_iter().enumerate() {
        let name = config.name.clone();
        let nodes_json = config.nodes_json.clone();
        let updated_at = config.updated_at;
        let target_apply = target.clone();
        let target_delete = target.clone();
        let name_apply = name.clone();
        let name_delete = name.clone();
        content = content.child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .gap_3()
                .border_1()
                .border_color(cx.theme().border)
                .rounded_md()
                .p_2()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(div().text_sm().font_semibold().child(name.clone()))
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(format!("更新时间：{updated_at}")),
                        ),
                )
                .child(
                    ButtonGroup::new(("canvas-config-actions", index))
                        .child(
                            Button::new(("apply-canvas-config", index))
                                .icon(Icon::new(IconName::ArrowRight))
                                .tooltip("应用配置")
                                .on_click(move |_, window, cx| {
                                    let target = target_apply.clone();
                                    let nodes_json = nodes_json.clone();
                                    let name = name_apply.clone();
                                    let window_handle = window.window_handle();
                                    let _ = window_handle.update(cx, move |_, window, cx| {
                                        if let Some(target) = target.upgrade() {
                                            let _ = target.update(cx, |this, cx| {
                                                this.apply_canvas_config(
                                                    nodes_json.clone(),
                                                    name.clone(),
                                                    window,
                                                    cx,
                                                );
                                            });
                                        }
                                    });
                                    window.close_dialog(cx);
                                }),
                        )
                        .child(
                            Button::new(("delete-canvas-config", index))
                                .icon(Icon::new(IconName::Delete))
                                .tooltip("删除配置")
                                .on_click(move |_, window, cx| {
                                    let target = target_delete.clone();
                                    let name = name_delete.clone();
                                    let window_handle = window.window_handle();
                                    let _ = window_handle.update(cx, move |_, _window, cx| {
                                        if let Some(target) = target.upgrade() {
                                            let _ = target.update(cx, |this, cx| {
                                                this.delete_canvas_config(name.clone(), cx);
                                            });
                                        }
                                    });
                                    window.close_dialog(cx);
                                }),
                        ),
                ),
        );
    }

    content
}

fn datasource_dialog_content(
    datasources: Vec<DatasourceInfo>,
    target: WeakEntity<FakeDataGenerator>,
    cx: &mut App,
) -> Div {
    let mut content = div().flex().flex_col().gap_3();

    if datasources.is_empty() {
        return content.child(
            div()
                .text_sm()
                .child("暂无已保存连接，请先保存连接或点击页面上的刷新连接列表。"),
        );
    }

    for (index, datasource) in datasources.into_iter().enumerate() {
        let name = datasource.name.clone();
        let description = format!(
            "{} | {} | {}",
            config_store::driver_label(&datasource.driver),
            datasource.host,
            datasource.database.clone().unwrap_or_default()
        );
        let target = target.clone();
        content = content.child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .gap_3()
                .border_1()
                .border_color(cx.theme().border)
                .rounded_md()
                .p_2()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(div().text_sm().font_semibold().child(name))
                        .child(div().text_xs().child(description)),
                )
                .child(
                    Button::new(("dialog-apply-datasource", index))
                        .icon(Icon::new(IconName::ArrowRight))
                        .tooltip("应用连接")
                        .on_click(move |_, window, cx| {
                            let info = datasource.clone();
                            let target = target.clone();
                            let window_handle = window.window_handle();
                            let _ = window_handle.update(cx, move |_, window, cx| {
                                if let Some(target) = target.upgrade() {
                                    let _ = target.update(cx, |this, cx| {
                                        let name = info.name.clone();
                                        this.apply_datasource(info, window, cx);
                                        this.status = format!("已应用连接 {name}。");
                                        cx.notify();
                                    });
                                }
                            });
                            window.close_dialog(cx);
                        }),
                ),
        );
    }

    content
}

fn build_generator_select_items() -> SearchableVec<SelectGroup<GeneratorSelectItem>> {
    let groups = generator_groups();
    let select_groups: Vec<SelectGroup<GeneratorSelectItem>> = groups
        .into_iter()
        .map(|group| {
            SelectGroup::new(group.label)
                .items(group.children.into_iter().map(|item| GeneratorSelectItem {
                    label: item.label.to_string(),
                    value: item.value.to_string(),
                }))
        })
        .collect();
    SearchableVec::new(select_groups)
}

#[derive(Clone)]
struct GeneratorSelectItem {
    label: String,
    value: String,
}

impl gpui_component::select::SelectItem for GeneratorSelectItem {
    type Value = String;
    fn title(&self) -> gpui::SharedString {
        self.label.clone().into()
    }
    fn value(&self) -> &Self::Value {
        &self.value
    }
    fn matches(&self, query: &str) -> bool {
        self.label.to_lowercase().contains(&query.to_lowercase())
            || self.value.to_lowercase().contains(&query.to_lowercase())
    }
}

struct GeneratorGroup {
    label: &'static str,
    children: Vec<GeneratorItem>,
}

struct GeneratorItem {
    label: &'static str,
    value: &'static str,
}

fn generator_groups() -> Vec<GeneratorGroup> {
    vec![
        GeneratorGroup {
            label: "日期时间",
            children: vec![
                GeneratorItem { label: "日期", value: "date" },
                GeneratorItem { label: "日期时间", value: "datetime" },
                GeneratorItem { label: "时间", value: "time" },
                GeneratorItem { label: "时间戳", value: "timestamp" },
                GeneratorItem { label: "时区名称", value: "timezone" },
            ],
        },
        GeneratorGroup {
            label: "地理位置",
            children: vec![
                GeneratorItem { label: "省份和城市", value: "province_city" },
                GeneratorItem { label: "国家或地区", value: "country_region" },
                GeneratorItem { label: "经纬度", value: "latitude_longitude" },
                GeneratorItem { label: "邮编", value: "zip_code" },
                GeneratorItem { label: "地址", value: "address" },
                GeneratorItem { label: "城市", value: "city" },
                GeneratorItem { label: "省/州", value: "state" },
                GeneratorItem { label: "街道地址", value: "street_address" },
                GeneratorItem { label: "固话区号", value: "phone_area_code" },
                GeneratorItem { label: "固定电话", value: "phone" },
            ],
        },
        GeneratorGroup {
            label: "教育",
            children: vec![
                GeneratorItem { label: "学历", value: "degree" },
                GeneratorItem { label: "小学名称", value: "primary_school" },
                GeneratorItem { label: "小学年级", value: "primary_school_grade" },
                GeneratorItem { label: "中学名称", value: "high_school" },
                GeneratorItem { label: "中学年级", value: "high_school_grade" },
                GeneratorItem { label: "班级", value: "school_class" },
                GeneratorItem { label: "大学", value: "college" },
                GeneratorItem { label: "专业", value: "major" },
            ],
        },
        GeneratorGroup {
            label: "金融",
            children: vec![
                GeneratorItem { label: "金额", value: "money" },
                GeneratorItem { label: "股票名称+股票代码", value: "stock" },
                GeneratorItem { label: "日K线数据", value: "stock_kline" },
                GeneratorItem { label: "基金名称+基金代码", value: "fund" },
                GeneratorItem { label: "货币信息", value: "currency" },
                GeneratorItem { label: "银行卡号", value: "bank_card" },
                GeneratorItem { label: "付款方式", value: "payment_method" },
                GeneratorItem { label: "信用卡类型", value: "credit_card_type" },
                GeneratorItem { label: "信用卡卡号", value: "credit_card_number" },
                GeneratorItem { label: "信用卡日期", value: "credit_card_date" },
            ],
        },
        GeneratorGroup {
            label: "个人",
            children: vec![
                GeneratorItem { label: "姓名", value: "name" },
                GeneratorItem { label: "性别", value: "gender" },
                GeneratorItem { label: "手机号", value: "mobile" },
                GeneratorItem { label: "身份证号", value: "id_card" },
                GeneratorItem { label: "用户名", value: "username" },
                GeneratorItem { label: "密码", value: "password" },
                GeneratorItem { label: "QQ号", value: "qq" },
                GeneratorItem { label: "昵称", value: "nickname" },
                GeneratorItem { label: "民族", value: "ethnicity" },
                GeneratorItem { label: "职位", value: "job_title" },
            ],
        },
        GeneratorGroup {
            label: "商业",
            children: vec![
                GeneratorItem { label: "公司", value: "company" },
                GeneratorItem { label: "部门", value: "department" },
                GeneratorItem { label: "行业", value: "industry" },
            ],
        },
        GeneratorGroup {
            label: "产品",
            children: vec![
                GeneratorItem { label: "产品名称", value: "product_name" },
                GeneratorItem { label: "产品类别", value: "product_category" },
                GeneratorItem { label: "颜色", value: "color" },
                GeneratorItem { label: "尺寸", value: "size" },
                GeneratorItem { label: "重量单位", value: "weight_unit" },
                GeneratorItem { label: "条码", value: "barcode" },
                GeneratorItem { label: "SKU", value: "sku" },
            ],
        },
        GeneratorGroup {
            label: "互联网",
            children: vec![
                GeneratorItem { label: "邮箱", value: "email" },
                GeneratorItem { label: "主机名", value: "hostname" },
                GeneratorItem { label: "IP地址", value: "ip" },
                GeneratorItem { label: "MAC地址", value: "mac" },
                GeneratorItem { label: "网址", value: "website" },
                GeneratorItem { label: "文件扩展名", value: "file_extension" },
                GeneratorItem { label: "文件名", value: "file_name" },
                GeneratorItem { label: "文件路径", value: "file_path" },
                GeneratorItem { label: "App Bundle ID", value: "app_bundle_id" },
                GeneratorItem { label: "应用名", value: "app_name" },
                GeneratorItem { label: "应用版本", value: "app_version" },
                GeneratorItem { label: "User-Agent", value: "user_agent" },
                GeneratorItem { label: "端口", value: "port" },
            ],
        },
        GeneratorGroup {
            label: "其它",
            children: vec![
                GeneratorItem { label: "数字", value: "number" },
                GeneratorItem { label: "布尔值", value: "boolean" },
                GeneratorItem { label: "JSON", value: "json" },
                GeneratorItem { label: "汉字", value: "chinese_char" },
                GeneratorItem { label: "成语", value: "idiom" },
                GeneratorItem { label: "车牌号", value: "license_plate" },
                GeneratorItem { label: "热门手机型号", value: "mobile_model" },
                GeneratorItem { label: "统一社会信用代码", value: "unified_social_credit_code" },
                GeneratorItem { label: "数据生成工具", value: "data_tool" },
                GeneratorItem { label: "序列", value: "sequence" },
                GeneratorItem { label: "枚举", value: "enum" },
                GeneratorItem { label: "文本", value: "text" },
                GeneratorItem { label: "图像或二进制", value: "binary" },
                GeneratorItem { label: "外键", value: "foreign_key" },
                GeneratorItem { label: "UUID", value: "uuid" },
                GeneratorItem { label: "正则表达式", value: "regex" },
            ],
        },
    ]
}
