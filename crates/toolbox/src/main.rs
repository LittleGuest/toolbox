#![recursion_limit = "1024"]

use std::sync::LazyLock;

use gpui_kit::{
    assets::Assets as ComponentAssets,
    component::{
        Theme, ThemeMode,
        button::*,
        command::{Command, CommandItem, CommandState},
        scroll::ScrollableElement,
        sidebar::*,
        *,
    },
    prelude::FluentBuilder,
    *,
};
mod config_store;
mod excalidraw_store;
mod search_pinyin;
mod views;
use search_pinyin::{TOOL_INDEX, label_pinyin, label_pinyin_initials};
use views::*;

struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> gpui_kit::Result<Option<std::borrow::Cow<'static, [u8]>>> {
        gpui_excalidraw::ui::icons::IconAssets.load(path, |path| ComponentAssets.load(path))
    }

    fn list(&self, path: &str) -> gpui_kit::Result<Vec<SharedString>> {
        gpui_excalidraw::ui::icons::IconAssets.list_with(path, |path| ComponentAssets.list(path))
    }
}

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
    TransformCron,
    TransformRmb,
    EncodeDecodeUrl,
    EncodeDecodeJwt,
    EncodeDecodeCharset,
    EncodeDecodeBaseEncoding,
    EncodeDecodeSymmetric,
    EncodeDecodeSmCrypto,
    EncodeDecodeEncodingFormat,
    FormatterJson,
    FormatterJsonSchema,
    FormatterSql,
    FormatterXml,
    GeneratorUuid,
    GeneratorHash,
    DatabaseDatafaker,
    DatabaseDiff,
    TextMarkdown,
    TextTextTools,
    TextDiff,
    RandomString,
    RandomNumber,
    RandomOther,
    RandomData,
    NumberBitwise,
    ColorConverter,
    NetworkIp,
    ImageExcalidraw,
    ImageConvert,
    PdfImagesToPdf,
    PdfMerge,
    PdfEdit,
    PdfPageNumber,
    PdfSplit,
    OtherQrCode,
    OtherQrCodeDecode,
    OtherClipboard,
    RegexVisualizer,
}

pub struct App {
    current_view: ViewType,
    sidebar_collapsed: bool,
    uuid_generator: Option<Entity<UuidGenerator>>,
    url_encoder: Option<Entity<UrlEncoder>>,
    jwt_decoder: Option<Entity<JwtDecoder>>,
    timestamp_converter: Option<Entity<TimestampConverter>>,
    base_converter: Option<Entity<BaseConverter>>,
    json_editor: Option<Entity<JsonEditor>>,
    markdown_editor: Option<Entity<MarkdownEditor>>,
    code_snippet: Option<Entity<CodeSnippet>>,
    todo_list: Option<Entity<TodoList>>,
    sql_formatter: Option<Entity<SqlFormatter>>,
    xml_formatter: Option<Entity<XmlFormatter>>,
    fake_data_generator: Option<Entity<FakeDataGenerator>>,
    database_diff: Option<Entity<DatabaseDiff>>,
    system_monitor: Option<Entity<SystemMonitor>>,
    transform_filetype: Option<Entity<TransformFiletype>>,
    ip_converter: Option<Entity<IpConverter>>,
    qrcode_generator: Option<Entity<QrCodeGenerator>>,
    clipboard_manager: Option<Entity<ClipboardManager>>,
    regex_visualizer: Option<Entity<RegexVisualizer>>,
    excalidraw: Option<Entity<ExcalidrawView>>,
    cron_converter: Option<Entity<CronConverter>>,
    base_encoding: Option<Entity<BaseEncodingConverter>>,
    symmetric_encrypt: Option<Entity<SymmetricEncryptor>>,
    sm_crypto: Option<Entity<SmCrypto>>,
    encoding_format: Option<Entity<EncodingFormatConvert>>,
    json_schema: Option<Entity<JsonSchemaValidator>>,
    qrcode_decoder: Option<Entity<QrCodeDecoder>>,
    text_tools: Option<Entity<TextTools>>,
    text_diff: Option<Entity<TextDiffTool>>,
    random_string: Option<Entity<RandomStringGenerator>>,
    random_number: Option<Entity<RandomNumberGenerator>>,
    random_other: Option<Entity<RandomOtherGenerator>>,
    random_data: Option<Entity<RandomDataGenerator>>,
    bitwise_calculator: Option<Entity<BitwiseCalculator>>,
    color_converter: Option<Entity<ColorConverter>>,
    rmb_case: Option<Entity<RmbCase>>,
    image_convert: Option<Entity<ImageConvert>>,
    pdf_images_to_pdf: Option<Entity<PdfFromFiles>>,
    pdf_merge: Option<Entity<PdfFromFiles>>,
    pdf_edit: Option<Entity<PdfEditTool>>,
    pdf_page_number: Option<Entity<PdfPageNumberTool>>,
    pdf_split: Option<Entity<PdfSplitTool>>,
    text_encoding: Option<Entity<TextEncodingPage>>,
    hash_page: Option<Entity<HashPage>>,
    command_state: Option<Entity<CommandState>>,
}

