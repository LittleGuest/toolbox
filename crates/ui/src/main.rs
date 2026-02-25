use gpui::*;
use gpui_component::{scroll::ScrollableElement, sidebar::*, *};
use gpui_component_assets::Assets;
mod views;
use views::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ViewType {
    Home,
    SystemMonitor,
    CodeSnippet,
    Todo,
    TransformFiletype,
    TransformTime,
    TransformBaseConversion,
    TransformOpenApi,
    EncodeDecodeBase64,
    EncodeDecodeUrl,
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
}

pub struct App {
    current_view: ViewType,
    uuid_generator: Option<Entity<UuidGenerator>>,
    hash_calculator: Option<Entity<HashCalculator>>,
    base64_encoder: Option<Entity<Base64Encoder>>,
    url_encoder: Option<Entity<UrlEncoder>>,
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
    transform_openapi: Option<Entity<TransformOpenapi>>,
}

impl App {
    fn new() -> Self {
        Self {
            current_view: ViewType::Home,
            uuid_generator: None,
            hash_calculator: None,
            base64_encoder: None,
            url_encoder: None,
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
            transform_openapi: None,
        }
    }

    fn set_view(&mut self, view: ViewType, cx: &mut Context<Self>) {
        self.current_view = view;
        cx.notify();
    }
}

impl Render for App {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let current_view = self.current_view;

