use std::sync::LazyLock;

use gpui::*;
use gpui_component::{button::*, scroll::ScrollableElement, sidebar::*, *};
use gpui_component_assets::Assets;
mod config_store;
mod views;
use views::*;

/// 全局 Tokio runtime，供 sqlx 等需要 Tokio 上下文的库使用
static TOKIO_RUNTIME: LazyLock<tokio::runtime::Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime")
});

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ViewType {
    Home,
    SystemMonitor,
    CodeSnippet,
    Todo,
    TransformFiletype,
    TransformTime,
    TransformBaseConversion,
    EncodeDecodeBase64,
    EncodeDecodeUrl,
    EncodeDecodeJwt,
    EncodeDecodeCharset,
    EncodeDecodeMessyCode,
    FormatterJson,
    FormatterSql,
    FormatterXml,
    GeneratorUuid,
    GeneratorHash,
    GeneratorChecksum,
    DatabaseDatafaker,
    DatabaseDiff,
    TextMarkdown,
    NetworkIp,
    ImageExcalidraw,
    OtherQrCode,
    OtherClipboard,
    RegexVisualizer,
    Settings,
}

pub struct App {
    current_view: ViewType,
    sidebar_collapsed: bool,
    uuid_generator: Option<Entity<UuidGenerator>>,
    hash_calculator: Option<Entity<HashCalculator>>,
    base64_encoder: Option<Entity<Base64Encoder>>,
    url_encoder: Option<Entity<UrlEncoder>>,
    jwt_decoder: Option<Entity<JwtDecoder>>,
    timestamp_converter: Option<Entity<TimestampConverter>>,
    base_converter: Option<Entity<BaseConverter>>,
    json_editor: Option<Entity<JsonEditor>>,
    markdown_editor: Option<Entity<MarkdownEditor>>,
    code_snippet: Option<Entity<CodeSnippet>>,
    todo_list: Option<Entity<TodoList>>,
    charset_encoder: Option<Entity<CharsetEncoder>>,
    messy_code_recover: Option<Entity<MessyCodeRecover>>,
    sql_formatter: Option<Entity<SqlFormatter>>,
    xml_formatter: Option<Entity<XmlFormatter>>,
    file_verify: Option<Entity<FileVerify>>,
    fake_data_generator: Option<Entity<FakeDataGenerator>>,
    database_diff: Option<Entity<DatabaseDiff>>,
    system_monitor: Option<Entity<SystemMonitor>>,
    transform_filetype: Option<Entity<TransformFiletype>>,
    ip_converter: Option<Entity<IpConverter>>,
    qrcode_generator: Option<Entity<QrCodeGenerator>>,
    clipboard_manager: Option<Entity<ClipboardManager>>,
    regex_visualizer: Option<Entity<RegexVisualizer>>,
    excalidraw: Option<Entity<ExcalidrawView>>,
    settings: Option<Entity<SettingsView>>,
}

impl App {
    fn new() -> Self {
        Self {
            current_view: ViewType::Home,
            sidebar_collapsed: false,
            uuid_generator: None,
            hash_calculator: None,
            base64_encoder: None,
            url_encoder: None,
            jwt_decoder: None,
            timestamp_converter: None,
            base_converter: None,
            json_editor: None,
            markdown_editor: None,
            code_snippet: None,
            todo_list: None,
            charset_encoder: None,
            messy_code_recover: None,
            sql_formatter: None,
            xml_formatter: None,
            file_verify: None,
            fake_data_generator: None,
            database_diff: None,
            system_monitor: None,
            transform_filetype: None,
            ip_converter: None,
            qrcode_generator: None,
            clipboard_manager: None,
            regex_visualizer: None,
            excalidraw: None,
            settings: None,
        }
    }

    fn set_view(&mut self, view: ViewType, cx: &mut Context<Self>) {
        self.current_view = view;
        cx.notify();
    }

    fn toggle_sidebar(&mut self, cx: &mut Context<Self>) {
        self.sidebar_collapsed = !self.sidebar_collapsed;
        cx.notify();
    }
}

impl Render for App {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let current_view = self.current_view;
        let sheet_layer = Root::render_sheet_layer(window, cx);
        let dialog_layer = Root::render_dialog_layer(window, cx);
        let notification_layer = Root::render_notification_layer(window, cx);