impl App {
    fn new() -> Self {
        Self {
            current_view: ViewType::Home,
            sidebar_collapsed: false,
            uuid_generator: None,
            url_encoder: None,
            jwt_decoder: None,
            timestamp_converter: None,
            base_converter: None,
            json_editor: None,
            markdown_editor: None,
            code_snippet: None,
            todo_list: None,
            sql_formatter: None,
            xml_formatter: None,
            fake_data_generator: None,
            database_diff: None,
            system_monitor: None,
            transform_filetype: None,
            ip_converter: None,
            qrcode_generator: None,
            clipboard_manager: None,
            regex_visualizer: None,
            excalidraw: None,
            cron_converter: None,
            base_encoding: None,
            symmetric_encrypt: None,
            sm_crypto: None,
            encoding_format: None,
            json_schema: None,
            qrcode_decoder: None,
            text_tools: None,
            text_diff: None,
            random_string: None,
            random_number: None,
            random_other: None,
            random_data: None,
            bitwise_calculator: None,
            color_converter: None,
            rmb_case: None,
            image_convert: None,
            pdf_images_to_pdf: None,
            pdf_merge: None,
            pdf_edit: None,
            pdf_page_number: None,
            pdf_split: None,
            text_encoding: None,
            hash_page: None,
            command_state: None,
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

    fn toggle_theme(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let new_mode = if cx.theme().mode.is_dark() {
            ThemeMode::Light
        } else {
            ThemeMode::Dark
        };
        Theme::change(new_mode, Some(window), cx);
        let value = if new_mode.is_dark() { "dark" } else { "light" };
        cx.spawn(async move |this, cx| {
            let _ = config_store::set_setting("theme", value).await;
            let _ = this.update(cx, |_, _| {});
        })
        .detach();
        cx.notify();
    }

    fn open_search(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.command_state.is_none() {
            self.command_state = Some(cx.new(|cx| CommandState::new(window, cx)));
        }
        let state = self.command_state.clone().unwrap();

        state.update(cx, |s, cx| s.set_query("", window, cx));
        let weak = cx.entity().downgrade();
        window.open_sheet_at(Placement::Top, cx, move |sheet, _, _cx| {
            sheet
                .overlay(false)
                .overlay_closable(true)
                .size(px(640.))
                .child(
                    Command::new(&state)
                        .searchable(true)
                        .filterable(true)
                        .placeholder("搜索工具，快速切换…")
                        .max_h(px(420.0))
                        .items(TOOL_INDEX.iter().map(|tool| {
                            let label: SharedString = if tool.group.is_empty() {
                                tool.label.into()
                            } else {
                                format!("{}（{}）", tool.label, tool.group).into()
                            };
                            let pinyin = label_pinyin(tool.label);
                            let initials = label_pinyin_initials(tool.label);

                            let mut keywords: Vec<String> = Vec::new();
                            if !tool.group.is_empty() {
                                keywords.push(tool.group.to_string());
                            }
                            if !tool.key.is_empty() {
                                keywords.push(tool.key.to_string());
                            }
                            keywords.push(pinyin);
                            keywords.push(initials);
                            CommandItem::new()
                                .label(label)
                                .icon(Icon::new(tool.icon.clone()))
                                .keywords(keywords)
                        }))
                        .on_confirm({
                            let weak = weak.clone();
                            move |index, window, cx| {
                                if index.section == 0 {
                                    let row = index.row;
                                    if let Some(this) = weak.upgrade()
                                        && let Some(view) =
                                            TOOL_INDEX.get(row).map(|tool| tool.view)
                                    {
                                        this.update(cx, |this, cx| {
                                            this.set_view(view, cx);
                                        });
                                    }
                                    window.close_sheet(cx);
                                }
                            }
                        }),
                )
        });
    }

    fn render_topbar(&self, cx: &Context<Self>) -> impl IntoElement {
        div()
            .flex_shrink_0()
            .h(px(56.0))
            .flex()
            .items_center()
            .justify_center()
            .px(px(20.0))
            .border_b_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().popover)
            .child(
                div()
                    .id("topbar-search")
                    .w_full()
                    .max_w(px(560.0))
                    .h(px(38.0))
                    .flex()
                    .items_center()
                    .gap_2()
                    .px(px(12.0))
                    .rounded(px(8.0))
                    .border_1()
                    .border_color(cx.theme().border)
                    .bg(cx.theme().background)
                    .text_color(cx.theme().muted_foreground)
                    .text_size(px(14.0))
                    .cursor_pointer()
                    .hover(|s| s.border_color(cx.theme().primary.opacity(0.45)))
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.open_search(window, cx);
                    }))
                    .child(Icon::new(IconName::Search))
                    .child("搜索工具，快速切换…"),
            )
    }
}

impl Render for App {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let current_view = self.current_view;