        div()
            .size_full()
            .flex()
            .overflow_hidden()
            .child(
                Sidebar::new(Side::Left)
                    .min_w(px(220.0))
                    .max_w(px(280.0))
                    .children([
                        SidebarGroup::new("导航").children([SidebarMenu::new().children([
                            SidebarMenuItem::new("首页").on_click(cx.listener(|this, _, _, cx| {
                                this.set_view(ViewType::Home, cx);
                            })),
                            SidebarMenuItem::new("系统监控").on_click(cx.listener(
                                |this, _, _, cx| {
                                    this.set_view(ViewType::SystemMonitor, cx);
                                },
                            )),
                            SidebarMenuItem::new("代码片段").on_click(cx.listener(
                                |this, _, _, cx| {
                                    this.set_view(ViewType::CodeSnippet, cx);
                                },
                            )),
                            SidebarMenuItem::new("待办事项").on_click(cx.listener(
                                |this, _, _, cx| {
                                    this.set_view(ViewType::Todo, cx);
                                },
                            )),
                        ])]),
                        SidebarGroup::new("转换").children([SidebarMenu::new().children([
                            SidebarMenuItem::new("文件格式转换").on_click(cx.listener(
                                |this, _, _, cx| {
                                    this.set_view(ViewType::TransformFiletype, cx);
                                },
                            )),
                            SidebarMenuItem::new("时间戳").on_click(cx.listener(
                                |this, _, _, cx| {
                                    this.set_view(ViewType::TransformTime, cx);
                                },
                            )),
                            SidebarMenuItem::new("进制转换").on_click(cx.listener(
                                |this, _, _, cx| {
                                    this.set_view(ViewType::TransformBaseConversion, cx);
                                },
                            )),
                            SidebarMenuItem::new("OpenApi").on_click(cx.listener(
                                |this, _, _, cx| {
                                    this.set_view(ViewType::TransformOpenApi, cx);
                                },
                            )),
                        ])]),
                        SidebarGroup::new("编码/解码").children([SidebarMenu::new().children([
                            SidebarMenuItem::new("Base64").on_click(cx.listener(
                                |this, _, _, cx| {
                                    this.set_view(ViewType::EncodeDecodeBase64, cx);
                                },
                            )),
                            SidebarMenuItem::new("URL").on_click(cx.listener(|this, _, _, cx| {
                                this.set_view(ViewType::EncodeDecodeUrl, cx);
                            })),
                            SidebarMenuItem::new("字符编码").on_click(cx.listener(
                                |this, _, _, cx| {
                                    this.set_view(ViewType::EncodeDecodeCharset, cx);
                                },
                            )),
                            SidebarMenuItem::new("乱码恢复").on_click(cx.listener(
                                |this, _, _, cx| {
                                    this.set_view(ViewType::EncodeDecodeMessyCode, cx);
                                },
                            )),
                        ])]),
                        SidebarGroup::new("格式化").children([SidebarMenu::new().children([
                            SidebarMenuItem::new("JSON Editor").on_click(cx.listener(
                                |this, _, _, cx| {
                                    this.set_view(ViewType::FormatterJson, cx);
                                },
                            )),
                            SidebarMenuItem::new("SQL").on_click(cx.listener(|this, _, _, cx| {
                                this.set_view(ViewType::FormatterSql, cx);
                            })),
                            SidebarMenuItem::new("XML").on_click(cx.listener(|this, _, _, cx| {
                                this.set_view(ViewType::FormatterXml, cx);
                            })),
                        ])]),
                        SidebarGroup::new("生成器").children([SidebarMenu::new().children([
                            SidebarMenuItem::new("UUID").on_click(cx.listener(|this, _, _, cx| {
                                this.set_view(ViewType::GeneratorUuid, cx);
                            })),
                            SidebarMenuItem::new("文本Hash").on_click(cx.listener(
                                |this, _, _, cx| {
                                    this.set_view(ViewType::GeneratorHash, cx);
                                },
                            )),
                            SidebarMenuItem::new("文件校验").on_click(cx.listener(
                                |this, _, _, cx| {
                                    this.set_view(ViewType::GeneratorChecksum, cx);
                                },
                            )),
                        ])]),
                        SidebarGroup::new("数据库").children([SidebarMenu::new().children([
                            SidebarMenuItem::new("假数据生成").on_click(cx.listener(
                                |this, _, _, cx| {
                                    this.set_view(ViewType::DatabaseDatafaker, cx);
                                },
                            )),
                            SidebarMenuItem::new("数据库差异").on_click(cx.listener(
                                |this, _, _, cx| {
                                    this.set_view(ViewType::DatabaseDiff, cx);
                                },
                            )),
                        ])]),
                        SidebarGroup::new("文本").children([SidebarMenu::new().children([
                            SidebarMenuItem::new("Markdown").on_click(cx.listener(
                                |this, _, _, cx| {
                                    this.set_view(ViewType::TextMarkdown, cx);
                                },
                            )),
                        ])]),
                    ]),
            )
            .child(
                div()
                    .flex_1()
                    .overflow_y_scrollbar()
                    .child(match current_view {
                        ViewType::Home => render_home_view(cx),
                        ViewType::SystemMonitor => render_system_monitor_view(self, cx),
                        ViewType::CodeSnippet => render_code_snippet_view(self, cx),
                        ViewType::Todo => render_todo_view(self, cx),
                        ViewType::TransformFiletype => render_transform_filetype_view(self, window, cx),
                        ViewType::TransformTime => render_timestamp_converter_view(self, window, cx),
                        ViewType::TransformBaseConversion => render_base_converter_view(self, window, cx),
                        ViewType::TransformOpenApi => render_transform_openapi_view(self, window, cx),
                        ViewType::EncodeDecodeBase64 => {
                            render_base64_encoder_view(self, window, cx)
                        }
                        ViewType::EncodeDecodeUrl => render_url_encoder_view(self, window, cx),
                        ViewType::EncodeDecodeCharset => {
                            render_encode_decode_charset_view(self, window, cx)
                        }
                        ViewType::EncodeDecodeMessyCode => {
                            render_encode_decode_messy_code_view(self, window, cx)
                        }
                        ViewType::FormatterJson => render_json_editor_view(self, window, cx),
                        ViewType::FormatterSql => render_formatter_sql_view(self, window, cx),
                        ViewType::FormatterXml => render_formatter_xml_view(self, window, cx),
                        ViewType::GeneratorUuid => render_uuid_generator_view(self, window, cx),
                        ViewType::GeneratorHash => render_hash_calculator_view(self, window, cx),
                        ViewType::GeneratorChecksum => render_generator_checksum_view(cx),
                        ViewType::DatabaseDatafaker => render_database_datafaker_view(self, window, cx),
                        ViewType::DatabaseDiff => render_database_diff_view(self, cx),
                        ViewType::TextMarkdown => render_markdown_editor_view(self, window, cx),
                    }),
            )
    }
}