        div()
            .size_full()
            .relative()
            .child(
                div()
                    .size_full()
                    .flex()
                    .overflow_hidden()
                    .child(
                        Sidebar::left()
                            .collapsed(self.sidebar_collapsed)
                            .header(
                                SidebarToggleButton::left()
                                    .collapsed(self.sidebar_collapsed)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.toggle_sidebar(cx);
                                    })),
                            )
                            .children([SidebarMenu::new().children([
                                // 首页 — 顶级（匹配 Tauri menu.ts）
                                SidebarMenuItem::new("首页")
                                    .icon(Icon::new(IconName::LayoutDashboard))
                                    .active(current_view == ViewType::Home)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.set_view(ViewType::Home, cx);
                                    })),
                                // 系统监控 — 顶级
                                SidebarMenuItem::new("系统监控")
                                    .icon(Icon::new(IconName::ChartPie))
                                    .active(current_view == ViewType::SystemMonitor)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.set_view(ViewType::SystemMonitor, cx);
                                    })),
                                // 代码片段 — 顶级
                                SidebarMenuItem::new("代码片段")
                                    .icon(Icon::new(IconName::File))
                                    .active(current_view == ViewType::CodeSnippet)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.set_view(ViewType::CodeSnippet, cx);
                                    })),
                                // 待办事项 — 顶级
                                SidebarMenuItem::new("待办事项")
                                    .icon(Icon::new(IconName::Check))
                                    .active(current_view == ViewType::Todo)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.set_view(ViewType::Todo, cx);
                                    })),
                                // 转换 — 可展开父菜单
                                SidebarMenuItem::new("转换")
                                    .icon(Icon::new(IconName::Replace))
                                    .click_to_open(true)
                                    .children([
                                        SidebarMenuItem::new("文件格式转换")
                                            .icon(Icon::new(IconName::File))
                                            .active(current_view == ViewType::TransformFiletype)
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.set_view(ViewType::TransformFiletype, cx);
                                            })),
                                        SidebarMenuItem::new("时间戳")
                                            .icon(Icon::new(IconName::Calendar))
                                            .active(current_view == ViewType::TransformTime)
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.set_view(ViewType::TransformTime, cx);
                                            })),
                                        SidebarMenuItem::new("进制转换")
                                            .icon(Icon::new(IconName::ALargeSmall))
                                            .active(current_view == ViewType::TransformBaseConversion)
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.set_view(
                                                    ViewType::TransformBaseConversion,
                                                    cx,
                                                );
                                            })),
                                    ]),
                                // 编码/解码 — 可展开父菜单
                                SidebarMenuItem::new("编码/解码")
                                    .icon(Icon::new(IconName::Dash))
                                    .click_to_open(true)
                                    .children([
                                        SidebarMenuItem::new("Base64")
                                            .icon(Icon::new(IconName::CaseSensitive))
                                            .active(current_view == ViewType::EncodeDecodeBase64)
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.set_view(ViewType::EncodeDecodeBase64, cx);
                                            })),
                                        SidebarMenuItem::new("URL")
                                            .icon(Icon::new(IconName::ExternalLink))
                                            .active(current_view == ViewType::EncodeDecodeUrl)
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.set_view(ViewType::EncodeDecodeUrl, cx);
                                            })),
                                        SidebarMenuItem::new("JWT")
                                            .icon(Icon::new(IconName::File))
                                            .active(current_view == ViewType::EncodeDecodeJwt)
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.set_view(ViewType::EncodeDecodeJwt, cx);
                                            })),
                                        SidebarMenuItem::new("字符编码")
                                            .icon(Icon::new(IconName::CaseSensitive))
                                            .active(current_view == ViewType::EncodeDecodeCharset)
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.set_view(ViewType::EncodeDecodeCharset, cx);
                                            })),
                                        SidebarMenuItem::new("乱码恢复")
                                            .icon(Icon::new(IconName::CaseSensitive))
                                            .active(current_view == ViewType::EncodeDecodeMessyCode)
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.set_view(ViewType::EncodeDecodeMessyCode, cx);
                                            })),
                                    ]),
                                // 格式化 — 可展开父菜单
                                SidebarMenuItem::new("格式化")
                                    .icon(Icon::new(IconName::Replace))
                                    .click_to_open(true)
                                    .children([
                                        SidebarMenuItem::new("JSON Editor")
                                            .icon(Icon::new(IconName::File))
                                            .active(current_view == ViewType::FormatterJson)
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.set_view(ViewType::FormatterJson, cx);
                                            })),
                                        SidebarMenuItem::new("SQL")
                                            .icon(Icon::new(IconName::SquareTerminal))
                                            .active(current_view == ViewType::FormatterSql)
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.set_view(ViewType::FormatterSql, cx);
                                            })),
                                        SidebarMenuItem::new("XML")
                                            .icon(Icon::new(IconName::File))
                                            .active(current_view == ViewType::FormatterXml)
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.set_view(ViewType::FormatterXml, cx);
                                            })),
                                    ]),
                                // 生成器 — 可展开父菜单
                                SidebarMenuItem::new("生成器")
                                    .icon(Icon::new(IconName::Plus))
                                    .click_to_open(true)
                                    .children([
                                        SidebarMenuItem::new("UUID")
                                            .icon(Icon::new(IconName::ALargeSmall))
                                            .active(current_view == ViewType::GeneratorUuid)
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.set_view(ViewType::GeneratorUuid, cx);
                                            })),
                                        SidebarMenuItem::new("文本Hash")
                                            .icon(Icon::new(IconName::Asterisk))
                                            .active(current_view == ViewType::GeneratorHash)
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.set_view(ViewType::GeneratorHash, cx);
                                            })),
                                        SidebarMenuItem::new("文件校验")
                                            .icon(Icon::new(IconName::File))
                                            .active(current_view == ViewType::GeneratorChecksum)
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.set_view(ViewType::GeneratorChecksum, cx);
                                            })),
                                    ]),
                                // 数据库 — 可展开父菜单
                                SidebarMenuItem::new("数据库")
                                    .icon(Icon::new(IconName::Building2))
                                    .click_to_open(true)
                                    .children([
                                        SidebarMenuItem::new("假数据生成")
                                            .icon(Icon::new(IconName::Folder))
                                            .active(current_view == ViewType::DatabaseDatafaker)
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.set_view(ViewType::DatabaseDatafaker, cx);
                                            })),
                                        SidebarMenuItem::new("数据库差异")
                                            .icon(Icon::new(IconName::Folder))
                                            .active(current_view == ViewType::DatabaseDiff)
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.set_view(ViewType::DatabaseDiff, cx);
                                            })),
                                    ]),
                                // 文本 — 可展开父菜单
                                SidebarMenuItem::new("文本")
                                    .icon(Icon::new(IconName::BookOpen))
                                    .click_to_open(true)
                                    .children([SidebarMenuItem::new("Markdown")
                                        .icon(Icon::new(IconName::BookOpen))
                                        .active(current_view == ViewType::TextMarkdown)
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.set_view(ViewType::TextMarkdown, cx);
                                        }))]),
                                // 网络 — 可展开父菜单
                                SidebarMenuItem::new("网络")
                                    .icon(Icon::new(IconName::Globe))
                                    .click_to_open(true)
                                    .children([SidebarMenuItem::new("IP")
                                        .icon(Icon::new(IconName::Globe))
                                        .active(current_view == ViewType::NetworkIp)
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.set_view(ViewType::NetworkIp, cx);
                                        }))]),
                                // 图像 — 可展开父菜单
                                SidebarMenuItem::new("图像")
                                    .icon(Icon::new(IconName::Frame))
                                    .click_to_open(true)
                                    .children([SidebarMenuItem::new("Excalidraw")
                                        .icon(Icon::new(IconName::Frame))
                                        .active(current_view == ViewType::ImageExcalidraw)
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.set_view(ViewType::ImageExcalidraw, cx);
                                        }))]),
                                // 其它 — 可展开父菜单
                                SidebarMenuItem::new("其它")
                                    .icon(Icon::new(IconName::Settings2))
                                    .click_to_open(true)
                                    .children([
                                        SidebarMenuItem::new("二维码")
                                            .icon(Icon::new(IconName::Frame))
                                            .active(current_view == ViewType::OtherQrCode)
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.set_view(ViewType::OtherQrCode, cx);
                                            })),
                                        SidebarMenuItem::new("剪贴板管理")
                                            .icon(Icon::new(IconName::Settings2))
                                            .active(current_view == ViewType::OtherClipboard)
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.set_view(ViewType::OtherClipboard, cx);
                                            })),
                                    ]),
                                // 正则 — 可展开父菜单
                                SidebarMenuItem::new("正则")
                                    .icon(Icon::new(IconName::Dash))
                                    .click_to_open(true)
                                    .children([SidebarMenuItem::new("正则可视化")
                                        .icon(Icon::new(IconName::Dash))
                                        .active(current_view == ViewType::RegexVisualizer)
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.set_view(ViewType::RegexVisualizer, cx);
                                        }))]),
                                // 设置 — 顶级（匹配 Tauri menu.ts）
                                SidebarMenuItem::new("设置")
                                    .icon(Icon::new(IconName::Settings))
                                    .active(current_view == ViewType::Settings)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.set_view(ViewType::Settings, cx);
                                    })),
                            ])]),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(640.0))
                            .overflow_y_scrollbar()
                            .child(match current_view {
                                ViewType::Home => render_home_view(cx),
                                ViewType::SystemMonitor => {
                                    render_system_monitor_view(self, window, cx)
                                }
                                ViewType::CodeSnippet => render_code_snippet_view(self, cx),
                                ViewType::Todo => render_todo_view(self, cx),
                                ViewType::TransformFiletype => {
                                    render_transform_filetype_view(self, window, cx)
                                }
                                ViewType::TransformTime => {
                                    render_timestamp_converter_view(self, window, cx)
                                }
                                ViewType::TransformBaseConversion => {
                                    render_base_converter_view(self, window, cx)
                                }
                                ViewType::EncodeDecodeBase64 => {
                                    render_base64_encoder_view(self, window, cx)
                                }
                                ViewType::EncodeDecodeUrl => {
                                    render_url_encoder_view(self, window, cx)
                                }
                                ViewType::EncodeDecodeJwt => {
                                    render_jwt_decoder_view(self, window, cx)
                                }
                                ViewType::EncodeDecodeCharset => {
                                    render_encode_decode_charset_view(self, window, cx)
                                }
                                ViewType::EncodeDecodeMessyCode => {
                                    render_encode_decode_messy_code_view(self, window, cx)
                                }
                                ViewType::FormatterJson => {
                                    render_json_editor_view(self, window, cx)
                                }
                                ViewType::FormatterSql => {
                                    render_formatter_sql_view(self, window, cx)
                                }
                                ViewType::FormatterXml => {
                                    render_formatter_xml_view(self, window, cx)
                                }
                                ViewType::GeneratorUuid => {
                                    render_uuid_generator_view(self, window, cx)
                                }
                                ViewType::GeneratorHash => {
                                    render_hash_calculator_view(self, window, cx)
                                }
                                ViewType::GeneratorChecksum => {
                                    render_generator_checksum_view(self, window, cx)
                                }
                                ViewType::DatabaseDatafaker => {
                                    render_database_datafaker_view(self, window, cx)
                                }
                                ViewType::DatabaseDiff => {
                                    render_database_diff_view(self, window, cx)
                                }
                                ViewType::TextMarkdown => {
                                    render_markdown_editor_view(self, window, cx)
                                }
                                ViewType::NetworkIp => render_ip_converter_view(self, window, cx),
                                ViewType::ImageExcalidraw => {
                                    render_excalidraw_view(self, window, cx)
                                }
                                ViewType::OtherQrCode => {
                                    render_qrcode_generator_view(self, window, cx)
                                }
                                ViewType::OtherClipboard => {
                                    render_clipboard_manager_view(self, window, cx)
                                }
                                ViewType::RegexVisualizer => {
                                    render_regex_visualizer_view(self, window, cx)
                                }
                                ViewType::Settings => render_settings_view(self, window, cx),
                            }),
                    ),
            )
            .children(sheet_layer)
            .children(dialog_layer)
            .children(notification_layer)
    }
}