        div().size_full().relative().child(
            div()
                .size_full()
                .flex()
                .overflow_hidden()
                .child(
                    Sidebar::new("main-sidebar")
                        .side(Side::Left)
                        .collapsed(self.sidebar_collapsed)
                        .header(
                            SidebarHeader::new()
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .child(
                                            SidebarToggleButton::new()
                                                .side(Side::Left)
                                                .collapsed(self.sidebar_collapsed)
                                                .on_click(cx.listener(|this, _, _, cx| {
                                                    this.toggle_sidebar(cx);
                                                })),
                                        )
                                        .when(!self.sidebar_collapsed, |this| {
                                            this.child(
                                                div()
                                                    .text_size(px(16.0))
                                                    .font_weight(FontWeight::SEMIBOLD)
                                                    .child("ToolBox"),
                                            )
                                        }),
                                )
                                .child(
                                    Button::new("theme-toggle")
                                        .ghost()
                                        .small()
                                        .icon(Icon::new(if cx.theme().mode.is_dark() {
                                            IconName::Sun
                                        } else {
                                            IconName::Moon
                                        }))
                                        .tooltip(if cx.theme().mode.is_dark() {
                                            "切换浅色模式"
                                        } else {
                                            "切换深色模式"
                                        })
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.toggle_theme(window, cx);
                                        })),
                                ),
                        )
                        .children([SidebarMenu::new().children([
                            SidebarMenuItem::new("首页")
                                .icon(Icon::new(IconName::LayoutDashboard))
                                .active(current_view == ViewType::Home)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.set_view(ViewType::Home, cx);
                                })),
                            SidebarMenuItem::new("系统监控")
                                .icon(Icon::new(IconName::ChartPie))
                                .active(current_view == ViewType::SystemMonitor)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.set_view(ViewType::SystemMonitor, cx);
                                })),
                            SidebarMenuItem::new("代码片段")
                                .icon(Icon::new(IconName::FileText))
                                .active(current_view == ViewType::CodeSnippet)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.set_view(ViewType::CodeSnippet, cx);
                                })),
                            SidebarMenuItem::new("待办事项")
                                .icon(Icon::new(IconName::Check))
                                .active(current_view == ViewType::Todo)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.set_view(ViewType::Todo, cx);
                                })),
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
                                            this.set_view(ViewType::TransformBaseConversion, cx);
                                        })),
                                    SidebarMenuItem::new("Cron 表达式")
                                        .icon(Icon::new(IconName::Calendar))
                                        .active(current_view == ViewType::TransformCron)
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.set_view(ViewType::TransformCron, cx);
                                        })),
                                    SidebarMenuItem::new("人民币大小写")
                                        .icon(Icon::new(IconName::ALargeSmall))
                                        .active(current_view == ViewType::TransformRmb)
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.set_view(ViewType::TransformRmb, cx);
                                        })),
                                ]),
                            SidebarMenuItem::new("编码/解码")
                                .icon(Icon::new(IconName::Dash))
                                .click_to_open(true)
                                .children([
                                    SidebarMenuItem::new("Base 编码")
                                        .icon(Icon::new(IconName::CaseSensitive))
                                        .active(current_view == ViewType::EncodeDecodeBaseEncoding)
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.set_view(ViewType::EncodeDecodeBaseEncoding, cx);
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
                                    SidebarMenuItem::new("文本编码")
                                        .icon(Icon::new(IconName::CaseSensitive))
                                        .active(current_view == ViewType::EncodeDecodeCharset)
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.set_view(ViewType::EncodeDecodeCharset, cx);
                                        })),
                                    SidebarMenuItem::new("编码格式转换")
                                        .icon(Icon::new(IconName::Replace))
                                        .active(
                                            current_view == ViewType::EncodeDecodeEncodingFormat,
                                        )
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.set_view(ViewType::EncodeDecodeEncodingFormat, cx);
                                        })),
                                    SidebarMenuItem::new("二维码")
                                        .icon(Icon::new(IconName::Frame))
                                        .active(current_view == ViewType::OtherQrCode)
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.set_view(ViewType::OtherQrCode, cx);
                                        })),
                                    SidebarMenuItem::new("二维码解码")
                                        .icon(Icon::new(IconName::Search))
                                        .active(current_view == ViewType::OtherQrCodeDecode)
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.set_view(ViewType::OtherQrCodeDecode, cx);
                                        })),
                                ]),
                            SidebarMenuItem::new("加密")
                                .icon(Icon::new(IconName::EyeOff))
                                .click_to_open(true)
                                .children([
                                    SidebarMenuItem::new("对称加密")
                                        .icon(Icon::new(IconName::EyeOff))
                                        .active(current_view == ViewType::EncodeDecodeSymmetric)
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.set_view(ViewType::EncodeDecodeSymmetric, cx);
                                        })),
                                    SidebarMenuItem::new("国密算法")
                                        .icon(Icon::new(IconName::Asterisk))
                                        .active(current_view == ViewType::EncodeDecodeSmCrypto)
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.set_view(ViewType::EncodeDecodeSmCrypto, cx);
                                        })),
                                ]),
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
                                    SidebarMenuItem::new("JSON Schema")
                                        .icon(Icon::new(IconName::Check))
                                        .active(current_view == ViewType::FormatterJsonSchema)
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.set_view(ViewType::FormatterJsonSchema, cx);
                                        })),
                                ]),
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
                                    SidebarMenuItem::new("Hash 计算")
                                        .icon(Icon::new(IconName::Asterisk))
                                        .active(current_view == ViewType::GeneratorHash)
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.set_view(ViewType::GeneratorHash, cx);
                                        })),
                                ]),
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
                            SidebarMenuItem::new("文本")
                                .icon(Icon::new(IconName::BookOpen))
                                .click_to_open(true)
                                .children([
                                    SidebarMenuItem::new("Markdown")
                                        .icon(Icon::new(IconName::BookOpen))
                                        .active(current_view == ViewType::TextMarkdown)
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.set_view(ViewType::TextMarkdown, cx);
                                        })),
                                    SidebarMenuItem::new("文本工具")
                                        .icon(Icon::new(IconName::BookOpen))
                                        .active(current_view == ViewType::TextTextTools)
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.set_view(ViewType::TextTextTools, cx);
                                        })),
                                    SidebarMenuItem::new("文本 / JSON 差异")
                                        .icon(Icon::new(IconName::Replace))
                                        .active(current_view == ViewType::TextDiff)
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.set_view(ViewType::TextDiff, cx);
                                        })),
                                ]),
                            SidebarMenuItem::new("随机")
                                .icon(Icon::new(IconName::Asterisk))
                                .click_to_open(true)
                                .children([
                                    SidebarMenuItem::new("随机字符串")
                                        .icon(Icon::new(IconName::CaseSensitive))
                                        .active(current_view == ViewType::RandomString)
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.set_view(ViewType::RandomString, cx);
                                        })),
                                    SidebarMenuItem::new("随机数字")
                                        .icon(Icon::new(IconName::Asterisk))
                                        .active(current_view == ViewType::RandomNumber)
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.set_view(ViewType::RandomNumber, cx);
                                        })),
                                    SidebarMenuItem::new("随机数据")
                                        .icon(Icon::new(IconName::Folder))
                                        .active(current_view == ViewType::RandomData)
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.set_view(ViewType::RandomData, cx);
                                        })),
                                ]),
                            SidebarMenuItem::new("网络")
                                .icon(Icon::new(IconName::Network))
                                .click_to_open(true)
                                .children([
                                    SidebarMenuItem::new("IP 地址转换")
                                        .icon(Icon::new(IconName::Network))
                                        .active(current_view == ViewType::NetworkIp)
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.set_view(ViewType::NetworkIp, cx);
                                        })),
                                    SidebarMenuItem::new("随机 IP / MAC / 时间")
                                        .icon(Icon::new(IconName::Calendar))
                                        .active(current_view == ViewType::RandomOther)
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.set_view(ViewType::RandomOther, cx);
                                        })),
                                ]),
                            SidebarMenuItem::new("按位计算器")
                                .icon(Icon::new(IconName::SquareTerminal))
                                .active(current_view == ViewType::NumberBitwise)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.set_view(ViewType::NumberBitwise, cx);
                                })),
                            SidebarMenuItem::new("颜色转换")
                                .icon(Icon::new(IconName::Palette))
                                .active(current_view == ViewType::ColorConverter)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.set_view(ViewType::ColorConverter, cx);
                                })),
                            SidebarMenuItem::new("正则")
                                .icon(Icon::new(IconName::Dash))
                                .active(current_view == ViewType::RegexVisualizer)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.set_view(ViewType::RegexVisualizer, cx);
                                })),
                            SidebarMenuItem::new("图像")
                                .icon(Icon::new(IconName::Frame))
                                .click_to_open(true)
                                .children([
                                    SidebarMenuItem::new("图片格式转换")
                                        .icon(Icon::new(IconName::Frame))
                                        .active(current_view == ViewType::ImageConvert)
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.set_view(ViewType::ImageConvert, cx);
                                        })),
                                    SidebarMenuItem::new("Excalidraw")
                                        .icon(Icon::new(IconName::Frame))
                                        .active(current_view == ViewType::ImageExcalidraw)
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.set_view(ViewType::ImageExcalidraw, cx);
                                        })),
                                ]),
                            SidebarMenuItem::new("PDF")
                                .icon(Icon::new(IconName::File))
                                .click_to_open(true)
                                .children([
                                    SidebarMenuItem::new("图片转 PDF")
                                        .icon(Icon::new(IconName::File))
                                        .active(current_view == ViewType::PdfImagesToPdf)
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.set_view(ViewType::PdfImagesToPdf, cx);
                                        })),
                                    SidebarMenuItem::new("PDF 合并")
                                        .icon(Icon::new(IconName::File))
                                        .active(current_view == ViewType::PdfMerge)
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.set_view(ViewType::PdfMerge, cx);
                                        })),
                                    SidebarMenuItem::new("PDF 编辑")
                                        .icon(Icon::new(IconName::File))
                                        .active(current_view == ViewType::PdfEdit)
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.set_view(ViewType::PdfEdit, cx);
                                        })),
                                    SidebarMenuItem::new("PDF 添加页码")
                                        .icon(Icon::new(IconName::File))
                                        .active(current_view == ViewType::PdfPageNumber)
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.set_view(ViewType::PdfPageNumber, cx);
                                        })),
                                    SidebarMenuItem::new("PDF 拆分")
                                        .icon(Icon::new(IconName::File))
                                        .active(current_view == ViewType::PdfSplit)
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.set_view(ViewType::PdfSplit, cx);
                                        })),
                                ]),
                            SidebarMenuItem::new("其它")
                                .icon(Icon::new(IconName::Settings2))
                                .click_to_open(true)
                                .children([SidebarMenuItem::new("剪贴板管理")
                                    .icon(Icon::new(IconName::Settings2))
                                    .active(current_view == ViewType::OtherClipboard)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.set_view(ViewType::OtherClipboard, cx);
                                    }))]),
                        ])]),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w(px(640.0))
                        .flex()
                        .flex_col()
                        .overflow_hidden()
                        .child(self.render_topbar(cx))
                        .child(
                            div()
                                .flex_1()
                                .overflow_y_scrollbar()
                                .bg(cx.theme().background)
                                .text_color(cx.theme().foreground)
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
                                    ViewType::TransformCron => render_cron_view(self, window, cx),
                                    ViewType::TransformRmb => {
                                        render_rmb_case_view(self, window, cx)
                                    }
                                    ViewType::EncodeDecodeUrl => {
                                        render_url_encoder_view(self, window, cx)
                                    }
                                    ViewType::EncodeDecodeJwt => {
                                        render_jwt_decoder_view(self, window, cx)
                                    }
                                    ViewType::EncodeDecodeCharset => {
                                        render_text_encoding_page(self, window, cx)
                                    }
                                    ViewType::EncodeDecodeBaseEncoding => {
                                        render_base_encoding_view(self, window, cx)
                                    }
                                    ViewType::EncodeDecodeSymmetric => {
                                        render_symmetric_encrypt_view(self, window, cx)
                                    }
                                    ViewType::EncodeDecodeSmCrypto => {
                                        render_sm_crypto_view(self, window, cx)
                                    }
                                    ViewType::EncodeDecodeEncodingFormat => {
                                        render_encoding_format_view(self, window, cx)
                                    }
                                    ViewType::FormatterJson => {
                                        render_json_editor_view(self, window, cx)
                                    }
                                    ViewType::FormatterJsonSchema => {
                                        render_json_schema_view(self, window, cx)
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
                                    ViewType::GeneratorHash => render_hash_page(self, window, cx),
                                    ViewType::DatabaseDatafaker => {
                                        render_database_datafaker_view(self, window, cx)
                                    }
                                    ViewType::DatabaseDiff => {
                                        render_database_diff_view(self, window, cx)
                                    }
                                    ViewType::TextMarkdown => {
                                        render_markdown_editor_view(self, window, cx)
                                    }
                                    ViewType::TextTextTools => {
                                        render_text_tools_view(self, window, cx)
                                    }
                                    ViewType::TextDiff => render_text_diff_view(self, window, cx),
                                    ViewType::RandomString => {
                                        render_random_string_view(self, window, cx)
                                    }
                                    ViewType::RandomNumber => {
                                        render_random_number_view(self, window, cx)
                                    }
                                    ViewType::RandomOther => {
                                        render_random_other_view(self, window, cx)
                                    }
                                    ViewType::RandomData => {
                                        render_random_data_view(self, window, cx)
                                    }
                                    ViewType::NumberBitwise => {
                                        render_bitwise_calculator_view(self, window, cx)
                                    }
                                    ViewType::ColorConverter => {
                                        render_color_converter_view(self, window, cx)
                                    }
                                    ViewType::NetworkIp => {
                                        render_ip_converter_view(self, window, cx)
                                    }
                                    ViewType::ImageExcalidraw => {
                                        render_excalidraw_view(self, window, cx)
                                    }
                                    ViewType::ImageConvert => {
                                        render_image_convert_view(self, window, cx)
                                    }
                                    ViewType::PdfImagesToPdf => {
                                        render_pdf_images_to_pdf_view(self, cx)
                                    }
                                    ViewType::PdfMerge => render_pdf_merge_view(self, cx),
                                    ViewType::PdfEdit => render_pdf_edit_view(self, window, cx),
                                    ViewType::PdfPageNumber => {
                                        render_pdf_page_number_view(self, window, cx)
                                    }
                                    ViewType::PdfSplit => render_pdf_split_view(self, window, cx),
                                    ViewType::OtherQrCode => {
                                        render_qrcode_generator_view(self, window, cx)
                                    }
                                    ViewType::OtherQrCodeDecode => {
                                        render_qrcode_decoder_view(self, window, cx)
                                    }
                                    ViewType::OtherClipboard => {
                                        render_clipboard_manager_view(self, window, cx)
                                    }
                                    ViewType::RegexVisualizer => {
                                        render_regex_visualizer_view(self, window, cx)
                                    }
                                }),
                        ),
                ),
        )
    }
}

