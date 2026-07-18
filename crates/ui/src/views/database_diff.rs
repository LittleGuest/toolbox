use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use database::{CheckReportBo, DatasourceInfo, DiffReport, Driver};
use gpui::{prelude::FluentBuilder as _, *};
use gpui_component::{
    WindowExt,
    button::*,
    checkbox::Checkbox,
    input::{Input, InputState},
    radio::{Radio, RadioGroup},
    scroll::ScrollableElement,
    select::{Select, SelectEvent, SelectState},
    tab::{Tab, TabBar},
    *,
};

use crate::config_store;

pub struct DatabaseDiff {
    report_output: Option<DiffReport>,
    sql_output: String,
    check_output: Option<Vec<CheckReportBo>>,
    code_output: String,
    /// 逆向生成：每个文件名 → 代码内容（保持插入顺序）
    generated_codes: Vec<(String, String)>,
    /// 逆向生成：当前激活的 tab 索引
    code_active_tab: usize,
    table_output: String,
    status: String,
    is_running: bool,
    saved_datasources: Vec<DatasourceInfo>,
    /// 差异报告行：基准库 / 变动库
    report_source_select: Entity<SelectState<Vec<String>>>,
    report_target_select: Entity<SelectState<Vec<String>>>,
    /// 差异SQL行：基准库 / 变动库
    sql_source_select: Entity<SelectState<Vec<String>>>,
    sql_target_select: Entity<SelectState<Vec<String>>>,
    /// 规范检查行：基准库
    check_source_select: Entity<SelectState<Vec<String>>>,
    /// 逆向生成：数据源选择
    code_datasource_select: Entity<SelectState<Vec<String>>>,
    /// 逆向生成：表选项列表 (key=schema.tableName, label=带注释的显示文本)
    code_table_options: Vec<(String, String)>,
    /// 逆向生成：已选中的表
    code_selected_tables: HashSet<String>,
    /// 逆向生成：表加载消息
    code_table_load_msg: String,
    code_language: String,
    code_file_types: Vec<String>,
    /// 逆向生成：语言 RadioGroup 当前选中索引（0=Rust, 1=Java）
    code_language_index: usize,
    entity_package_state: Entity<InputState>,
    mapper_package_state: Entity<InputState>,
    service_package_state: Entity<InputState>,
    service_impl_package_state: Entity<InputState>,
    controller_package_state: Entity<InputState>,
    /// 连接管理抽屉的表单
    conn_form: DbForm,
    /// 连接管理抽屉的驱动选择
    conn_driver: String,
    conn_driver_state: Entity<SelectState<Vec<String>>>,
    /// 编辑中的连接名称（None=新建）
    conn_edit_name: Option<String>,
    _subscriptions: Vec<Subscription>,
}

struct DbForm {
    name: Entity<InputState>,
    host: Entity<InputState>,
    port: Entity<InputState>,
    database: Entity<InputState>,
    username: Entity<InputState>,
    password: Entity<InputState>,
}

#[derive(Clone)]
struct StandardCheckOption {
    code: i32,
    desc: String,
}

impl DatabaseDiff {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let empty_options: Vec<String> = Vec::new();

        let report_source_select =
            cx.new(|cx| SelectState::new(empty_options.clone(), None, window, cx));
        let report_target_select =
            cx.new(|cx| SelectState::new(empty_options.clone(), None, window, cx));
        let sql_source_select =
            cx.new(|cx| SelectState::new(empty_options.clone(), None, window, cx));
        let sql_target_select =
            cx.new(|cx| SelectState::new(empty_options.clone(), None, window, cx));
        let check_source_select =
            cx.new(|cx| SelectState::new(empty_options.clone(), None, window, cx));
        let code_datasource_select =
            cx.new(|cx| SelectState::new(empty_options.clone(), None, window, cx));

        let conn_form = DbForm::new(window, cx);
        let conn_driver_state = cx.new(|cx| {
            let mut state = SelectState::new(
                vec![
                    "PostgreSQL".to_string(),
                    "MySQL".to_string(),
                    "SQLite".to_string(),
                ],
                None,
                window,
                cx,
            );
            state.set_selected_value(&"PostgreSQL".to_string(), window, cx);
            state
        });