fn render_home_view(cx: &mut Context<App>) -> Div {
    // 匹配 Tauri Home.vue：menuAll（排除首页，展平子菜单），flex-wrap 240px 卡片仅显示标题
    let titles: &[&str] = &[
        "系统监控", "代码片段", "待办事项",
        "文件格式转换", "时间戳", "进制转换",
        "Base64", "URL", "JWT", "字符编码", "乱码恢复",
        "JSON Editor", "SQL", "XML",
        "UUID", "文本Hash", "文件校验",
        "假数据生成", "数据库差异",
        "Markdown",
        "IP",
        "Excalidraw",
        "二维码", "剪贴板管理",
        "正则可视化",
        "设置",
    ];
    div()
        .p_6()
        .size_full()
        .child(
            div()
                .flex()
                .flex_wrap()
                .justify_center()
                .gap_2p5()
                .children(titles.iter().enumerate().map(|(i, &title)| {
                    div()
                        .id(ElementId::Name(SharedString::from(format!("home-card-{i}"))))
                        .w(px(240.0))
                        .min_w(px(240.0))
                        .border_1()
                        .border_color(cx.theme().border)
                        .rounded(px(12.0))
                        .p_6()
                        .cursor_pointer()
                        .hover(|style| style.bg(cx.theme().secondary))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            if let Some(view) = view_for_title(title) {
                                this.set_view(view, cx);
                            }
                        }))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_sm()
                                .font_semibold()
                                .child(title),
                        )
                })),
        )
        .child(
            // 匹配 Tauri footer
            div()
                .mt_8()
                .pt_4()
                .border_t_1()
                .border_color(cx.theme().border)
                .text_center()
                .child("ToolBox"),
        )
}