fn render_home_view(cx: &mut Context<App>) -> Div {
    let items: &[(&str, &str, IconName)] = &[
        ("系统监控", "系统资源实时监控", IconName::ChartPie),
        ("代码片段", "代码片段管理", IconName::File),
        ("待办事项", "待办事项清单", IconName::Check),
        ("对称加密", "AES / DES / RC4 等对称加密", IconName::EyeOff),
        (
            "国密算法",
            "SM2 / SM3 / SM4 国密算法工具",
            IconName::Asterisk,
        ),
        ("按位计算器", "按位运算计算器", IconName::SquareTerminal),
        ("颜色转换", "颜色格式转换", IconName::Palette),
        ("正则", "正则表达式可视化", IconName::Dash),
        ("文件格式转换", "文件类型识别与转换", IconName::File),
        ("时间戳", "时间戳与日期互转", IconName::Calendar),
        ("进制转换", "进制及字符串进制互转", IconName::ALargeSmall),
        ("Cron 表达式", "Cron 表达式解析与生成", IconName::Calendar),
        ("人民币大小写", "", IconName::ALargeSmall),
        (
            "Base 编码",
            "Base64 / Base32 / Base58 编码解码",
            IconName::CaseSensitive,
        ),
        ("URL", "URL 编码与解码", IconName::ExternalLink),
        ("JWT", "JWT 解析与验证", IconName::File),
        (
            "文本编码",
            "字符集 / 乱码 / 转义 / Unicode 编码转换",
            IconName::CaseSensitive,
        ),
        (
            "编码格式转换",
            "UTF-8 / GBK / ISO-8859-1 等字符集互转",
            IconName::Replace,
        ),
        ("二维码", "生成二维码", IconName::Frame),
        ("二维码解码", "从图片识别并解码二维码", IconName::Search),
        ("JSON Editor", "JSON 编辑与格式化", IconName::File),
        ("SQL", "SQL 语句格式化", IconName::SquareTerminal),
        ("XML", "XML 格式化与压缩", IconName::File),
        ("JSON Schema", "JSON Schema 校验", IconName::Check),
        ("UUID", "批量生成 UUID", IconName::ALargeSmall),
        ("Hash 计算", "文本与文件 Hash 计算", IconName::Asterisk),
        ("假数据生成", "可视化构造假数据", IconName::Folder),
        ("数据库差异", "数据库结构与数据对比", IconName::Folder),
        ("Markdown", "Markdown 编辑与预览", IconName::BookOpen),
        (
            "文本工具",
            "大小写 / 清理 / 统计等文本处理",
            IconName::BookOpen,
        ),
        (
            "文本 / JSON 差异",
            "文本 / JSON 差异对比",
            IconName::Replace,
        ),
        ("随机字符串", "随机字符串生成", IconName::CaseSensitive),
        ("随机数字", "随机数字生成", IconName::Asterisk),
        ("随机数据", "结构化随机数据", IconName::Folder),
        ("IP 地址转换", "IP 地址数值转换", IconName::Network),
        (
            "随机 IP / MAC / 时间",
            "随机 IP / MAC / 时间",
            IconName::Calendar,
        ),
        ("图片格式转换", "图片格式转换", IconName::Frame),
        ("Excalidraw", "手绘风格画板", IconName::Frame),
        ("图片转 PDF", "多张图片合成 PDF", IconName::File),
        ("PDF 合并", "合并多个 PDF", IconName::File),
        ("PDF 编辑", "编辑 PDF 页面", IconName::File),
        ("PDF 添加页码", "为 PDF 添加页码", IconName::File),
        ("PDF 拆分", "拆分 PDF 页面", IconName::File),
        ("剪贴板管理", "剪贴板历史管理", IconName::Settings2),
    ];

    let group_order = [
        "常用",
        "转换",
        "编码/解码",
        "加密",
        "格式化",
        "生成器",
        "数据库",
        "文本",
        "随机",
        "网络",
        "图像",
        "PDF",
        "其它",
    ];

    let card = |i: usize,
                title: &'static str,
                desc: &'static str,
                icon: IconName,
                cx: &mut Context<App>| {
        div()
            .id(ElementId::Name(SharedString::from(format!(
                "home-card-{i}"
            ))))
            .w(px(248.0))
            .min_w(px(248.0))
            .flex()
            .items_start()
            .gap_3()
            .border_1()
            .border_color(cx.theme().border)
            .rounded(px(14.0))
            .p(px(18.0))
            .bg(cx.theme().popover)
            .cursor_pointer()
            .hover(|style| {
                style
                    .border_color(cx.theme().primary.opacity(0.45))
                    .bg(cx.theme().secondary)
            })
            .on_click(cx.listener(move |this, _, _, cx| {
                if let Some(view) = view_for_title(title) {
                    this.set_view(view, cx);
                }
            }))
            .child(
                div()
                    .w(px(40.0))
                    .h(px(40.0))
                    .rounded(px(11.0))
                    .bg(cx.theme().primary.opacity(0.12))
                    .text_color(cx.theme().primary)
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(Icon::new(icon).size(px(20.0))),
            )
            .child(
                div()
                    .flex_1()
                    .flex_col()
                    .gap_0p5()
                    .child(div().text_sm().font_semibold().child(title))
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(desc),
                    ),
            )
    };

    let mut groups: Vec<Div> = Vec::new();
    for group in group_order {
        let mut cards: Vec<AnyElement> = Vec::new();
        for (i, &(title, desc, ref icon)) in items.iter().enumerate() {
            let g = TOOL_INDEX
                .iter()
                .find(|t| t.label == title)
                .map(|t| t.group)
                .unwrap_or("");
            let gname = if g.is_empty() { "常用" } else { g };
            if gname == group {
                cards.push(card(i, title, desc, icon.clone(), cx).into_any_element());
            }
        }
        if cards.is_empty() {
            continue;
        }
        groups.push(
            div()
                .flex_col()
                .gap_3()
                .child(
                    div()
                        .text_size(px(13.0))
                        .font_semibold()
                        .text_color(cx.theme().muted_foreground)
                        .child(group.to_string()),
                )
                .child(div().flex().flex_wrap().gap(px(14.0)).children(cards)),
        );
    }

    div()
        .w_full()
        .max_w(px(1280.0))
        .flex_col()
        .gap_6()
        .pt(px(8.0))
        .px(px(24.0))
        .pb(px(40.0))
        .children(groups)
        .child(
            div()
                .mt(px(24.0))
                .pt(px(20.0))
                .border_t_1()
                .border_color(cx.theme().border)
                .text_center()
                .text_size(px(12.5))
                .text_color(cx.theme().muted_foreground)
                .child("ToolBox · 开发工具集合"),
        )
}