        let entity_package_state = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("com.example.entity")
                .default_value("com.example.entity".to_string())
        });
        let mapper_package_state = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("com.example.mapper")
                .default_value("com.example.mapper".to_string())
        });
        let service_package_state = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("com.example.service")
                .default_value("com.example.service".to_string())
        });
        let service_impl_package_state = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("com.example.service.impl")
                .default_value("com.example.service.impl".to_string())
        });
        let controller_package_state = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("com.example.controller")
                .default_value("com.example.controller".to_string())
        });

        let _subscriptions = vec![
            cx.subscribe_in(
                &conn_driver_state,
                window,
                |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                    if let SelectEvent::Confirm(Some(value)) = ev {
                        this.conn_driver = value.clone();
                        cx.notify();
                    }
                },
            ),
            cx.subscribe_in(
                &report_source_select,
                window,
                |_, _, _: &SelectEvent<Vec<String>>, _, cx| {
                    cx.notify();
                },
            ),
            cx.subscribe_in(
                &report_target_select,
                window,
                |_, _, _: &SelectEvent<Vec<String>>, _, cx| {
                    cx.notify();
                },
            ),
            cx.subscribe_in(
                &sql_source_select,
                window,
                |_, _, _: &SelectEvent<Vec<String>>, _, cx| {
                    cx.notify();
                },
            ),
            cx.subscribe_in(
                &sql_target_select,
                window,
                |_, _, _: &SelectEvent<Vec<String>>, _, cx| {
                    cx.notify();
                },
            ),
            cx.subscribe_in(
                &check_source_select,
                window,
                |_, _, _: &SelectEvent<Vec<String>>, _, cx| {
                    cx.notify();
                },
            ),
            cx.subscribe_in(
                &code_datasource_select,
                window,
                |this, _, ev: &SelectEvent<Vec<String>>, _, cx| {
                    if let SelectEvent::Confirm(_) = ev {
                        this.on_code_datasource_change(cx);
                    }
                },
            ),
        ];

        // 初始化时自动加载已保存的数据源连接
        cx.spawn_in(window, async move |this: WeakEntity<Self>, cx| {
            let result = config_store::load_datasources().await;
            let _ = this.update_in(cx, |this, window, cx| {
                match result {
                    Ok(datasources) => {
                        this.status = format!("已加载 {} 个已保存连接。", datasources.len());
                        this.saved_datasources = datasources.clone();
                        this.refresh_select_options(&datasources, window, cx);
                    }
                    Err(err) => {
                        this.status = format!("加载连接列表失败：{err}");
                    }
                }
                cx.notify();
            });
        })
        .detach();

        Self {
            report_output: None,
            sql_output: String::new(),
            check_output: None,
            code_output: String::new(),
            generated_codes: Vec::new(),
            code_active_tab: 0,
            table_output: String::new(),
            status: "请选择基准库和变动库连接。".to_string(),
            is_running: false,
            saved_datasources: Vec::new(),
            report_source_select,
            report_target_select,
            sql_source_select,
            sql_target_select,
            check_source_select,
            code_datasource_select,
            code_table_options: Vec::new(),
            code_selected_tables: HashSet::new(),
            code_table_load_msg: String::new(),
            code_language: "java".to_string(),
            code_file_types: default_file_types("java"),
            code_language_index: 0,
            entity_package_state,
            mapper_package_state,
            service_package_state,
            service_impl_package_state,
            controller_package_state,
            conn_form,
            conn_driver: "PostgreSQL".to_string(),
            conn_driver_state,
            conn_edit_name: None,
            _subscriptions,
        }
    }

    /// 从 SelectState 读取选中连接，再从 saved_datasources 中按标签查找
    fn selected_datasource(
        &self,
        select: &Entity<SelectState<Vec<String>>>,
        cx: &App,
    ) -> Option<DatasourceInfo> {
        let selected_label = select.read(cx).selected_value()?.to_string();
        self.saved_datasources
            .iter()
            .find(|ds| datasource_label(ds) == selected_label)
            .cloned()
    }

    /// 刷新所有 Select 的选项列表
    fn refresh_select_options(
        &mut self,
        datasources: &[DatasourceInfo],
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let options = datasource_options(datasources);
        for select in [
            &self.report_source_select,
            &self.report_target_select,
            &self.sql_source_select,
            &self.sql_target_select,
            &self.check_source_select,
            &self.code_datasource_select,
        ] {
            select.update(cx, |state, cx| {
                state.set_items(options.clone(), window, cx);
            });
        }
    }

    /// 打开连接管理抽屉（新建或编辑）
    fn open_conn_sheet(
        &mut self,
        edit_name: Option<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.conn_edit_name = edit_name.clone();

        if let Some(ref name) = edit_name {
            // 编辑模式：从已保存列表加载数据
            if let Some(ds) = self.saved_datasources.iter().find(|d| &d.name == name) {
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
                        .child(field("类型", Select::new(&driver_state)))
                        .child(field("连接名称", Input::new(&form_name)))
                        .child(field("主机", Input::new(&form_host)))
                        .child(field("端口", Input::new(&form_port)))
                        .child(field("用户名", Input::new(&form_username)))
                        .child(field("密码", Input::new(&form_password).mask_toggle()))
                        .child(field("数据库", Input::new(&form_database)))
                        .child(
                            div()
                                .flex()
                                .justify_end()
                                .gap_2()
                                .mt_2()
                                .child({
                                    let weak = weak.clone();
                                    Button::new("conn-sheet-ping")
                                        .icon(Icon::new(IconName::ArrowRight))
                                        .tooltip("测试连接")
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
                                        .primary()
                                        .icon(Icon::new(IconName::Check))
                                        .tooltip("保存")
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

    /// 从连接管理表单保存连接
    fn save_conn(&mut self, window: &mut Window, cx: &mut Context<Self>) {
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

        cx.spawn_in(window, async move |this: WeakEntity<Self>, cx| {
            // save_datasource 内部用 ON CONFLICT(name) 做 upsert，新建和编辑统一走此路径
            let result = config_store::save_datasource(info).await;
            let _ = this.update_in(cx, |this, window, cx| {
                this.status = match result {
                    Ok(_) => {
                        upsert_datasource(&mut this.saved_datasources, saved_info);
                        this.refresh_select_options(&this.saved_datasources.clone(), window, cx);
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

    /// 从连接管理表单测试连接
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

    fn ping_by_name(&mut self, name: String, cx: &mut Context<Self>) {
        let Some(ds) = self.saved_datasources.iter().find(|d| d.name == name) else {
            self.status = format!("未找到连接 {name}");
            cx.notify();
            return;
        };
        let info = ds.clone();
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

    fn refresh_datasources(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.status = "正在刷新连接列表...".to_string();
        cx.notify();

        cx.spawn_in(window, async move |this: WeakEntity<Self>, cx| {
            let result = config_store::load_datasources().await;
            let _ = this.update_in(cx, |this, window, cx| {
                match result {
                    Ok(datasources) => {
                        this.status = format!("已加载 {} 个已保存连接。", datasources.len());
                        this.saved_datasources = datasources.clone();
                        this.refresh_select_options(&datasources, window, cx);
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

    /// 根据名称删除已保存连接
    fn delete_datasource_by_name(
        &mut self,
        name: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.status = format!("正在删除连接 {name}...");
        cx.notify();

        cx.spawn_in(window, async move |this: WeakEntity<Self>, cx| {
            let result = config_store::delete_datasource(name.clone()).await;
            let _ = this.update_in(cx, |this, window, cx| {
                this.status = match result {
                    Ok(true) => {
                        this.saved_datasources.retain(|item| item.name != name);
                        this.refresh_select_options(&this.saved_datasources.clone(), window, cx);
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

    fn load_tables(&mut self, cx: &mut Context<Self>) {
        let source = self.selected_datasource(&self.report_source_select, cx);
        let target = self.selected_datasource(&self.report_target_select, cx);
        let (Some(source), Some(target)) = (source, target) else {
            self.status = "请先选择基准库和变动库连接。".to_string();
            cx.notify();
            return;
        };

        self.is_running = true;
        self.status = "正在加载数据源表...".to_string();
        cx.notify();

        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            let source_tables = database::database_tables(source).await;
            let target_tables = database::database_tables(target).await;
            let output = match (source_tables, target_tables) {
                (Ok(source_tables), Ok(target_tables)) => {
                    let source_lines = source_tables
                        .into_iter()
                        .map(|table| {
                            format!(
                                "基准库 | {}.{} | {}",
                                table.schema, table.name, table.comment
                            )
                        })
                        .collect::<Vec<_>>();
                    let target_lines = target_tables
                        .into_iter()
                        .map(|table| {
                            format!(
                                "变动库 | {}.{} | {}",
                                table.schema, table.name, table.comment
                            )
                        })
                        .collect::<Vec<_>>();
                    source_lines
                        .into_iter()
                        .chain(target_lines)
                        .collect::<Vec<_>>()
                        .join("\n")
                }
                (Err(err), _) => format!("加载基准库表失败：{err}"),
                (_, Err(err)) => format!("加载变动库表失败：{err}"),
            };

            let _ = this.update(cx, |this, cx| {
                this.is_running = false;
                this.table_output = output;
                this.status = "数据源表加载完成".to_string();
                cx.notify();
            });
        })
        .detach();
    }

    fn generate_report(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let source = self.selected_datasource(&self.report_source_select, cx);
        let target = self.selected_datasource(&self.report_target_select, cx);
        let (Some(source), Some(target)) = (source, target) else {
            self.status = "请先选择基准库和变动库连接。".to_string();
            cx.notify();
            return;
        };

        self.is_running = true;
        self.status = "正在生成差异报告...".to_string();
        cx.notify();

        cx.spawn_in(window, async move |this: WeakEntity<Self>, cx| {
            let result = database::diff_report(source, target).await;
            let _ = this.update_in(cx, |this, window, cx| {
                this.is_running = false;
                match result {
                    Ok(report) => {
                        this.report_output = Some(report);
                        this.status = "差异报告生成完成".to_string();
                        this.open_report_drawer(window, cx);
                    }
                    Err(err) => this.status = format!("差异报告生成失败：{err}"),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn generate_sql(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let source = self.selected_datasource(&self.sql_source_select, cx);
        let target = self.selected_datasource(&self.sql_target_select, cx);
        let (Some(source), Some(target)) = (source, target) else {
            self.status = "请先选择基准库和变动库连接。".to_string();
            cx.notify();
            return;
        };

        self.is_running = true;
        self.status = "正在生成差异 SQL...".to_string();
        cx.notify();

        cx.spawn_in(window, async move |this: WeakEntity<Self>, cx| {
            let result = database::diff_sql(source, target).await;
            let _ = this.update_in(cx, |this, window, cx| {
                this.is_running = false;
                match result {
                    Ok(sqls) => {
                        this.sql_output = sqls.join("\n\n");
                        this.status = "差异 SQL 生成完成".to_string();
                        this.open_sql_drawer(window, cx);
                    }
                    Err(err) => this.status = format!("差异 SQL 生成失败：{err}"),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn open_standard_check_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let source = self.selected_datasource(&self.check_source_select, cx);
        let Some(source) = source else {
            self.status = "请先选择基准库连接。".to_string();
            cx.notify();
            return;
        };

        self.status = "正在加载规范检查项...".to_string();
        cx.notify();

        cx.spawn_in(window, async move |this: WeakEntity<Self>, cx| {
            let options = database::database_standard_check_codes()
                .await
                .into_iter()
                .filter_map(standard_check_option)
                .collect::<Vec<_>>();
            let _ = this.update_in(cx, |this, window, cx| {
                this.status = format!("已加载 {} 个规范检查项。", options.len());
                this.show_standard_check_dialog(source, options, window, cx);
                cx.notify();
            });
        })
        .detach();
    }

    fn show_standard_check_dialog(
        &mut self,
        source: DatasourceInfo,
        options: Vec<StandardCheckOption>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let this = cx.entity().downgrade();
        // 共享选中状态：抽屉内 checkbox 与「保存」按钮共享
        // 默认不选（对齐 Tauri：customStandardChecked = []）
        let selected: Rc<RefCell<HashSet<i32>>> = Rc::new(RefCell::new(HashSet::new()));

        window.open_sheet_at(Placement::Bottom, cx, move |sheet, _, cx| {
            let weak = this.clone();
            let options_clone = options.clone();
            let selected_clone = selected.clone();
            let source_clone = source.clone();

            // 构建 checkbox 列表
            let mut checklist = div().flex().flex_col().gap_2();
            for (i, opt) in options_clone.iter().enumerate() {
                let is_checked = selected_clone.borrow().contains(&opt.code);
                let opt_code = opt.code;
                let opt_desc = opt.desc.clone();
                let sel = selected_clone.clone();
                let cb = Checkbox::new(("check-opt", i))
                    .label(opt_desc)
                    .checked(is_checked)
                    .on_click(move |checked: &bool, _, _| {
                        let mut sel = sel.borrow_mut();
                        if *checked {
                            sel.insert(opt_code);
                        } else {
                            sel.remove(&opt_code);
                        }
                    });
                checklist = checklist.child(cb);
            }

            sheet
                .overlay(true)
                .overlay_closable(true)
                .size(px(500.))
                .title("自定义规范检查")
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .p_4()
                        .child(checklist)
                        .child(
                            div()
                                .flex()
                                .justify_end()
                                .gap_2()
                                .mt_2()
                                .child(
                                    Button::new("check-cancel")
                                        .label("取消")
                                        .on_click(move |_, window, cx| {
                                            window.close_sheet(cx);
                                        }),
                                )
                                .child({
                                    let weak = weak.clone();
                                    let selected = selected.clone();
                                    let source = source_clone.clone();
                                    Button::new("check-save")
                                        .primary()
                                        .label("保存")
                                        .on_click(move |_, window, cx| {
                                            let codes: Vec<i32> =
                                                selected.borrow().iter().copied().collect();
                                            if let Some(this) = weak.upgrade() {
                                                this.update(cx, |this, cx| {
                                                    this.run_standard_check_with_codes(
                                                        source.clone(),
                                                        codes,
                                                        "自定义检查".to_string(),
                                                        window,
                                                        cx,
                                                    );
                                                });
                                            }
                                            window.close_sheet(cx);
                                        })
                                }),
                        ),
                )
        });
    }

    fn run_standard_check_with_codes(
        &mut self,
        source: DatasourceInfo,
        codes: Vec<i32>,
        label: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if codes.is_empty() {
            self.status = "没有可执行的规范检查项。".to_string();
            cx.notify();
            return;
        }

        self.is_running = true;
        self.status = format!("正在执行规范检查：{label}...");
        cx.notify();

        cx.spawn_in(window, async move |this: WeakEntity<Self>, cx| {
            let result = database::standard_check(source, codes).await;
            let _ = this.update_in(cx, |this, window, cx| {
                this.is_running = false;
                match result {
                    Ok(report) => {
                        this.check_output = Some(report);
                        this.status = format!("规范检查完成：{label}");
                        this.open_check_drawer(window, cx);
                    }
                    Err(err) => this.status = format!("规范检查失败：{err}"),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn on_language_radio_change(&mut self, index: usize, cx: &mut Context<Self>) {
        // 顺序：0=Java, 1=Rust（对齐 Tauri）
        let lang_key = if index == 0 { "java" } else { "rust" };
        if lang_key != self.code_language {
            self.code_language = lang_key.to_string();
            self.code_language_index = index;
            self.code_file_types = default_file_types(lang_key);
            self.generated_codes.clear();
            self.code_active_tab = 0;
            cx.notify();
        }
    }

    /// 逆向生成：数据源切换时重新加载表列表
    fn on_code_datasource_change(&mut self, cx: &mut Context<Self>) {
        let datasource = self.selected_datasource(&self.code_datasource_select, cx);
        let Some(datasource) = datasource else {
            self.code_table_options.clear();
            self.code_selected_tables.clear();
            self.code_table_load_msg = "请先选择数据源".to_string();
            cx.notify();
            return;
        };

        self.code_table_load_msg = "正在加载表...".to_string();
        cx.notify();

        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            let result = database::database_table_tree(datasource).await;
            let _ = this.update(cx, |this, cx| {
                match result {
                    Ok(tables) => {
                        let options = tables
                            .iter()
                            .map(|t| {
                                let key = if t.schema.is_empty() {
                                    t.table_name.clone()
                                } else {
                                    format!("{}.{}", t.schema, t.table_name)
                                };
                                let label = if t.table_comment.is_empty() {
                                    key.clone()
                                } else {
                                    format!("{}（{}）", key, t.table_comment)
                                };
                                (key, label)
                            })
                            .collect::<Vec<_>>();
                        if options.is_empty() {
                            this.code_table_load_msg =
                                "当前数据源未读取到表，请检查连接和数据库权限".to_string();
                        } else {
                            this.code_table_load_msg.clear();
                        }
                        this.code_table_options = options;
                        this.code_selected_tables.clear();
                        this.generated_codes.clear();
                        this.code_active_tab = 0;
                    }
                    Err(err) => {
                        this.code_table_options.clear();
                        this.code_selected_tables.clear();
                        this.code_table_load_msg = format!("加载表失败: {err}");
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn toggle_file_type(&mut self, file_type: &str, cx: &mut Context<Self>) {
        if let Some(pos) = self
            .code_file_types
            .iter()
            .position(|item| item == file_type)
        {
            self.code_file_types.remove(pos);
        } else {
            self.code_file_types.push(file_type.to_string());
        }
        cx.notify();
    }

    fn toggle_table_selection(&mut self, table: &str, cx: &mut Context<Self>) {
        if self.code_selected_tables.contains(table) {
            self.code_selected_tables.remove(table);
        } else {
            self.code_selected_tables.insert(table.to_string());
        }
        cx.notify();
    }

    fn read_package_names(&self, cx: &App) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert(
            "entity.java".to_string(),
            read_input(&self.entity_package_state, cx),
        );
        map.insert(
            "mapper.java".to_string(),
            read_input(&self.mapper_package_state, cx),
        );
        map.insert(
            "service.java".to_string(),
            read_input(&self.service_package_state, cx),
        );
        map.insert(
            "serviceImpl.java".to_string(),
            read_input(&self.service_impl_package_state, cx),
        );
        map.insert(
            "controller.java".to_string(),
            read_input(&self.controller_package_state, cx),
        );
        map
    }

    fn generate_code(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let source = self.selected_datasource(&self.code_datasource_select, cx);
        let Some(source) = source else {
            self.status = "请先选择数据源。".to_string();
            cx.notify();
            return;
        };

        let language = self.code_language.clone();
        let file_types = if self.code_file_types.is_empty() {
            default_file_types(&language)
        } else {
            self.code_file_types.clone()
        };
        let table_names = self.code_selected_tables.iter().cloned().collect::<Vec<_>>();
        let package_names = self.read_package_names(cx);
        let label = language_label(&language);

        self.is_running = true;
        self.status = format!("正在逆向生成 {label} 代码...");
        cx.notify();

        cx.spawn_in(window, async move |this: WeakEntity<Self>, cx| {
            let result = generate_code_async(source, &language, &file_types, &table_names, &package_names).await;
            let _ = this.update_in(cx, |this, window, cx| {
                this.is_running = false;
                match result {
                    Ok(codes) => {
                        if codes.is_empty() {
                            this.status = "没有生成任何文件，请检查语言和文件类型".to_string();
                        } else {
                            this.generated_codes = codes;
                            this.code_active_tab = 0;
                            this.status = "代码生成成功".to_string();
                        }
                    }
                    Err(err) => this.status = format!("逆向生成失败：{err}"),
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// 打开逆向生成代码的抽屉（匹配 Tauri DatabaseGeneratorCode.vue 的右侧抽屉）
    fn open_code_gen_drawer(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let weak = cx.entity().downgrade();
        let datasource_state = self.code_datasource_select.clone();
        let code_language = self.code_language.clone();
        let code_language_index = self.code_language_index;
        let code_file_types = self.code_file_types.clone();
        let code_table_options = self.code_table_options.clone();
        let code_selected_tables = self.code_selected_tables.clone();
        let code_table_load_msg = self.code_table_load_msg.clone();
        let generated_codes = self.generated_codes.clone();
        let code_active_tab = self.code_active_tab;
        let entity_state = self.entity_package_state.clone();
        let mapper_state = self.mapper_package_state.clone();
        let service_state = self.service_package_state.clone();
        let service_impl_state = self.service_impl_package_state.clone();
        let controller_state = self.controller_package_state.clone();
        let is_java = code_language == "java";
        let available_types = available_file_types(&code_language);

        // 共享选中状态：抽屉内 checkbox 与「生成代码」按钮共享
        let file_types_shared: Rc<RefCell<HashSet<String>>> =
            Rc::new(RefCell::new(code_file_types.iter().cloned().collect()));
        let tables_shared: Rc<RefCell<HashSet<String>>> =
            Rc::new(RefCell::new(code_selected_tables.iter().cloned().collect()));

        window.open_sheet_at(Placement::Right, cx, move |sheet, window, cx| {
            let weak2 = weak.clone();

            // 语言 RadioGroup（顺序：Java, Rust，对齐 Tauri）
            let language_radio = RadioGroup::horizontal("code-language")
                .selected_index(Some(code_language_index))
                .on_click(move |index: &usize, _, cx| {
                    if let Some(this) = weak2.upgrade() {
                        this.update(cx, |this, cx| {
                            this.on_language_radio_change(*index, cx);
                        });
                    }
                })
                .child(Radio::new("lang-java").label("Java"))
                .child(Radio::new("lang-rust").label("Rust"));

            // 生成文件类型 checkbox 组
            let mut file_checkboxes = div().flex().flex_wrap().gap_3();
            for (i, ft) in available_types.iter().enumerate() {
                let is_active = file_types_shared.borrow().contains(ft);
                let label = file_type_label(ft);
                let ft_clone = ft.clone();
                let sel = file_types_shared.clone();
                let cb = Checkbox::new(("code-ft", i))
                    .label(label)
                    .checked(is_active)
                    .on_click(move |checked: &bool, _, _| {
                        let mut sel = sel.borrow_mut();
                        if *checked {
                            sel.insert(ft_clone.clone());
                        } else {
                            sel.remove(&ft_clone);
                        }
                    });
                file_checkboxes = file_checkboxes.child(cb);
            }

            // 表多选 checkbox 列表
            let mut table_list = div().flex().flex_col().gap_1().max_h(px(160.)).overflow_y_scrollbar();
            if code_table_options.is_empty() {
                table_list = table_list.child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(if code_table_load_msg.is_empty() {
                            "请先选择数据源".to_string()
                        } else {
                            code_table_load_msg.clone()
                        }),
                );
            } else {
                for (i, (table_key, table_label)) in code_table_options.iter().enumerate() {
                    let is_checked = tables_shared.borrow().contains(table_key);
                    let table_key_clone = table_key.clone();
                    let table_label_clone = table_label.clone();
                    let sel = tables_shared.clone();
                    let cb = Checkbox::new(("code-table", i))
                        .label(table_label_clone)
                        .checked(is_checked)
                        .on_click(move |checked: &bool, _, _| {
                            let mut sel = sel.borrow_mut();
                            if *checked {
                                sel.insert(table_key_clone.clone());
                            } else {
                                sel.remove(&table_key_clone);
                            }
                        });
                    table_list = table_list.child(cb);
                }
            }

            // 生成结果 TabBar
            let weak5 = weak.clone();
            let result_section = if generated_codes.is_empty() {
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("暂无生成结果")
            } else {
                let mut tab_bar = TabBar::new("code-tabs")
                    .underline()
                    .selected_index(code_active_tab)
                    .on_click(move |index: &usize, _, cx| {
                        if let Some(this) = weak5.upgrade() {
                            this.update(cx, |this, cx| {
                                this.code_active_tab = *index;
                                cx.notify();
                            });
                        }
                    });
                for (file_name, _) in generated_codes.iter() {
                    tab_bar = tab_bar.child(Tab::new().label(file_name.clone()));
                }

                let active_code = generated_codes
                    .get(code_active_tab)
                    .map(|(_, code)| code.clone())
                    .unwrap_or_default();
                let active_file = generated_codes
                    .get(code_active_tab)
                    .map(|(file, _)| file.clone())
                    .unwrap_or_default();

                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(tab_bar)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_end()
                            .gap_2()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(file_language_tag(&active_file)),
                            )
                            .child({
                                let code = active_code.clone();
                                Button::new("copy-code-tab")
                                    .label("复制")
                                    .small()
                                    .on_click(move |_, _, cx| {
                                        cx.write_to_clipboard(ClipboardItem::new_string(
                                            code.clone(),
                                        ));
                                    })
                            }),
                    )
                    .child(
                        div()
                            .max_h(px(360.))
                            .overflow_y_scrollbar()
                            .p_3()
                            .bg(rgb(0x0f172a))
                            .rounded_md()
                            .text_sm()
                            .font_family("monospace")
                            .text_color(rgb(0xe2e8f0))
                            .child(active_code.clone()),
                    )
            };

            sheet
                .overlay(true)
                .overlay_closable(true)
                .size(px(900.0))
                .title("数据库代码生成")
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .p_4()
                        .h_full()
                        .overflow_y_scrollbar()
                        .child(field("数据源", Select::new(&datasource_state)))
                        .child(field("语言", language_radio))
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(div().w(px(90.0)).text_sm().child("表"))
                                .child(table_list)
                                .when(!code_table_load_msg.is_empty(), |this| {
                                    this.child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(code_table_load_msg.clone()),
                                    )
                                }),
                        )
                        .child(
                            div()
                                .flex()
                                .items_start()
                                .gap_2()
                                .child(div().w(px(90.0)).text_sm().pt_1().child("生成文件"))
                                .child(file_checkboxes),
                        )
                        .when(is_java, |this| {
                            this.child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .child(field("entity.java 包名", Input::new(&entity_state)))
                                    .child(field("mapper.java 包名", Input::new(&mapper_state)))
                                    .child(field("service.java 包名", Input::new(&service_state)))
                                    .child(field("serviceImpl.java 包名", Input::new(&service_impl_state)))
                                    .child(field("controller.java 包名", Input::new(&controller_state))),
                            )
                        })
                        .child(
                            div()
                                .mt_2()
                                .border_t_1()
                                .border_color(cx.theme().border)
                                .pt_3()
                                .child(result_section),
                        )
                        .child(
                            div()
                                .flex()
                                .justify_end()
                                .gap_2()
                                .mt_2()
                                .child({
                                    let _ = weak.clone();
                                    Button::new("code-gen-cancel")
                                        .label("取消")
                                        .on_click(move |_, window, cx| {
                                            window.close_sheet(cx);
                                        })
                                })
                                .child({
                                    let weak = weak.clone();
                                    let ft_shared = file_types_shared.clone();
                                    let tbl_shared = tables_shared.clone();
                                    Button::new("code-gen-run")
                                        .primary()
                                        .label("生成代码")
                                        .on_click(move |_, window, cx| {
                                            if let Some(this) = weak.upgrade() {
                                                this.update(cx, |this, cx| {
                                                    // 从共享状态读取用户选择
                                                    this.code_file_types =
                                                        ft_shared.borrow().iter().cloned().collect();
                                                    this.code_selected_tables =
                                                        tbl_shared.borrow().iter().cloned().collect();
                                                    this.generate_code(window, cx);
                                                });
                                            }
                                        })
                                }),
                        ),
                )
        });
    }

    fn clear_outputs(&mut self, cx: &mut Context<Self>) {
        self.report_output = None;
        self.sql_output.clear();
        self.check_output = None;
        self.code_output.clear();
        self.generated_codes.clear();
        self.code_active_tab = 0;
        self.table_output.clear();
        self.status = "结果已清空。".to_string();
        cx.notify();
    }

    fn open_report_drawer(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let report = self.report_output.clone();
        let scroll_handle = ScrollHandle::default();
        window.open_sheet_at(Placement::Right, cx, move |sheet, window, cx| {
            let content_h = window.viewport_size().height - px(130.0);
            let mut content = div().flex().flex_col().gap_4().p_3();

            if let Some(ref report) = report {
                // 增加的表（info 蓝色，对齐 Tauri n-tag type="info"）
                if !report.incres.is_empty() {
                    content = content.child(
                        div()
                            .child(div().text_base().font_semibold().mb_2().child("增加的表"))
                            .child(
                                div().flex().flex_wrap().gap_2().children(
                                    report.incres.iter().enumerate().map(|(i, name)| {
                                        tag("incre", i, name, cx.theme().info)
                                    }),
                                ),
                            ),
                    );
                }
                // 删除的表（danger 红色，对齐 Tauri n-tag type="error"）
                if !report.misses.is_empty() {
                    content = content.child(
                        div()
                            .child(div().text_base().font_semibold().mb_2().child("删除的表"))
                            .child(
                                div().flex().flex_wrap().gap_2().children(
                                    report.misses.iter().enumerate().map(|(i, name)| {
                                        tag("miss", i, name, cx.theme().danger)
                                    }),
                                ),
                            ),
                    );
                }
                // 变动的表
                if !report.changes.is_empty() {
                    content = content.child(
                        div().text_base().font_semibold().mb_2().child("变动的表"),
                    );
                    for (ti, table) in report.changes.iter().enumerate() {
                        let mut table_div = div()
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded_md()
                            .p_3()
                            .mb_3()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .mb_2()
                                    .child(
                                        div()
                                            .px_2()
                                            .py_1()
                                            .rounded_sm()
                                            .bg(cx.theme().warning)
                                            .text_sm()
                                            .child(table.table_name.clone()),
                                    ),
                            );

                        // 表注释变化（info 色 tag，对齐 Tauri n-tag type="info"）
                        if table.comment_change {
                            let src_label = format!("基准库：{}", table.source_comment);
                            let tgt_label = format!("变动库：{}", table.target_comment);
                            table_div = table_div.child(
                                div().mb_2().flex().flex_col().gap_1()
                                    .child(tag(("tc-src-comment", ti), 0, &src_label, cx.theme().info))
                                    .child(tag(("tc-tgt-comment", ti), 0, &tgt_label, cx.theme().info)),
                            );
                        }

                        // 增加的字段（warning 黄色，对齐 Tauri n-tag type="warning"）
                        if !table.incre_columns.is_empty() {
                            table_div = table_div.child(
                                div().mb_2()
                                    .child(div().text_sm().font_semibold().mb_1().child("增加的字段"))
                                    .child(div().flex().flex_wrap().gap_1()
                                        .children(table.incre_columns.iter().enumerate().map(|(i, c)| {
                                            tag(("tc-incre", ti), i, c, cx.theme().warning)
                                        })))
                            );
                        }

                        // 缺失的字段（warning 黄色，对齐 Tauri n-tag type="warning"）
                        if !table.miss_columns.is_empty() {
                            table_div = table_div.child(
                                div().mb_2()
                                    .child(div().text_sm().font_semibold().mb_1().child("缺失的字段"))
                                    .child(div().flex().flex_wrap().gap_1()
                                        .children(table.miss_columns.iter().enumerate().map(|(i, c)| {
                                            tag(("tc-miss", ti), i, c, cx.theme().warning)
                                        })))
                            );
                        }

                        // 变动的字段
                        if !table.columns.is_empty() {
                            let mut cols_div = div().mb_2()
                                .child(div().text_sm().font_semibold().mb_1().child("变动的字段"));
                            for (ci, col) in table.columns.iter().enumerate() {
                                cols_div = cols_div.child(
                                    div().border_1().border_color(cx.theme().border).rounded_sm().p_2().mb_1()
                                        .child(div().text_sm().font_semibold().child(col.name.clone()))
                                        .when(col.field_type_change, |this| this.child(change_row("类型",
                                            &col.source_field_type.map(|t| format!("{t:?}")).unwrap_or_default(),
                                            &col.target_field_type.map(|t| format!("{t:?}")).unwrap_or_default())))
                                        .when(col.length_change, |this| this.child(change_row("长度",
                                            &col.source_length.map(|l| l.to_string()).unwrap_or_default(),
                                            &col.target_length.map(|l| l.to_string()).unwrap_or_default())))
                                        .when(col.scale_change, |this| this.child(change_row("小数位",
                                            &col.source_scale.map(|l| l.to_string()).unwrap_or_default(),
                                            &col.target_scale.map(|l| l.to_string()).unwrap_or_default())))
                                        .when(col.null_change, |this| this.child(change_row("为空",
                                            &col.source_null.to_string(), &col.target_null.to_string())))
                                        .when(col.unsigned_change, |this| this.child(change_row("无符号",
                                            &col.source_unsigned.to_string(), &col.target_unsigned.to_string())))
                                        .when(col.default_change, |this| this.child(change_row("默认值",
                                            col.source_default.as_deref().unwrap_or(""),
                                            col.target_default.as_deref().unwrap_or(""))))
                                        .when(col.comment_change, |this| this.child(change_row("注释",
                                            &col.source_comment, &col.target_comment)))
                                );
                            }
                            table_div = table_div.child(cols_div);
                        }

                        // 增加的索引（success 绿色，对齐 Tauri n-tag type="success"）
                        if !table.incre_indexs.is_empty() {
                            table_div = table_div.child(
                                div().mb_2()
                                    .child(div().text_sm().font_semibold().mb_1().child("增加的索引"))
                                    .child(div().flex().flex_wrap().gap_1()
                                        .children(table.incre_indexs.iter().enumerate().map(|(i, c)| {
                                            tag(("tc-ix-incre", ti), i, c, cx.theme().success)
                                        })))
                            );
                        }

                        // 缺失的索引（success 绿色，对齐 Tauri n-tag type="success"）
                        if !table.miss_indexs.is_empty() {
                            table_div = table_div.child(
                                div().mb_2()
                                    .child(div().text_sm().font_semibold().mb_1().child("缺失的索引"))
                                    .child(div().flex().flex_wrap().gap_1()
                                        .children(table.miss_indexs.iter().enumerate().map(|(i, c)| {
                                            tag(("tc-ix-miss", ti), i, c, cx.theme().success)
                                        })))
                            );
                        }

                        // 变动的索引
                        if !table.indexs.is_empty() {
                            let mut idx_div = div().mb_2()
                                .child(div().text_sm().font_semibold().mb_1().child("变动的索引"));
                            for (ii, idx) in table.indexs.iter().enumerate() {
                                idx_div = idx_div.child(
                                    div().border_1().border_color(cx.theme().border).rounded_sm().p_2().mb_1()
                                        .child(div().text_sm().font_semibold().child(idx.name.clone()))
                                        .when(idx.non_unique_change, |this| this.child(change_row("唯一性",
                                            &idx.source_non_unique.to_string(), &idx.target_non_unique.to_string())))
                                        .when(idx.column_name_change, |this| this.child(change_row("列名",
                                            &idx.source_column_name, &idx.target_column_name)))
                                        .when(idx.index_type_change, |this| this.child(change_row("索引类型",
                                            &idx.source_index_type, &idx.target_index_type)))
                                        .when(idx.index_comment_change, |this| this.child(change_row("索引注释",
                                            &idx.source_index_comment, &idx.target_index_comment)))
                                );
                            }
                            table_div = table_div.child(idx_div);
                        }

                        content = content.child(table_div);
                    }
                }
            } else {
                content = content.child(div().text_sm().text_color(cx.theme().muted_foreground).child("暂无数据"));
            }

            sheet
                .overlay(true)
                .overlay_closable(true)
                .size(px(600.0))
                .p_0()
                .title("数据库差异报告")
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .id("report-scroll")
                                .h(content_h)
                                .overflow_y_scroll()
                                .track_scroll(&scroll_handle)
                                .child(content)
                                .vertical_scrollbar(&scroll_handle),
                        ),
                )
        });
    }

    fn open_sql_drawer(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let content = self.sql_output.clone();
        let scroll_handle = ScrollHandle::default();
        window.open_sheet_at(Placement::Right, cx, move |sheet, window, cx| {
            let content_h = window.viewport_size().height - px(130.0);
            sheet
                .overlay(true)
                .overlay_closable(true)
                .size(px(560.0))
                .p_0()
                .title("数据库差异SQL")
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .id("sql-scroll")
                                .h(content_h)
                                .overflow_y_scroll()
                                .track_scroll(&scroll_handle)
                                .p_3()
                                .text_sm()
                                .font_family("monospace")
                                .child(content.clone())
                                .vertical_scrollbar(&scroll_handle),
                        )
                        .child(
                            h_flex()
                                .justify_end()
                                .gap_2()
                                .p_3()
                                .border_t_1()
                                .border_color(cx.theme().border)
                                .child({
                                    let content = content.clone();
                                    Button::new("copy-sql")
                                        .icon(Icon::new(IconName::Copy))
                                        .tooltip("复制")
                                        .on_click(move |_, _, cx| {
                                            cx.write_to_clipboard(ClipboardItem::new_string(
                                                content.clone(),
                                            ));
                                        })
                                }),
                        ),
                )
        });
    }

    fn open_check_drawer(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let report = self.check_output.clone();
        let scroll_handle = ScrollHandle::default();
        window.open_sheet_at(Placement::Right, cx, move |sheet, window, cx| {
            let content_h = window.viewport_size().height - px(130.0);
            let mut content = div().flex().flex_col().gap_4().p_3();

            if let Some(ref reports) = report {
                for (ri, item) in reports.iter().enumerate() {
                    // 区块：带边框，内部留白（标题用默认色，对齐 Tauri n-tag 无 type）
                    let mut block = div()
                        .border_1()
                        .border_color(cx.theme().border)
                        .rounded_md()
                        .px_5()
                        .py_3()
                        .mb_4()
                        .child(
                            div().mb_2().child(tag(
                                ("check-block", ri),
                                0,
                                &item.name,
                                cx.theme().muted_foreground,
                            )),
                        );

                    // 建议列表（编号 + 描述，对齐 Tauri 无 show 过滤）
                    if !item.suggests.is_empty() {
                        let mut sugg_div = div().flex().flex_col().gap_1();
                        for (si, sugg) in item.suggests.iter().enumerate() {
                            sugg_div = sugg_div.child(
                                div()
                                    .flex()
                                    .items_start()
                                    .gap_1()
                                    .text_sm()
                                    .child(
                                        div()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(format!("{}.", si + 1)),
                                    )
                                    .child(div().flex_1().child(sugg.desc.clone())),
                            );
                        }
                        block = block.child(sugg_div);
                    }

                    // 子区块（缩进，标题用默认色）
                    for (ci, child) in item.children.iter().enumerate() {
                        let mut child_block = div()
                            .ml_8()
                            .px_5()
                            .py_2()
                            .mb_2()
                            .child(
                                div().mb_1().child(tag(
                                    ("check-child", ri, ci),
                                    0,
                                    &child.name,
                                    cx.theme().muted_foreground,
                                )),
                            );

                        if !child.suggests.is_empty() {
                            let mut sugg_div = div().flex().flex_col().gap_1();
                            for (si, sugg) in child.suggests.iter().enumerate() {
                                sugg_div = sugg_div.child(
                                    div()
                                        .flex()
                                        .items_start()
                                        .gap_1()
                                        .text_sm()
                                        .child(
                                            div()
                                                .text_color(cx.theme().muted_foreground)
                                                .child(format!("{}.", si + 1)),
                                        )
                                        .child(div().flex_1().child(sugg.desc.clone())),
                                );
                            }
                            child_block = child_block.child(sugg_div);
                        }

                        block = block.child(child_block);
                    }

                    content = content.child(block);
                }
            } else {
                content = content.child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child("暂无数据"),
                );
            }

            sheet
                .overlay(true)
                .overlay_closable(true)
                .size(px(600.0))
                .p_0()
                .title("数据库规范检查")
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .id("check-scroll")
                                .h(content_h)
                                .overflow_y_scroll()
                                .track_scroll(&scroll_handle)
                                .child(content)
                                .vertical_scrollbar(&scroll_handle),
                        ),
                )
        });
    }

    /// 执行全部规范检查（不打开选择弹窗）
    fn run_standard_check_all(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let source = self.selected_datasource(&self.check_source_select, cx);
        let Some(source) = source else {
            self.status = "请先选择基准库连接。".to_string();
            cx.notify();
            return;
        };

        self.is_running = true;
        self.status = "正在执行规范检查（全部）...".to_string();
        cx.notify();

        cx.spawn_in(window, async move |this: WeakEntity<Self>, cx| {
            let codes: Vec<i32> = database::database_standard_check_codes()
                .await
                .into_iter()
                .filter_map(standard_check_option)
                .map(|opt| opt.code)
                .collect();
            let result = database::standard_check(source, codes).await;
            let _ = this.update_in(cx, |this, window, cx| {
                this.is_running = false;
                match result {
                    Ok(report) => {
                        this.check_output = Some(report);
                        this.status = "规范检查完成：全部检查".to_string();
                        this.open_check_drawer(window, cx);
                    }
                    Err(err) => this.status = format!("规范检查失败：{err}"),
                }
                cx.notify();
            });
        })
        .detach();
    }
}

impl Render for DatabaseDiff {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()

            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    // 差异报告行
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(title_with_tooltip(
                                "差异报告",
                                "对比两个数据库之间的差异变化，用于评审检查数据库的变动",
                            ))
                            .child(div().text_sm().child("基准库"))
                            .child(
                                div()
                                    .w(px(250.0))
                                    .child(Select::new(&self.report_source_select)),
                            )
                            .child(div().text_sm().child("变动库"))
                            .child(
                                div()
                                    .w(px(250.0))
                                    .child(Select::new(&self.report_target_select)),
                            )
                            .child(
                                Button::new("db-diff-report").label("生成").on_click(
                                    cx.listener(|this, _, window, cx| {
                                        this.generate_report(window, cx);
                                    }),
                                ),
                            ),
                    )
                    // 差异SQL行
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(title_with_tooltip(
                                "差异SQL",
                                "对比基准库之后,生成的差异sql，在变化库上执行即可补齐差异。\n（注意：sql语句仅供参考，执行前应当检查一下sql，出现数据丢失一概不负责）",
                            ))
                            .child(div().text_sm().child("基准库"))
                            .child(
                                div()
                                    .w(px(250.0))
                                    .child(Select::new(&self.sql_source_select)),
                            )
                            .child(div().text_sm().child("变动库"))
                            .child(
                                div().w(px(250.0)).child(Select::new(&self.sql_target_select)),
                            )
                            .child(
                                Button::new("db-diff-sql").label("结构差异").on_click(
                                    cx.listener(|this, _, window, cx| {
                                        this.generate_sql(window, cx);
                                    }),
                                ),
                            ),
                    )
                    // 规范检查行
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(title_with_tooltip(
                                "规范检查",
                                "对基准库的数据库设计进行规范检查",
                            ))
                            .child(div().text_sm().child("基准库"))
                            .child(
                                div()
                                    .w(px(250.0))
                                    .child(Select::new(&self.check_source_select)),
                            )
                            .child(
                                Button::new("db-standard-check").label("检查").on_click(
                                    cx.listener(|this, _, window, cx| {
                                        this.run_standard_check_all(window, cx);
                                    }),
                                ),
                            )
                            .child(
                                Button::new("db-custom-check").label("自定义检查").on_click(
                                    cx.listener(|this, _, window, cx| {
                                        this.open_standard_check_dialog(window, cx);
                                    }),
                                ),
                            ),
                    )
                    // 逆向生成行
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(title_with_tooltip(
                                "逆向生成",
                                "一键生成entity.java，mapper.java，mapper.xml，service.java，serviceImpl.java，controller.java文件",
                            ))
                            .child(
                                Button::new("db-generate-code").label("生成").on_click(
                                    cx.listener(|this, _, window, cx| {
                                        this.open_code_gen_drawer(window, cx);
                                    }),
                                ),
                            ),
                    )
                    // 连接管理：匹配 Tauri 的 "新建连接" 按钮 + 数据表格布局
                    .child(
                        Button::new("new-conn")
                            .label("新建连接")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.open_conn_sheet(None, window, cx);
                            })),
                    )
                    .child(saved_datasource_panel(self, cx))
                    // 状态（仅显示运行中状态）
                    .when(!self.status.is_empty(), |this| {
                        this.child(
                            div()
                                .text_sm()
                                .text_color(if self.is_running {
                                    cx.theme().warning
                                } else {
                                    cx.theme().muted_foreground
                                })
                                .child(self.status.clone()),
                        )
                    }),
            )
    }
}

impl DatabaseDiff {
    fn all_outputs(&self) -> String {
        let report = self
            .report_output
            .as_ref()
            .map(|r| serde_json::to_string_pretty(r).unwrap_or_default())
            .unwrap_or_default();
        let check = self
            .check_output
            .as_ref()
            .map(|c| serde_json::to_string_pretty(c).unwrap_or_default())
            .unwrap_or_default();
        let code = self
            .generated_codes
            .iter()
            .map(|(file, content)| format!("// ===== {file} =====\n{content}"))
            .collect::<Vec<_>>()
            .join("\n\n");
        [
            ("数据源表", self.table_output.as_str()),
            ("差异报告", report.as_str()),
            ("差异 SQL", self.sql_output.as_str()),
            ("规范检查", check.as_str()),
            ("逆向生成", code.as_str()),
        ]
        .into_iter()
        .filter(|(_, value)| !value.trim().is_empty())
        .map(|(title, value)| format!("## {title}\n{value}"))
        .collect::<Vec<_>>()
        .join("\n\n")
    }
}

async fn generate_code_async(
    source: DatasourceInfo,
    language: &str,
    file_types: &[String],
    table_names: &[String],
    package_names: &HashMap<String, String>,
) -> Result<Vec<(String, String)>, String> {
    let tree = database::database_table_tree(source)
        .await
        .map_err(|err| err.to_string())?;
    let value = serde_json::to_value(tree).map_err(|err| err.to_string())?;
    let tables = value.as_array().cloned().unwrap_or_default();
    if tables.is_empty() {
        return Err("未找到可生成的表，请先确认基准库表结构。".to_string());
    }

    // 如果指定了表名，只生成选中的表
    let selected_tables: Vec<serde_json::Value> = if table_names.is_empty() {
        tables
    } else {
        tables
            .into_iter()
            .filter(|t| {
                let name = t
                    .get("tableName")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default();
                let schema = t
                    .get("schema")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default();
                let full = if schema.is_empty() {
                    name.to_string()
                } else {
                    format!("{schema}.{name}")
                };
                table_names.contains(&full) || table_names.contains(&name.to_string())
            })
            .collect()
    };

    if selected_tables.is_empty() {
        return Err("未找到可生成的表，请先选择表".to_string());
    }

    let mut files = Vec::new();
    for table in selected_tables {
        let table_name = table
            .get("tableName")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        if table_name.is_empty() {
            continue;
        }
        for file_type in file_types {
            if let Some((file_name, code)) =
                render_generate_file(language, file_type, &table, package_names)
            {
                files.push((format!("{table_name}.{file_name}"), code));
            }
        }
    }

    if files.is_empty() {
        Err("没有生成任何代码，请检查语言和文件类型。".to_string())
    } else {
        Ok(files)
    }
}

fn render_generate_file(
    language: &str,
    file_type: &str,
    table: &serde_json::Value,
    package_names: &HashMap<String, String>,
) -> Option<(String, String)> {
    match language {
        "java" => render_java_file(file_type, table, package_names),
        _ => render_rust_file(file_type, table),
    }
}

fn render_rust_file(file_type: &str, table: &serde_json::Value) -> Option<(String, String)> {
    if file_type != "model.rs" {
        return None;
    }

    let table_name = table
        .get("tableName")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    let table_comment = table
        .get("tableComment")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    let struct_name = upper_camel(table_name);
    let columns = table
        .get("children")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();

    let fields = columns
        .iter()
        .filter_map(rust_field)
        .collect::<Vec<_>>()
        .join("\n");

    let file_name = format!("{}.rs", snake_case(table_name));
    let code = format!(
        "use serde::{{Deserialize, Serialize}};\n\n/// {table_comment}\n#[derive(Debug, Clone, Serialize, Deserialize)]\n#[serde(rename_all = \"camelCase\")]\npub struct {struct_name} {{\n{fields}\n}}\n"
    );
    Some((file_name, code))
}

fn rust_field(column: &serde_json::Value) -> Option<String> {
    let name = column.get("name").and_then(serde_json::Value::as_str)?;
    let comment = column
        .get("comment")
        .and_then(serde_json::Value::as_str)
        .unwrap_or(name);
    let column_type = column
        .get("type")
        .map(value_to_type_text)
        .unwrap_or_else(|| "text".to_string());
    let is_null = column
        .get("isNull")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true);
    let rust_type_str = rust_type(&column_type);
    let field_type = if is_null {
        format!("Option<{}>", rust_type_str)
    } else {
        rust_type_str.to_string()
    };
    Some(format!(
        "    /// {comment}\n    pub {}: {field_type},",
        rust_ident(name)
    ))
}

fn render_java_file(
    file_type: &str,
    table: &serde_json::Value,
    package_names: &HashMap<String, String>,
) -> Option<(String, String)> {
    let table_name = table
        .get("tableName")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    let table_comment = table
        .get("tableComment")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    let class_name = upper_camel(table_name);
    let entity_package = package_name(package_names, "entity.java", "com.example.entity");
    let mapper_package = package_name(package_names, "mapper.java", "com.example.mapper");
    let service_package = package_name(package_names, "service.java", "com.example.service");
    let service_impl_package = package_name(
        package_names,
        "serviceImpl.java",
        "com.example.service.impl",
    );
    let controller_package =
        package_name(package_names, "controller.java", "com.example.controller");

    match file_type {
        "entity.java" => Some((
            format!("{class_name}.java"),
            java_entity(
                table,
                &class_name,
                &entity_package,
                table_name,
                table_comment,
            ),
        )),
        "mapper.java" => Some((
            format!("{class_name}Mapper.java"),
            format!(
                "package {mapper_package};\n\nimport com.baomidou.mybatisplus.core.mapper.BaseMapper;\nimport {entity_package}.{class_name};\n\n/**\n * {table_comment}\n */\npublic interface {class_name}Mapper extends BaseMapper<{class_name}> {{\n}}\n"
            ),
        )),
        "mapper.xml" => Some((
            format!("{class_name}Mapper.xml"),
            format!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\" ?>\n<!DOCTYPE mapper PUBLIC \"-//mybatis.org//DTD Mapper 3.0//EN\" \"http://mybatis.org/dtd/mybatis-3-mapper.dtd\">\n<mapper namespace=\"{mapper_package}.{class_name}Mapper\">\n</mapper>\n"
            ),
        )),
        "service.java" => Some((
            format!("I{class_name}Service.java"),
            format!(
                "package {service_package};\n\nimport com.baomidou.mybatisplus.extension.service.IService;\nimport {entity_package}.{class_name};\n\n/**\n * {table_comment}\n */\npublic interface I{class_name}Service extends IService<{class_name}> {{\n}}\n"
            ),
        )),
        "serviceImpl.java" => Some((
            format!("{class_name}ServiceImpl.java"),
            format!(
                "package {service_impl_package};\n\nimport com.baomidou.mybatisplus.extension.service.impl.ServiceImpl;\nimport {entity_package}.{class_name};\nimport {mapper_package}.{class_name}Mapper;\nimport {service_package}.I{class_name}Service;\nimport org.springframework.stereotype.Service;\n\n/**\n * {table_comment}\n */\n@Service\npublic class {class_name}ServiceImpl extends ServiceImpl<{class_name}Mapper, {class_name}> implements I{class_name}Service {{\n}}\n"
            ),
        )),
        "controller.java" => Some((
            format!("{class_name}Controller.java"),
            format!(
                "package {controller_package};\n\nimport org.springframework.web.bind.annotation.RequestMapping;\nimport org.springframework.web.bind.annotation.RestController;\n\n/**\n * {table_comment}\n */\n@RestController\n@RequestMapping(\"/{route}\")\npublic class {class_name}Controller {{\n}}\n",
                route = kebab_case(table_name)
            ),
        )),
        _ => None,
    }
}

fn java_entity(
    table: &serde_json::Value,
    class_name: &str,
    package: &str,
    table_name: &str,
    table_comment: &str,
) -> String {
    let columns = table
        .get("children")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let imports = java_imports(&columns);
    let fields = columns
        .iter()
        .filter_map(|column| {
            let name = column.get("name").and_then(serde_json::Value::as_str)?;
            let comment = column
                .get("comment")
                .and_then(serde_json::Value::as_str)
                .filter(|s| !s.is_empty())
                .unwrap_or(name);
            let column_type = column
                .get("type")
                .map(value_to_type_text)
                .unwrap_or_else(|| "String".to_string());
            let length = column
                .get("length")
                .and_then(serde_json::Value::as_i64)
                .map(|v| v as i32);
            let field_type = java_type(&column_type, length);
            let field_name = lower_camel(name);
            let is_primary_key = column
                .get("isPrimaryKey")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false);
            let table_id = if is_primary_key { "    @TableId\n" } else { "" };
            Some(format!(
                "    /** {comment} */\n{table_id}    private {field_type} {field_name};\n"
            ))
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "package {package};\n\nimport com.baomidou.mybatisplus.annotation.TableId;\nimport com.baomidou.mybatisplus.annotation.TableName;\nimport lombok.Data;\n{imports}/**\n * {table_comment}\n */\n@Data\n@TableName(\"{table_name}\")\npublic class {class_name} {{\n\n{fields}\n}}\n"
    )
}

fn java_imports(columns: &[serde_json::Value]) -> String {
    let mut imports = std::collections::HashSet::new();
    for column in columns {
        let column_type = column
            .get("type")
            .map(value_to_type_text)
            .unwrap_or_else(|| "String".to_string());
        match column_type.as_str() {
            "Date" => {
                imports.insert("import java.time.LocalDate;");
            }
            "DateTime" | "Timestamp" | "Time" => {
                imports.insert("import java.time.LocalDateTime;");
            }
            "Decimal" | "Numeric" => {
                imports.insert("import java.math.BigDecimal;");
            }
            _ => {}
        }
    }
    if imports.is_empty() {
        String::new()
    } else {
        let mut sorted = imports.into_iter().collect::<Vec<_>>();
        sorted.sort();
        format!("{}\n", sorted.join("\n"))
    }
}

fn java_type(column_type: &str, length: Option<i32>) -> &'static str {
    match column_type {
        "Bigint" => "Long",
        "SmallInt" | "TinyInt" => "Integer",
        "Int" | "Integer" | "MediumInt" => "Integer",
        "Float" | "Double" | "Real" => "Double",
        "Decimal" | "Numeric" => "BigDecimal",
        "Date" => "LocalDate",
        "DateTime" | "Timestamp" | "Time" => "LocalDateTime",
        "Bit" if length == Some(1) => "Boolean",
        "Blob" | "LongBlob" | "MediumBlob" | "TinyBlob" | "Binary" | "Varbinary" => "byte[]",
        _ => "String",
    }
}

fn package_name(package_names: &HashMap<String, String>, key: &str, default: &str) -> String {
    package_names
        .get(key)
        .filter(|value| !value.trim().is_empty())
        .cloned()
        .unwrap_or_else(|| default.to_string())
}

fn default_file_types(language: &str) -> Vec<String> {
    match language {
        "java" => vec![
            "entity.java".to_string(),
            "mapper.java".to_string(),
            "mapper.xml".to_string(),
            "service.java".to_string(),
            "serviceImpl.java".to_string(),
            "controller.java".to_string(),
        ],
        _ => vec!["model.rs".to_string()],
    }
}

fn language_key(label: &str) -> String {
    match label {
        "Java" => "java".to_string(),
        _ => "rust".to_string(),
    }
}

fn language_label(key: &str) -> &'static str {
    match key {
        "java" => "Java",
        _ => "Rust",
    }
}

fn rust_type(column_type: &str) -> &'static str {
    let lower = column_type.to_ascii_lowercase();
    if lower.contains("bigint") {
        "i64"
    } else if lower.contains("int") {
        "i32"
    } else if lower.contains("decimal")
        || lower.contains("numeric")
        || lower.contains("double")
        || lower.contains("float")
        || lower.contains("real")
    {
        "f64"
    } else if lower.contains("bool") || lower.contains("bit") {
        "bool"
    } else {
        "String"
    }
}

fn rust_ident(name: &str) -> String {
    let mut value = name.replace('-', "_");
    if matches!(
        value.as_str(),
        "as" | "async"
            | "await"
            | "break"
            | "const"
            | "continue"
            | "crate"
            | "dyn"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "union"
            | "unsafe"
            | "use"
            | "where"
            | "while"
    ) {
        value = format!("r#{value}");
    }
    value
}

fn upper_camel(value: &str) -> String {
    let mut out = String::new();
    for part in value
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|part| !part.is_empty())
    {
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            out.push(first.to_ascii_uppercase());
            out.extend(chars.map(|ch| ch.to_ascii_lowercase()));
        }
    }
    if out.is_empty() {
        "GeneratedModel".to_string()
    } else {
        out
    }
}

fn lower_camel(value: &str) -> String {
    let upper = upper_camel(value);
    let mut chars = upper.chars();
    match chars.next() {
        Some(first) => format!("{}{}", first.to_ascii_lowercase(), chars.as_str()),
        None => String::new(),
    }
}

fn snake_case(value: &str) -> String {
    split_words(value).join("_").to_lowercase()
}

fn kebab_case(value: &str) -> String {
    split_words(value).join("-").to_lowercase()
}

fn split_words(value: &str) -> Vec<String> {
    value
        .split(|c: char| c == '_' || c == '-' || c == ' ' || c == '.')
        .filter(|part| !part.is_empty())
        .map(|part| part.to_string())
        .collect()
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

impl DbForm {
    fn new(window: &mut Window, cx: &mut Context<DatabaseDiff>) -> Self {
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
        let username = read_input(&self.username, cx);
        let password = read_input(&self.password, cx);
        let port = read_input(&self.port, cx).parse::<u16>().ok();

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
            port,
            username: empty_to_none(username),
            password: empty_to_none(password),
            database: Some(database),
        })
    }

    fn apply(&self, info: &DatasourceInfo, window: &mut Window, cx: &mut Context<DatabaseDiff>) {
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

fn upsert_datasource(items: &mut Vec<DatasourceInfo>, info: DatasourceInfo) {
    items.retain(|item| item.name != info.name);
    items.insert(0, info);
}

fn saved_datasource_panel(this: &DatabaseDiff, cx: &mut Context<DatabaseDiff>) -> Div {
    // 匹配 Tauri 的 n-data-table 布局: 连接名称 | 主机 | 端口 | 数据库 | 操作
    let mut panel = div()
        .border_1()
        .border_color(cx.theme().border)
        .rounded_lg()
        .overflow_hidden();

    // 表头
    let header = div()
        .flex()
        .items_center()
        .bg(cx.theme().muted)
        .border_b_1()
        .border_color(cx.theme().border)
        .child(
            div()
                .w(px(140.))
                .p_2()
                .text_sm()
                .font_semibold()
                .child("连接名称"),
        )
        .child(
            div()
                .w(px(160.))
                .p_2()
                .text_sm()
                .font_semibold()
                .child("主机"),
        )
        .child(
            div()
                .w(px(80.))
                .p_2()
                .text_sm()
                .font_semibold()
                .child("端口"),
        )
        .child(
            div()
                .w(px(140.))
                .p_2()
                .text_sm()
                .font_semibold()
                .child("数据库"),
        )
        .child(div().flex_1().p_2().text_sm().font_semibold().child("操作"));
    panel = panel.child(header);

    if this.saved_datasources.is_empty() {
        panel = panel.child(
            div()
                .p_4()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("暂无连接，点击「新建连接」添加。"),
        );
    } else {
        for (index, datasource) in this.saved_datasources.iter().enumerate() {
            let edit_name = datasource.name.clone();
            let edit_name2 = datasource.name.clone();
            let delete_name = datasource.name.clone();
            let row = div()
                .flex()
                .items_center()
                .border_b_1()
                .border_color(cx.theme().border)
                .child(
                    div()
                        .w(px(140.))
                        .p_2()
                        .text_sm()
                        .child(datasource.name.clone()),
                )
                .child(
                    div()
                        .w(px(160.))
                        .p_2()
                        .text_sm()
                        .child(datasource.host.clone()),
                )
                .child(
                    div()
                        .w(px(80.))
                        .p_2()
                        .text_sm()
                        .child(datasource.port.map(|p| p.to_string()).unwrap_or_default()),
                )
                .child(
                    div()
                        .w(px(140.))
                        .p_2()
                        .text_sm()
                        .child(datasource.database.clone().unwrap_or_default()),
                )
                .child(
                    div().flex_1().p_2().child(
                        ButtonGroup::new(("ds-ops", index))
                            .child(
                                Button::new(("ds-test", index))
                                    .label("测试")
                                    .xsmall()
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.ping_by_name(edit_name.clone(), cx);
                                    })),
                            )
                            .child(
                                Button::new(("ds-edit", index))
                                    .label("编辑")
                                    .xsmall()
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        this.open_conn_sheet(Some(edit_name2.clone()), window, cx);
                                    })),
                            )
                            .child(
                                Button::new(("ds-delete", index))
                                    .label("删除")
                                    .xsmall()
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        this.delete_datasource_by_name(
                                            delete_name.clone(),
                                            window,
                                            cx,
                                        );
                                    })),
                            ),
                    ),
                );
            panel = panel.child(row);
        }
    }

    panel
}

fn available_file_types(language: &str) -> Vec<String> {
    match language {
        "java" => vec![
            "entity.java".to_string(),
            "mapper.java".to_string(),
            "mapper.xml".to_string(),
            "service.java".to_string(),
            "serviceImpl.java".to_string(),
            "controller.java".to_string(),
        ],
        _ => vec!["model.rs".to_string()],
    }
}

fn file_type_label(file_type: &str) -> &'static str {
    match file_type {
        "entity.java" => "Entity",
        "mapper.java" => "Mapper",
        "mapper.xml" => "Mapper XML",
        "service.java" => "Service",
        "serviceImpl.java" => "Service Impl",
        "controller.java" => "Controller",
        "model.rs" => "Model",
        _ => "Unknown",
    }
}

fn file_language_tag(file_name: &str) -> &'static str {
    if file_name.ends_with(".java") {
        "java"
    } else if file_name.ends_with(".xml") {
        "xml"
    } else if file_name.ends_with(".rs") {
        "rust"
    } else {
        "text"
    }
}

fn standard_check_option(
    item: std::collections::HashMap<String, String>,
) -> Option<StandardCheckOption> {
    let code = item
        .get("value")
        .or_else(|| item.get("code"))?
        .parse::<i32>()
        .ok()?;
    let desc = item
        .get("desc")
        .or_else(|| item.get("label"))
        .cloned()
        .unwrap_or_else(|| format!("检查项 {code}"));
    Some(StandardCheckOption { code, desc })
}

fn title_with_tooltip(title: &'static str, tooltip_text: &'static str) -> Div {
    div()
        .flex()
        .items_center()
        .gap_1()
        .child(div().text_sm().font_semibold().child(title))
        .child(
            Button::new(title)
                .icon(Icon::new(IconName::Info))
                .ghost()
                .tooltip(tooltip_text),
        )
}

fn datasource_label(ds: &DatasourceInfo) -> String {
    let database = ds.database.as_deref().unwrap_or("");
    let host = if ds.host.is_empty() {
        String::new()
    } else {
        format!("（{}:{}）", ds.host, ds.port.unwrap_or(0))
    };
    format!("{database} / {} {host}", ds.name)
}

fn datasource_options(datasources: &[DatasourceInfo]) -> Vec<String> {
    datasources.iter().map(datasource_label).collect()
}

fn field(label: &'static str, input: impl IntoElement) -> Div {
    div()
        .flex()
        .items_center()
        .gap_2()
        .child(div().w(px(72.0)).text_sm().child(label))
        .child(div().flex_1().child(input))
}

/// 渲染标签（用于增加/缺失的表名、字段名等）
fn tag(
    prefix: impl std::hash::Hash + std::fmt::Debug,
    index: usize,
    label: &str,
    color: Hsla,
) -> Stateful<Div> {
    div()
        .id(ElementId::Name(std::format!("{prefix:?}-{index}").into()))
        .px_2()
        .py_1()
        .rounded_sm()
        .text_xs()
        .bg(color.opacity(0.15))
        .border_1()
        .border_color(color.opacity(0.5))
        .text_color(color)
        .child(label.to_string())
}

/// 渲染变化行（源值 变更为 目标值）
fn change_row(label: &str, source: &str, target: &str) -> Div {
    div()
        .flex()
        .items_center()
        .gap_1()
        .text_xs()
        .child(div().w(px(60.0)).text_color(rgba(0x7c7c7cff)).child(label.to_string()))
        .child(div().child(source.to_string()))
        .child(div().text_color(rgba(0x7c7c7cff)).child("变更为"))
        .child(div().child(target.to_string()))
}