fn view_for_title(title: &str) -> Option<ViewType> {
    match title {
        "系统监控" => Some(ViewType::SystemMonitor),
        "代码片段" => Some(ViewType::CodeSnippet),
        "待办事项" => Some(ViewType::Todo),
        "文件格式转换" => Some(ViewType::TransformFiletype),
        "时间戳" => Some(ViewType::TransformTime),
        "进制转换" => Some(ViewType::TransformBaseConversion),
        "Base64" => Some(ViewType::EncodeDecodeBase64),
        "URL" => Some(ViewType::EncodeDecodeUrl),
        "JWT" => Some(ViewType::EncodeDecodeJwt),
        "字符编码" => Some(ViewType::EncodeDecodeCharset),
        "乱码恢复" => Some(ViewType::EncodeDecodeMessyCode),
        "JSON Editor" => Some(ViewType::FormatterJson),
        "SQL" => Some(ViewType::FormatterSql),
        "XML" => Some(ViewType::FormatterXml),
        "UUID" => Some(ViewType::GeneratorUuid),
        "文本Hash" => Some(ViewType::GeneratorHash),
        "文件校验" => Some(ViewType::GeneratorChecksum),
        "假数据生成" => Some(ViewType::DatabaseDatafaker),
        "数据库差异" => Some(ViewType::DatabaseDiff),
        "Markdown" => Some(ViewType::TextMarkdown),
        "IP" => Some(ViewType::NetworkIp),
        "Excalidraw" => Some(ViewType::ImageExcalidraw),
        "二维码" => Some(ViewType::OtherQrCode),
        "剪贴板管理" => Some(ViewType::OtherClipboard),
        "正则可视化" => Some(ViewType::RegexVisualizer),
        "设置" => Some(ViewType::Settings),
        _ => None,
    }
}