fn view_for_title(title: &str) -> Option<ViewType> {
    TOOL_INDEX
        .iter()
        .find(|tool| tool.label == title)
        .map(|tool| tool.view)
}

fn render_text_encoding_page(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.text_encoding.is_none() {
        app.text_encoding = Some(cx.new(|cx| TextEncodingPage::new(window, cx)));
    }

    if let Some(ref page) = app.text_encoding {
        div().child(page.clone())
    } else {
        div().child("Loading...")
    }
}

fn render_hash_page(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.hash_page.is_none() {
        app.hash_page = Some(cx.new(|cx| HashPage::new(window, cx)));
    }

    if let Some(ref page) = app.hash_page {
        div().child(page.clone())
    } else {
        div().child("Loading...")
    }
}

fn render_system_monitor_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.system_monitor.is_none() {
        app.system_monitor = Some(cx.new(|cx| SystemMonitor::new(window, cx)));
    }

    if let Some(ref sys_monitor) = app.system_monitor {
        div().child(sys_monitor.clone())
    } else {
        div().child("Loading...")
    }
}

fn render_code_snippet_view(app: &mut App, cx: &mut Context<App>) -> Div {
    if app.code_snippet.is_none() {
        app.code_snippet = Some(cx.new(|_| CodeSnippet::new()));
    }

    if let Some(ref code_snip) = app.code_snippet {
        div().child(code_snip.clone())
    } else {
        div().child("Loading...")
    }
}