fn render_home_view(cx: &mut Context<App>) -> Div {
    div()
        .p_4()
        .gap_4()
        .child(
            div()
                .text_xl()
                .font_semibold()
                .mb_4()
                .child("欢迎使用 Toolbox"),
        )
        .child(div().grid().grid_cols(3).gap_4().children([
            create_menu_card("系统监控", "查看系统资源使用情况", cx),
            create_menu_card("代码片段", "管理常用代码片段", cx),
            create_menu_card("待办事项", "管理待办任务", cx),
            create_menu_card("文件格式转换", "转换文件格式", cx),
            create_menu_card("时间戳", "时间戳转换工具", cx),
            create_menu_card("进制转换", "进制转换工具", cx),
            create_menu_card("OpenApi", "OpenApi 工具", cx),
            create_menu_card("Base64", "Base64 编码解码", cx),
            create_menu_card("URL", "URL 编码解码", cx),
            create_menu_card("字符编码", "字符编码转换", cx),
            create_menu_card("乱码恢复", "乱码恢复工具", cx),
            create_menu_card("JSON Editor", "JSON 格式化编辑", cx),
            create_menu_card("SQL", "SQL 格式化", cx),
            create_menu_card("XML", "XML 格式化", cx),
            create_menu_card("UUID", "UUID 生成器", cx),
            create_menu_card("文本Hash", "文本 Hash 计算", cx),
            create_menu_card("文件校验", "文件校验和计算", cx),
            create_menu_card("假数据生成", "数据库假数据生成", cx),
            create_menu_card("数据库差异", "数据库差异对比", cx),
            create_menu_card("Markdown", "Markdown 编辑器", cx),
        ]))
}

fn create_menu_card(title: &'static str, description: &'static str, cx: &mut Context<App>) -> Div {
    div()
        .border_1()
        .border_color(cx.theme().border)
        .rounded_lg()
        .p_4()
        .cursor_pointer()
        .hover(|style| style.bg(cx.theme().secondary))
        .child(div().text_lg().font_semibold().mb_2().child(title))
        .child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(description),
        )
}

fn render_system_monitor_view(app: &mut App, cx: &mut Context<App>) -> Div {
    if app.system_monitor.is_none() {
        app.system_monitor = Some(cx.new(|_| SystemMonitor::new()));
    }

    if let Some(ref sys_monitor) = app.system_monitor {
        div().p_4().child(sys_monitor.clone())
    } else {
        div().p_4().child("Loading...")
    }
}

fn render_code_snippet_view(app: &mut App, cx: &mut Context<App>) -> Div {
    if app.code_snippet.is_none() {
        app.code_snippet = Some(cx.new(|_| CodeSnippet::new()));
    }

    if let Some(ref code_snip) = app.code_snippet {
        div().p_4().child(code_snip.clone())
    } else {
        div().p_4().child("Loading...")
    }
}

fn render_todo_view(app: &mut App, cx: &mut Context<App>) -> Div {
    if app.todo_list.is_none() {
        let todo_list = cx.new(|_| TodoList::new());
        app.todo_list = Some(todo_list.clone());
    }

    if let Some(ref todo) = app.todo_list {
        div().p_4().child(todo.clone())
    } else {
        div().p_4().child("Loading...")
    }
}

fn render_transform_filetype_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.transform_filetype.is_none() {
        app.transform_filetype = Some(cx.new(|cx| TransformFiletype::new(window, cx)));
    }

    if let Some(ref transform) = app.transform_filetype {
        div().p_4().child(transform.clone())
    } else {
        div().p_4().child("Loading...")
    }
}

fn render_transform_openapi_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.transform_openapi.is_none() {
        app.transform_openapi = Some(cx.new(|cx| TransformOpenapi::new(window, cx)));
    }

    if let Some(ref openapi) = app.transform_openapi {
        div().p_4().child(openapi.clone())
    } else {
        div().p_4().child("Loading...")
    }
}

fn render_encode_decode_charset_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.charset_encoder.is_none() {
        app.charset_encoder = Some(cx.new(|cx| CharsetEncoder::new(window, cx)));
    }

    if let Some(ref charset_enc) = app.charset_encoder {
        div().p_4().child(charset_enc.clone())
    } else {
        div().p_4().child("Loading...")
    }
}

fn render_encode_decode_messy_code_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.messy_code_recover.is_none() {
        app.messy_code_recover = Some(cx.new(|cx| MessyCodeRecover::new(window, cx)));
    }

    if let Some(ref messy) = app.messy_code_recover {
        div().p_4().child(messy.clone())
    } else {
        div().p_4().child("Loading...")
    }
}

fn render_formatter_sql_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.sql_formatter.is_none() {
        app.sql_formatter = Some(cx.new(|cx| SqlFormatter::new(window, cx)));
    }

    if let Some(ref sql_fmt) = app.sql_formatter {
        div().p_4().child(sql_fmt.clone())
    } else {
        div().p_4().child("Loading...")
    }
}