fn render_system_monitor_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.system_monitor.is_none() {
        app.system_monitor = Some(cx.new(|cx| SystemMonitor::new(window, cx)));
    }

    if let Some(ref sys_monitor) = app.system_monitor {
        div().p_6().child(sys_monitor.clone())
    } else {
        div().p_6().child("Loading...")
    }
}

fn render_code_snippet_view(app: &mut App, cx: &mut Context<App>) -> Div {
    if app.code_snippet.is_none() {
        app.code_snippet = Some(cx.new(|_| CodeSnippet::new()));
    }

    if let Some(ref code_snip) = app.code_snippet {
        div().p_6().child(code_snip.clone())
    } else {
        div().p_6().child("Loading...")
    }
}

fn render_todo_view(app: &mut App, cx: &mut Context<App>) -> Div {
    if app.todo_list.is_none() {
        let todo_list = cx.new(|_| TodoList::new());
        app.todo_list = Some(todo_list.clone());
    }

    if let Some(ref todo) = app.todo_list {
        div().p_6().child(todo.clone())
    } else {
        div().p_6().child("Loading...")
    }
}

fn render_transform_filetype_view(
    app: &mut App,
    window: &mut Window,
    cx: &mut Context<App>,
) -> Div {
    if app.transform_filetype.is_none() {
        app.transform_filetype = Some(cx.new(|cx| TransformFiletype::new(window, cx)));
    }

    if let Some(ref transform) = app.transform_filetype {
        div().p_6().child(transform.clone())
    } else {
        div().p_6().child("Loading...")
    }
}