fn render_todo_view(app: &mut App, cx: &mut Context<App>) -> Div {
    if app.todo_list.is_none() {
        let todo_list = cx.new(|_| TodoList::new());
        app.todo_list = Some(todo_list.clone());
    }

    if let Some(ref todo) = app.todo_list {
        div().child(todo.clone())
    } else {
        div().child("Loading...")
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
        div().child(transform.clone())
    } else {
        div().child("Loading...")
    }
}

fn render_formatter_sql_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.sql_formatter.is_none() {
        app.sql_formatter = Some(cx.new(|cx| SqlFormatter::new(window, cx)));
    }

    if let Some(ref sql_fmt) = app.sql_formatter {
        div().child(sql_fmt.clone())
    } else {
        div().child("Loading...")
    }
}

fn render_formatter_xml_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.xml_formatter.is_none() {
        app.xml_formatter = Some(cx.new(|cx| XmlFormatter::new(window, cx)));
    }

    if let Some(ref xml_fmt) = app.xml_formatter {
        div().child(xml_fmt.clone())
    } else {
        div().child("Loading...")
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
        div().child(db_diff.clone())
    } else {
        div().child("Loading...")
    }
}

fn render_uuid_generator_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.uuid_generator.is_none() {
        app.uuid_generator = Some(cx.new(|cx| UuidGenerator::new(window, cx)));
    }

    if let Some(ref uuid_gen) = app.uuid_generator {
        div().child(uuid_gen.clone())
    } else {
        div().child("Loading...")
    }
}