fn render_formatter_xml_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.xml_formatter.is_none() {
        app.xml_formatter = Some(cx.new(|cx| XmlFormatter::new(window, cx)));
    }

    if let Some(ref xml_fmt) = app.xml_formatter {
        div().p_4().child(xml_fmt.clone())
    } else {
        div().p_4().child("Loading...")
    }
}

fn render_generator_checksum_view(cx: &mut Context<App>) -> Div {
    div()
        .p_4()
        .child(div().text_xl().font_semibold().mb_4().child("文件校验"))
        .child(
            div()
                .p_4()
                .border_1()
                .border_color(cx.theme().border)
                .rounded_lg()
                .child("文件校验功能开发中..."),
        )
}

fn render_database_datafaker_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.fake_data_generator.is_none() {
        app.fake_data_generator = Some(cx.new(|cx| FakeDataGenerator::new(window, cx)));
    }

    if let Some(ref fake_gen) = app.fake_data_generator {
        div().p_4().child(fake_gen.clone())
    } else {
        div().p_4().child("Loading...")
    }
}

fn render_database_diff_view(app: &mut App, cx: &mut Context<App>) -> Div {
    if app.database_diff.is_none() {
        app.database_diff = Some(cx.new(|_| DatabaseDiff::new()));
    }

    if let Some(ref db_diff) = app.database_diff {
        div().p_4().child(db_diff.clone())
    } else {
        div().p_4().child("Loading...")
    }
}

fn render_uuid_generator_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.uuid_generator.is_none() {
        app.uuid_generator = Some(cx.new(|cx| UuidGenerator::new(window, cx)));
    }

    if let Some(ref uuid_gen) = app.uuid_generator {
        div().p_4().child(uuid_gen.clone())
    } else {
        div().p_4().child("Loading...")
    }
}

fn render_hash_calculator_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.hash_calculator.is_none() {
        app.hash_calculator = Some(cx.new(|cx| HashCalculator::new(window, cx)));
    }

    if let Some(ref hash_calc) = app.hash_calculator {
        div().p_4().child(hash_calc.clone())
    } else {
        div().p_4().child("Loading...")
    }
}

fn render_base64_encoder_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.base64_encoder.is_none() {
        app.base64_encoder = Some(cx.new(|cx| Base64Encoder::new(window, cx)));
    }

    if let Some(ref base64_enc) = app.base64_encoder {
        div().p_4().child(base64_enc.clone())
    } else {
        div().p_4().child("Loading...")
    }
}

fn render_url_encoder_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.url_encoder.is_none() {
        app.url_encoder = Some(cx.new(|cx| UrlEncoder::new(window, cx)));
    }

    if let Some(ref url_enc) = app.url_encoder {
        div().p_4().child(url_enc.clone())
    } else {
        div().p_4().child("Loading...")
    }
}

fn render_timestamp_converter_view(app: &mut App, _window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.timestamp_converter.is_none() {
        app.timestamp_converter = Some(cx.new(|_cx| TimestampConverter::new()));
    }

    if let Some(ref ts_conv) = app.timestamp_converter {
        div().p_4().child(ts_conv.clone())
    } else {
        div().p_4().child("Loading...")
    }
}

fn render_base_converter_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.base_converter.is_none() {
        app.base_converter = Some(cx.new(|cx| BaseConverter::new(window, cx)));
    }

    if let Some(ref base_conv) = app.base_converter {
        div().p_4().child(base_conv.clone())
    } else {
        div().p_4().child("Loading...")
    }
}

fn render_json_editor_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.json_editor.is_none() {
        app.json_editor = Some(cx.new(|cx| JsonEditor::new(window, cx)));
    }

    if let Some(ref json_edit) = app.json_editor {
        div().p_4().child(json_edit.clone())
    } else {
        div().p_4().child("Loading...")
    }
}

fn render_markdown_editor_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.markdown_editor.is_none() {
        app.markdown_editor = Some(cx.new(|cx| MarkdownEditor::new(window, cx)));
    }

    if let Some(ref md_edit) = app.markdown_editor {
        div().p_4().child(md_edit.clone())
    } else {
        div().p_4().child("Loading...")
    }
}

fn main() {
    let app = Application::new().with_assets(Assets);

    app.run(move |cx| {
        gpui_component::init(cx);
        let bounds = Bounds::centered(None, size(px(1000.0), px(800.0)), cx);

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