fn render_encode_decode_charset_view(
    app: &mut App,
    window: &mut Window,
    cx: &mut Context<App>,
) -> Div {
    if app.charset_encoder.is_none() {
        app.charset_encoder = Some(cx.new(|cx| CharsetEncoder::new(window, cx)));
    }

    if let Some(ref charset_enc) = app.charset_encoder {
        div().p_6().child(charset_enc.clone())
    } else {
        div().p_6().child("Loading...")
    }
}

fn render_encode_decode_messy_code_view(
    app: &mut App,
    window: &mut Window,
    cx: &mut Context<App>,
) -> Div {
    if app.messy_code_recover.is_none() {
        app.messy_code_recover = Some(cx.new(|cx| MessyCodeRecover::new(window, cx)));
    }

    if let Some(ref messy) = app.messy_code_recover {
        div().p_6().child(messy.clone())
    } else {
        div().p_6().child("Loading...")
    }
}

fn render_formatter_sql_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.sql_formatter.is_none() {
        app.sql_formatter = Some(cx.new(|cx| SqlFormatter::new(window, cx)));
    }

    if let Some(ref sql_fmt) = app.sql_formatter {
        div().p_6().child(sql_fmt.clone())
    } else {
        div().p_6().child("Loading...")
    }
}

fn render_formatter_xml_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.xml_formatter.is_none() {
        app.xml_formatter = Some(cx.new(|cx| XmlFormatter::new(window, cx)));
    }

    if let Some(ref xml_fmt) = app.xml_formatter {
        div().p_6().child(xml_fmt.clone())
    } else {
        div().p_6().child("Loading...")
    }
}

fn render_generator_checksum_view(
    app: &mut App,
    window: &mut Window,
    cx: &mut Context<App>,
) -> Div {
    if app.file_verify.is_none() {
        app.file_verify = Some(cx.new(|cx| FileVerify::new(window, cx)));
    }

    if let Some(ref file_verify) = app.file_verify {
        div().p_6().child(file_verify.clone())
    } else {
        div().p_6().child("Loading...")
    }
}

fn render_database_datafaker_view(
    app: &mut App,
    window: &mut Window,
    cx: &mut Context<App>,
) -> Div {
    if app.fake_data_generator.is_none() {
        app.fake_data_generator = Some(cx.new(|cx| FakeDataGenerator::new(window, cx)));
    }

    if let Some(ref fake_gen) = app.fake_data_generator {
        div().size_full().p_6().child(fake_gen.clone())
    } else {
        div().size_full().p_6().child("Loading...")
    }
}

fn render_database_diff_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.database_diff.is_none() {
        app.database_diff = Some(cx.new(|cx| DatabaseDiff::new(window, cx)));
    }

    if let Some(ref db_diff) = app.database_diff {
        div().p_6().child(db_diff.clone())
    } else {
        div().p_6().child("Loading...")
    }
}

fn render_uuid_generator_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.uuid_generator.is_none() {
        app.uuid_generator = Some(cx.new(|cx| UuidGenerator::new(window, cx)));
    }

    if let Some(ref uuid_gen) = app.uuid_generator {
        div().p_6().child(uuid_gen.clone())
    } else {
        div().p_6().child("Loading...")
    }
}

fn render_hash_calculator_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.hash_calculator.is_none() {
        app.hash_calculator = Some(cx.new(|cx| HashCalculator::new(window, cx)));
    }

    if let Some(ref hash_calc) = app.hash_calculator {
        div().p_6().child(hash_calc.clone())
    } else {
        div().p_6().child("Loading...")
    }
}

fn render_base64_encoder_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.base64_encoder.is_none() {
        app.base64_encoder = Some(cx.new(|cx| Base64Encoder::new(window, cx)));
    }

    if let Some(ref base64_enc) = app.base64_encoder {
        div().p_6().child(base64_enc.clone())
    } else {
        div().p_6().child("Loading...")
    }
}

fn render_url_encoder_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.url_encoder.is_none() {
        app.url_encoder = Some(cx.new(|cx| UrlEncoder::new(window, cx)));
    }

    if let Some(ref url_enc) = app.url_encoder {
        div().p_6().child(url_enc.clone())
    } else {
        div().p_6().child("Loading...")
    }
}