fn render_url_encoder_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.url_encoder.is_none() {
        app.url_encoder = Some(cx.new(|cx| UrlEncoder::new(window, cx)));
    }

    if let Some(ref url_enc) = app.url_encoder {
        div().child(url_enc.clone())
    } else {
        div().child("Loading...")
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
        div().child(ts_conv.clone())
    } else {
        div().child("Loading...")
    }
}

fn render_jwt_decoder_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.jwt_decoder.is_none() {
        app.jwt_decoder = Some(cx.new(|cx| JwtDecoder::new(window, cx)));
    }

    if let Some(ref jwt) = app.jwt_decoder {
        div().child(jwt.clone())
    } else {
        div().child("Loading...")
    }
}

fn render_ip_converter_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.ip_converter.is_none() {
        app.ip_converter = Some(cx.new(|cx| IpConverter::new(window, cx)));
    }

    if let Some(ref ip) = app.ip_converter {
        div().child(ip.clone())
    } else {
        div().child("Loading...")
    }
}

fn render_encoding_format_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.encoding_format.is_none() {
        app.encoding_format = Some(cx.new(|cx| EncodingFormatConvert::new(window, cx)));
    }
    if let Some(ref view) = app.encoding_format {
        div().child(view.clone())
    } else {
        div().child("Loading...")
    }
}

fn render_sm_crypto_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.sm_crypto.is_none() {
        app.sm_crypto = Some(cx.new(|cx| SmCrypto::new(window, cx)));
    }
    if let Some(ref view) = app.sm_crypto {
        div().child(view.clone())
    } else {
        div().child("Loading...")
    }
}

fn render_json_schema_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.json_schema.is_none() {
        app.json_schema = Some(cx.new(|cx| JsonSchemaValidator::new(window, cx)));
    }
    if let Some(ref view) = app.json_schema {
        div().child(view.clone())
    } else {
        div().child("Loading...")
    }
}

fn render_qrcode_decoder_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.qrcode_decoder.is_none() {
        app.qrcode_decoder = Some(cx.new(|cx| QrCodeDecoder::new(window, cx)));
    }
    if let Some(ref view) = app.qrcode_decoder {
        div().child(view.clone())
    } else {
        div().child("Loading...")
    }
}

fn render_qrcode_generator_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.qrcode_generator.is_none() {
        app.qrcode_generator = Some(cx.new(|cx| QrCodeGenerator::new(window, cx)));
    }

    if let Some(ref qrcode) = app.qrcode_generator {
        div().child(qrcode.clone())
    } else {
        div().child("Loading...")
    }
}

fn render_clipboard_manager_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.clipboard_manager.is_none() {
        app.clipboard_manager = Some(cx.new(|cx| ClipboardManager::new(window, cx)));
    }

    if let Some(ref clipboard) = app.clipboard_manager {
        div().child(clipboard.clone())
    } else {
        div().child("Loading...")
    }
}

fn render_regex_visualizer_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.regex_visualizer.is_none() {
        app.regex_visualizer = Some(cx.new(|cx| RegexVisualizer::new(window, cx)));
    }

    if let Some(ref regex) = app.regex_visualizer {
        div()
            .h_full()
            .pt(px(16.0))
            .px(px(20.0))
            .pb(px(24.0))
            .child(regex.clone())
    } else {
        div().child("Loading...")
    }
}

fn render_excalidraw_view(app: &mut App, _window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.excalidraw.is_none() {
        app.excalidraw = Some(cx.new(|_| {
            let mut editor = ExcalidrawView::new();
            excalidraw_store::install_scene_store(&mut editor);
            editor
        }));
    }

    if let Some(ref excalidraw) = app.excalidraw {
        div()
            .size_full()
            .pt(px(4.0))
            .px(px(12.0))
            .pb(px(12.0))
            .child(excalidraw.clone())
    } else {
        div().size_full().p_6().child("Loading...")
    }
}

fn render_rmb_case_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.rmb_case.is_none() {
        app.rmb_case = Some(cx.new(|cx| RmbCase::new(window, cx)));
    }

    if let Some(ref view) = app.rmb_case {
        div().child(view.clone())
    } else {
        div().child("Loading...")
    }
}

fn render_image_convert_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.image_convert.is_none() {
        app.image_convert = Some(cx.new(|cx| ImageConvert::new(window, cx)));
    }

    if let Some(ref view) = app.image_convert {
        div().child(view.clone())
    } else {
        div().child("Loading...")
    }
}

fn render_pdf_images_to_pdf_view(app: &mut App, cx: &mut Context<App>) -> Div {
    if app.pdf_images_to_pdf.is_none() {
        app.pdf_images_to_pdf = Some(cx.new(|_| PdfFromFiles::new(PdfFromFilesKind::ImagesToPdf)));
    }

    if let Some(ref view) = app.pdf_images_to_pdf {
        div().child(view.clone())
    } else {
        div().child("Loading...")
    }
}

fn render_pdf_merge_view(app: &mut App, cx: &mut Context<App>) -> Div {
    if app.pdf_merge.is_none() {
        app.pdf_merge = Some(cx.new(|_| PdfFromFiles::new(PdfFromFilesKind::Merge)));
    }

    if let Some(ref view) = app.pdf_merge {
        div().child(view.clone())
    } else {
        div().child("Loading...")
    }
}

fn render_pdf_edit_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.pdf_edit.is_none() {
        app.pdf_edit = Some(cx.new(|cx| PdfEditTool::new(window, cx)));
    }

    if let Some(ref view) = app.pdf_edit {
        div().child(view.clone())
    } else {
        div().child("Loading...")
    }
}

fn render_pdf_page_number_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.pdf_page_number.is_none() {
        app.pdf_page_number = Some(cx.new(|cx| PdfPageNumberTool::new(window, cx)));
    }

    if let Some(ref view) = app.pdf_page_number {
        div().child(view.clone())
    } else {
        div().child("Loading...")
    }
}

fn render_pdf_split_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.pdf_split.is_none() {
        app.pdf_split = Some(cx.new(|cx| PdfSplitTool::new(window, cx)));
    }

    if let Some(ref view) = app.pdf_split {
        div().child(view.clone())
    } else {
        div().child("Loading...")
    }
}

fn render_base_converter_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.base_converter.is_none() {
        app.base_converter = Some(cx.new(|cx| BaseConverter::new(window, cx)));
    }

    if let Some(ref base_conv) = app.base_converter {
        div().child(base_conv.clone())
    } else {
        div().child("Loading...")
    }
}

fn render_json_editor_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.json_editor.is_none() {
        app.json_editor = Some(cx.new(|cx| JsonEditor::new(window, cx)));
    }

    if let Some(ref json_edit) = app.json_editor {
        div().child(json_edit.clone())
    } else {
        div().child("Loading...")
    }
}

fn render_markdown_editor_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.markdown_editor.is_none() {
        app.markdown_editor = Some(cx.new(|cx| MarkdownEditor::new(window, cx)));
    }

    if let Some(ref md_edit) = app.markdown_editor {
        div().child(md_edit.clone())
    } else {
        div().child("Loading...")
    }
}

fn render_cron_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.cron_converter.is_none() {
        app.cron_converter = Some(cx.new(|cx| CronConverter::new(window, cx)));
    }

    if let Some(ref view) = app.cron_converter {
        div().child(view.clone())
    } else {
        div().child("Loading...")
    }
}

fn render_base_encoding_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.base_encoding.is_none() {
        app.base_encoding = Some(cx.new(|cx| BaseEncodingConverter::new(window, cx)));
    }

    if let Some(ref view) = app.base_encoding {
        div().child(view.clone())
    } else {
        div().child("Loading...")
    }
}

fn render_symmetric_encrypt_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.symmetric_encrypt.is_none() {
        app.symmetric_encrypt = Some(cx.new(|cx| SymmetricEncryptor::new(window, cx)));
    }

    if let Some(ref view) = app.symmetric_encrypt {
        div().child(view.clone())
    } else {
        div().child("Loading...")
    }
}

fn render_text_tools_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.text_tools.is_none() {
        app.text_tools = Some(cx.new(|cx| TextTools::new(window, cx)));
    }

    if let Some(ref view) = app.text_tools {
        div().child(view.clone())
    } else {
        div().child("Loading...")
    }
}

fn render_text_diff_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.text_diff.is_none() {
        app.text_diff = Some(cx.new(|cx| TextDiffTool::new(window, cx)));
    }

    if let Some(ref view) = app.text_diff {
        div().child(view.clone())
    } else {
        div().child("Loading...")
    }
}

fn render_random_string_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.random_string.is_none() {
        app.random_string = Some(cx.new(|cx| RandomStringGenerator::new(window, cx)));
    }

    if let Some(ref view) = app.random_string {
        div().child(view.clone())
    } else {
        div().child("Loading...")
    }
}

fn render_random_number_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.random_number.is_none() {
        app.random_number = Some(cx.new(|cx| RandomNumberGenerator::new(window, cx)));
    }

    if let Some(ref view) = app.random_number {
        div().child(view.clone())
    } else {
        div().child("Loading...")
    }
}

fn render_random_other_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.random_other.is_none() {
        app.random_other = Some(cx.new(|cx| RandomOtherGenerator::new(window, cx)));
    }

    if let Some(ref view) = app.random_other {
        div().child(view.clone())
    } else {
        div().child("Loading...")
    }
}

fn render_random_data_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.random_data.is_none() {
        app.random_data = Some(cx.new(|cx| RandomDataGenerator::new(window, cx)));
    }

    if let Some(ref view) = app.random_data {
        div().child(view.clone())
    } else {
        div().child("Loading...")
    }
}

fn render_bitwise_calculator_view(
    app: &mut App,
    window: &mut Window,
    cx: &mut Context<App>,
) -> Div {
    if app.bitwise_calculator.is_none() {
        app.bitwise_calculator = Some(cx.new(|cx| BitwiseCalculator::new(window, cx)));
    }

    if let Some(ref view) = app.bitwise_calculator {
        div().child(view.clone())
    } else {
        div().child("Loading...")
    }
}

fn render_color_converter_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.color_converter.is_none() {
        app.color_converter = Some(cx.new(|cx| ColorConverter::new(window, cx)));
    }

    if let Some(ref view) = app.color_converter {
        div().child(view.clone())
    } else {
        div().child("Loading...")
    }
}

fn main() {
    sqlx::any::install_default_drivers();

    let _guard = TOKIO_RUNTIME.enter();

    let app = gpui_kit::platform::application().with_assets(Assets);

    app.run(move |cx| {
        gpui_kit::init(cx);
        let bounds = Bounds::centered(None, size(px(1200.0), px(800.0)), cx);

        cx.spawn(async move |cx| {
            if let Ok(Some(saved)) = config_store::get_setting("theme").await {
                let mode = if saved == "dark" {
                    ThemeMode::Dark
                } else {
                    ThemeMode::Light
                };
                cx.update(|cx| Theme::change(mode, None, cx));
            }

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