fn render_timestamp_converter_view(
    app: &mut App,
    window: &mut Window,
    cx: &mut Context<App>,
) -> Div {
    if app.timestamp_converter.is_none() {
        app.timestamp_converter = Some(cx.new(|cx| TimestampConverter::new(window, cx)));
    }

    if let Some(ref ts_conv) = app.timestamp_converter {
        div().p_6().child(ts_conv.clone())
    } else {
        div().p_6().child("Loading...")
    }
}

fn render_jwt_decoder_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.jwt_decoder.is_none() {
        app.jwt_decoder = Some(cx.new(|cx| JwtDecoder::new(window, cx)));
    }

    if let Some(ref jwt) = app.jwt_decoder {
        div().p_6().child(jwt.clone())
    } else {
        div().p_6().child("Loading...")
    }
}

fn render_ip_converter_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.ip_converter.is_none() {
        app.ip_converter = Some(cx.new(|cx| IpConverter::new(window, cx)));
    }

    if let Some(ref ip) = app.ip_converter {
        div().p_6().child(ip.clone())
    } else {
        div().p_6().child("Loading...")
    }
}

fn render_qrcode_generator_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.qrcode_generator.is_none() {
        app.qrcode_generator = Some(cx.new(|cx| QrCodeGenerator::new(window, cx)));
    }

    if let Some(ref qrcode) = app.qrcode_generator {
        div().p_6().child(qrcode.clone())
    } else {
        div().p_6().child("Loading...")
    }
}

fn render_clipboard_manager_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.clipboard_manager.is_none() {
        app.clipboard_manager = Some(cx.new(|cx| ClipboardManager::new(window, cx)));
    }

    if let Some(ref clipboard) = app.clipboard_manager {
        div().p_6().child(clipboard.clone())
    } else {
        div().p_6().child("Loading...")
    }
}

fn render_regex_visualizer_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.regex_visualizer.is_none() {
        app.regex_visualizer = Some(cx.new(|cx| RegexVisualizer::new(window, cx)));
    }

    if let Some(ref regex) = app.regex_visualizer {
        div().p_6().child(regex.clone())
    } else {
        div().p_6().child("Loading...")
    }
}

fn render_excalidraw_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.excalidraw.is_none() {
        app.excalidraw = Some(cx.new(|cx| ExcalidrawView::new(window, cx)));
    }

    if let Some(ref excalidraw) = app.excalidraw {
        div().size_full().p_6().child(excalidraw.clone())
    } else {
        div().size_full().p_6().child("Loading...")
    }
}

fn render_settings_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.settings.is_none() {
        app.settings = Some(cx.new(|cx| SettingsView::new(window, cx)));
    }

    if let Some(ref settings) = app.settings {
        div().p_6().child(settings.clone())
    } else {
        div().p_6().child("Loading...")
    }
}

fn render_base_converter_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.base_converter.is_none() {
        app.base_converter = Some(cx.new(|cx| BaseConverter::new(window, cx)));
    }

    if let Some(ref base_conv) = app.base_converter {
        div().p_6().child(base_conv.clone())
    } else {
        div().p_6().child("Loading...")
    }
}

fn render_json_editor_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.json_editor.is_none() {
        app.json_editor = Some(cx.new(|cx| JsonEditor::new(window, cx)));
    }

    if let Some(ref json_edit) = app.json_editor {
        div().p_6().child(json_edit.clone())
    } else {
        div().p_6().child("Loading...")
    }
}

fn render_markdown_editor_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.markdown_editor.is_none() {
        app.markdown_editor = Some(cx.new(|cx| MarkdownEditor::new(window, cx)));
    }

    if let Some(ref md_edit) = app.markdown_editor {
        div().p_6().child(md_edit.clone())
    } else {
        div().p_6().child("Loading...")
    }
}

fn main() {
    sqlx::any::install_default_drivers();

    // 进入 Tokio runtime 上下文，使 sqlx 等库能正常工作
    let _guard = TOKIO_RUNTIME.enter();

    let app = Application::new().with_assets(Assets);

    app.run(move |cx| {
        gpui_component::init(cx);
        let bounds = Bounds::centered(None, size(px(1200.0), px(800.0)), cx);

        cx.spawn(async move |cx| {
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(TitlebarOptions {
                        title: Some("ToolBox".into()),
                        appears_transparent: false,
                        traffic_light_position: None,
                    }),
                    ..Default::default()
                },
                |window, cx| {
                    let view = cx.new(|_| App::new());
                    cx.new(|cx| Root::new(view, window, cx))
                },
            )?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
