use std::sync::LazyLock;

use gpui_kit::*;
use gpui_kit::component::{button::*, scroll::ScrollableElement, sidebar::*, *};
use gpui_kit::assets::Assets;
mod config_store;
mod design;
mod views;
use views::*;

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
    EncodeDecodeBase64,
    EncodeDecodeUrl,
    EncodeDecodeJwt,
    EncodeDecodeCharset,
    EncodeDecodeMessyCode,
    EncodeDecodeBase64Image,
    EncodeDecodeEscape,
    EncodeDecodeBaseEncoding,
    EncodeDecodeUtf8Unicode,
    EncodeDecodeSymmetric,
    FormatterJson,
    FormatterSql,
    FormatterXml,
    GeneratorUuid,
    GeneratorHash,
    GeneratorChecksum,
    DatabaseDatafaker,
    DatabaseDiff,
    TextMarkdown,
    TextCase,
    TextTextTools,
    TextStrHex,
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
    cron_converter: Option<Entity<CronConverter>>,
    base64_image: Option<Entity<Base64ImageConverter>>,
    escape_tools: Option<Entity<EscapeTools>>,
    base_encoding: Option<Entity<BaseEncodingConverter>>,
    utf8_unicode: Option<Entity<Utf8UnicodeConverter>>,
    symmetric_encrypt: Option<Entity<SymmetricEncryptor>>,
    case_converter: Option<Entity<CaseConverter>>,
    text_tools: Option<Entity<TextTools>>,
    str_hex: Option<Entity<StrHexConverter>>,
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
            cron_converter: None,
            base64_image: None,
            escape_tools: None,
            base_encoding: None,
            utf8_unicode: None,
            symmetric_encrypt: None,
            case_converter: None,
            text_tools: None,
            str_hex: None,
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
                        Sidebar::new("main-sidebar")
                            .side(Side::Left)
                            .collapsed(self.sidebar_collapsed)
                            .header(
                                SidebarToggleButton::new()
                                    .side(Side::Left)
                                    .collapsed(self.sidebar_collapsed)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.toggle_sidebar(cx);
                                    })),
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
                                    .icon(Icon::new(IconName::File))
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
                                                this.set_view(
                                                    ViewType::TransformBaseConversion,
                                                    cx,
                                                );
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
                                        SidebarMenuItem::new("Base64 图片")
                                            .icon(Icon::new(IconName::Frame))
                                            .active(current_view == ViewType::EncodeDecodeBase64Image)
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.set_view(
                                                    ViewType::EncodeDecodeBase64Image,
                                                    cx,
                                                );
                                            })),
                                        SidebarMenuItem::new("转义工具")
                                            .icon(Icon::new(IconName::Dash))
                                            .active(current_view == ViewType::EncodeDecodeEscape)
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.set_view(ViewType::EncodeDecodeEscape, cx);
                                            })),
                                        SidebarMenuItem::new("Base32 / Base58")
                                            .icon(Icon::new(IconName::CaseSensitive))
                                            .active(current_view == ViewType::EncodeDecodeBaseEncoding)
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.set_view(
                                                    ViewType::EncodeDecodeBaseEncoding,
                                                    cx,
                                                );
                                            })),
                                        SidebarMenuItem::new("UTF8 / Unicode")
                                            .icon(Icon::new(IconName::CaseSensitive))
                                            .active(current_view == ViewType::EncodeDecodeUtf8Unicode)
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.set_view(
                                                    ViewType::EncodeDecodeUtf8Unicode,
                                                    cx,
                                                );
                                            })),
                                        SidebarMenuItem::new("对称加密")
                                            .icon(Icon::new(IconName::SquareTerminal))
                                            .active(current_view == ViewType::EncodeDecodeSymmetric)
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.set_view(
                                                    ViewType::EncodeDecodeSymmetric,
                                                    cx,
                                                );
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
                                        SidebarMenuItem::new("大小写转换")
                                            .icon(Icon::new(IconName::ALargeSmall))
                                            .active(current_view == ViewType::TextCase)
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.set_view(ViewType::TextCase, cx);
                                            })),
                                        SidebarMenuItem::new("文本工具")
                                            .icon(Icon::new(IconName::BookOpen))
                                            .active(current_view == ViewType::TextTextTools)
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.set_view(ViewType::TextTextTools, cx);
                                            })),
                                        SidebarMenuItem::new("字符串进制转换")
                                            .icon(Icon::new(IconName::ALargeSmall))
                                            .active(current_view == ViewType::TextStrHex)
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.set_view(ViewType::TextStrHex, cx);
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
                                        SidebarMenuItem::new("随机 IP / MAC / 时间")
                                            .icon(Icon::new(IconName::Globe))
                                            .active(current_view == ViewType::RandomOther)
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.set_view(ViewType::RandomOther, cx);
                                            })),
                                        SidebarMenuItem::new("随机数据")
                                            .icon(Icon::new(IconName::Folder))
                                            .active(current_view == ViewType::RandomData)
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.set_view(ViewType::RandomData, cx);
                                            })),
                                    ]),
                                SidebarMenuItem::new("数字")
                                    .icon(Icon::new(IconName::Globe))
                                    .click_to_open(true)
                                    .children([
                                        SidebarMenuItem::new("IP 地址转换")
                                            .icon(Icon::new(IconName::Globe))
                                            .active(current_view == ViewType::NetworkIp)
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.set_view(ViewType::NetworkIp, cx);
                                            })),
                                        SidebarMenuItem::new("按位计算器")
                                            .icon(Icon::new(IconName::SquareTerminal))
                                            .active(current_view == ViewType::NumberBitwise)
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.set_view(ViewType::NumberBitwise, cx);
                                            })),
                                    ]),
                                SidebarMenuItem::new("颜色转换")
                                    .icon(Icon::new(IconName::Frame))
                                    .click_to_open(true)
                                    .children([SidebarMenuItem::new("颜色转换")
                                        .icon(Icon::new(IconName::Frame))
                                        .active(current_view == ViewType::ColorConverter)
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.set_view(ViewType::ColorConverter, cx);
                                        }))]),
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
                                SidebarMenuItem::new("正则")
                                    .icon(Icon::new(IconName::Dash))
                                    .click_to_open(true)
                                    .children([SidebarMenuItem::new("正则可视化")
                                        .icon(Icon::new(IconName::Dash))
                                        .active(current_view == ViewType::RegexVisualizer)
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.set_view(ViewType::RegexVisualizer, cx);
                                        }))]),
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
                                ViewType::TransformRmb => render_rmb_case_view(self, window, cx),
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
                                ViewType::EncodeDecodeBase64Image => {
                                    render_base64_image_view(self, window, cx)
                                }
                                ViewType::EncodeDecodeEscape => {
                                    render_escape_tools_view(self, window, cx)
                                }
                                ViewType::EncodeDecodeBaseEncoding => {
                                    render_base_encoding_view(self, window, cx)
                                }
                                ViewType::EncodeDecodeUtf8Unicode => {
                                    render_utf8_unicode_view(self, window, cx)
                                }
                                ViewType::EncodeDecodeSymmetric => {
                                    render_symmetric_encrypt_view(self, window, cx)
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
                                ViewType::TextCase => render_case_converter_view(self, window, cx),
                                ViewType::TextTextTools => {
                                    render_text_tools_view(self, window, cx)
                                }
                                ViewType::TextStrHex => render_str_hex_view(self, window, cx),
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
                                ViewType::RandomData => render_random_data_view(self, window, cx),
                                ViewType::NumberBitwise => {
                                    render_bitwise_calculator_view(self, window, cx)
                                }
                                ViewType::ColorConverter => {
                                    render_color_converter_view(self, window, cx)
                                }
                                ViewType::NetworkIp => render_ip_converter_view(self, window, cx),
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
    let items: &[(&str, &str, IconName)] = &[
        ("系统监控", "实时监控 CPU、内存与磁盘", IconName::ChartPie),
        ("代码片段", "管理常用代码片段", IconName::File),
        ("待办事项", "简单的任务清单", IconName::Check),
        ("文件格式转换", "转换各类文件格式", IconName::Replace),
        ("时间戳", "时间戳与日期互转", IconName::Calendar),
        ("进制转换", "十进制 / 十六进制 / 二进制", IconName::ALargeSmall),
        ("Cron 表达式", "解析与生成 Cron", IconName::Calendar),
        ("人民币大小写", "金额与中文大写互转", IconName::ALargeSmall),
        ("Base64", "Base64 编码与解码", IconName::CaseSensitive),
        ("Base64 图片", "图片与 Base64 互转", IconName::Frame),
        ("URL", "URL 编码与解码", IconName::ExternalLink),
        ("JWT", "解析与校验 JWT", IconName::File),
        ("字符编码", "字符集编码转换", IconName::CaseSensitive),
        ("乱码恢复", "修复乱码文本", IconName::CaseSensitive),
        ("转义工具", "HTML / 字符串转义", IconName::Dash),
        ("Base32 / Base58", "Base32 与 Base58 编码", IconName::CaseSensitive),
        ("UTF8 / Unicode", "UTF8 与 Unicode 互转", IconName::CaseSensitive),
        ("对称加密", "AES / DES / RC4 / Rabbit", IconName::SquareTerminal),
        ("JSON Editor", "JSON 格式化与编辑", IconName::File),
        ("SQL", "SQL 格式化", IconName::SquareTerminal),
        ("XML", "XML 格式化", IconName::File),
        ("UUID", "生成多种 UUID", IconName::ALargeSmall),
        ("文本Hash", "计算文本散列值", IconName::Asterisk),
        ("文件校验", "校验文件散列值", IconName::File),
        ("假数据生成", "生成模拟业务数据", IconName::Folder),
        ("数据库差异", "对比两个数据库结构", IconName::Folder),
        ("Markdown", "Markdown 编辑与预览", IconName::BookOpen),
        ("大小写转换", "文本大小写快速转换", IconName::ALargeSmall),
        ("文本工具", "常用文本处理工具", IconName::BookOpen),
        ("字符串进制转换", "字符串与十六进制互转", IconName::ALargeSmall),
        ("文本 / JSON 差异", "对比文本或 JSON", IconName::Replace),
        ("随机字符串", "生成随机字符串", IconName::CaseSensitive),
        ("随机数字", "生成随机数字", IconName::Asterisk),
        ("随机 IP / MAC / 时间", "生成随机 IP / MAC / 时间", IconName::Globe),
        ("随机数据", "生成随机数据记录", IconName::Folder),
        ("IP 地址转换", "IP 与整数互转", IconName::Globe),
        ("按位计算器", "二进制位运算", IconName::SquareTerminal),
        ("颜色转换", "HEX / RGB / HSL / CMYK", IconName::Frame),
        ("Excalidraw", "白板绘图", IconName::Frame),
        ("图片格式转换", "常见图片格式互转", IconName::Frame),
        ("图片转 PDF", "图片合成为 PDF", IconName::File),
        ("PDF 合并", "多个 PDF 合并", IconName::File),
        ("PDF 编辑", "删除 / 旋转 / 排序页面", IconName::File),
        ("PDF 添加页码", "添加页码或自定义文本", IconName::File),
        ("PDF 拆分", "按页码范围拆分", IconName::File),
        ("二维码", "生成二维码", IconName::Frame),
        ("剪贴板管理", "剪贴板历史记录", IconName::Settings2),
        ("正则可视化", "正则表达式调试", IconName::Dash),
        ("设置", "应用偏好设置", IconName::Settings),
    ];
    div()
        .p_6()
        .size_full()
        .flex_col()
        .gap_4()
        .child(design::page_header("工具箱", "选择工具开始使用", cx))
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap_3()
                .children(items.iter().enumerate().map(|(i, &(title, desc, ref icon))| {
                    div()
                        .id(ElementId::Name(SharedString::from(format!("home-card-{i}"))))
                        .w(px(248.0))
                        .min_w(px(248.0))
                        .flex()
                        .items_start()
                        .gap_3()
                        .border_1()
                        .border_color(cx.theme().border)
                        .rounded(px(12.0))
                        .p_4()
                        .bg(cx.theme().popover)
                        .cursor_pointer()
                        .hover(|style| {
                            style.border_color(cx.theme().primary.opacity(0.45)).bg(cx.theme().secondary)
                        })
                        .on_click(cx.listener(move |this, _, _, cx| {
                            if let Some(view) = view_for_title(title) {
                                this.set_view(view, cx);
                            }
                        }))
                        .child(
                            div()
                                .w(px(36.0))
                                .h(px(36.0))
                                .rounded(px(10.0))
                                .bg(cx.theme().primary.opacity(0.12))
                                .text_color(cx.theme().primary)
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(Icon::new(icon.clone()).size(px(18.0))),
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
                })),
        )
        .child(
            div()
                .mt_2()
                .pt_4()
                .border_t_1()
                .border_color(cx.theme().border)
                .text_center()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
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
        "Cron 表达式" => Some(ViewType::TransformCron),
        "人民币大小写" => Some(ViewType::TransformRmb),
        "Base64" => Some(ViewType::EncodeDecodeBase64),
        "Base64 图片" => Some(ViewType::EncodeDecodeBase64Image),
        "URL" => Some(ViewType::EncodeDecodeUrl),
        "JWT" => Some(ViewType::EncodeDecodeJwt),
        "字符编码" => Some(ViewType::EncodeDecodeCharset),
        "乱码恢复" => Some(ViewType::EncodeDecodeMessyCode),
        "转义工具" => Some(ViewType::EncodeDecodeEscape),
        "Base32 / Base58" => Some(ViewType::EncodeDecodeBaseEncoding),
        "UTF8 / Unicode" => Some(ViewType::EncodeDecodeUtf8Unicode),
        "对称加密" => Some(ViewType::EncodeDecodeSymmetric),
        "JSON Editor" => Some(ViewType::FormatterJson),
        "SQL" => Some(ViewType::FormatterSql),
        "XML" => Some(ViewType::FormatterXml),
        "UUID" => Some(ViewType::GeneratorUuid),
        "文本Hash" => Some(ViewType::GeneratorHash),
        "文件校验" => Some(ViewType::GeneratorChecksum),
        "假数据生成" => Some(ViewType::DatabaseDatafaker),
        "数据库差异" => Some(ViewType::DatabaseDiff),
        "Markdown" => Some(ViewType::TextMarkdown),
        "大小写转换" => Some(ViewType::TextCase),
        "文本工具" => Some(ViewType::TextTextTools),
        "字符串进制转换" => Some(ViewType::TextStrHex),
        "文本 / JSON 差异" => Some(ViewType::TextDiff),
        "随机字符串" => Some(ViewType::RandomString),
        "随机数字" => Some(ViewType::RandomNumber),
        "随机 IP / MAC / 时间" => Some(ViewType::RandomOther),
        "随机数据" => Some(ViewType::RandomData),
        "IP 地址转换" => Some(ViewType::NetworkIp),
        "按位计算器" => Some(ViewType::NumberBitwise),
        "颜色转换" => Some(ViewType::ColorConverter),
        "Excalidraw" => Some(ViewType::ImageExcalidraw),
        "图片格式转换" => Some(ViewType::ImageConvert),
        "图片转 PDF" => Some(ViewType::PdfImagesToPdf),
        "PDF 合并" => Some(ViewType::PdfMerge),
        "PDF 编辑" => Some(ViewType::PdfEdit),
        "PDF 添加页码" => Some(ViewType::PdfPageNumber),
        "PDF 拆分" => Some(ViewType::PdfSplit),
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
        // 对齐 Vue：tb-page padding(16/20/24) + 卡片撑满剩余高度
        div()
            .h_full()
            .pt(px(16.0))
            .px(px(20.0))
            .pb(px(24.0))
            .child(regex.clone())
    } else {
        div().p_6().child("Loading...")
    }
}

fn render_excalidraw_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.excalidraw.is_none() {
        app.excalidraw = Some(cx.new(|cx| ExcalidrawView::new(window, cx)));
    }

    if let Some(ref excalidraw) = app.excalidraw {
        // 对齐 Vue excalidraw-page：padding 4px 12px 12px
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

fn render_rmb_case_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.rmb_case.is_none() {
        app.rmb_case = Some(cx.new(|cx| RmbCase::new(window, cx)));
    }

    if let Some(ref view) = app.rmb_case {
        div().p_6().child(view.clone())
    } else {
        div().p_6().child("Loading...")
    }
}

fn render_image_convert_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.image_convert.is_none() {
        app.image_convert = Some(cx.new(|cx| ImageConvert::new(window, cx)));
    }

    if let Some(ref view) = app.image_convert {
        div().p_6().child(view.clone())
    } else {
        div().p_6().child("Loading...")
    }
}

fn render_pdf_images_to_pdf_view(app: &mut App, cx: &mut Context<App>) -> Div {
    if app.pdf_images_to_pdf.is_none() {
        app.pdf_images_to_pdf = Some(cx.new(|_| {
            PdfFromFiles::new(PdfFromFilesKind::ImagesToPdf)
        }));
    }

    if let Some(ref view) = app.pdf_images_to_pdf {
        div().p_6().child(view.clone())
    } else {
        div().p_6().child("Loading...")
    }
}

fn render_pdf_merge_view(app: &mut App, cx: &mut Context<App>) -> Div {
    if app.pdf_merge.is_none() {
        app.pdf_merge = Some(cx.new(|_| PdfFromFiles::new(PdfFromFilesKind::Merge)));
    }

    if let Some(ref view) = app.pdf_merge {
        div().p_6().child(view.clone())
    } else {
        div().p_6().child("Loading...")
    }
}

fn render_pdf_edit_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.pdf_edit.is_none() {
        app.pdf_edit = Some(cx.new(|cx| PdfEditTool::new(window, cx)));
    }

    if let Some(ref view) = app.pdf_edit {
        div().p_6().child(view.clone())
    } else {
        div().p_6().child("Loading...")
    }
}

fn render_pdf_page_number_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.pdf_page_number.is_none() {
        app.pdf_page_number = Some(cx.new(|cx| PdfPageNumberTool::new(window, cx)));
    }

    if let Some(ref view) = app.pdf_page_number {
        div().p_6().child(view.clone())
    } else {
        div().p_6().child("Loading...")
    }
}

fn render_pdf_split_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.pdf_split.is_none() {
        app.pdf_split = Some(cx.new(|cx| PdfSplitTool::new(window, cx)));
    }

    if let Some(ref view) = app.pdf_split {
        div().p_6().child(view.clone())
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

fn render_cron_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.cron_converter.is_none() {
        app.cron_converter = Some(cx.new(|cx| CronConverter::new(window, cx)));
    }

    if let Some(ref view) = app.cron_converter {
        div().p_6().child(view.clone())
    } else {
        div().p_6().child("Loading...")
    }
}

fn render_base64_image_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.base64_image.is_none() {
        app.base64_image = Some(cx.new(|cx| Base64ImageConverter::new(window, cx)));
    }

    if let Some(ref view) = app.base64_image {
        div().p_6().child(view.clone())
    } else {
        div().p_6().child("Loading...")
    }
}

fn render_escape_tools_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.escape_tools.is_none() {
        app.escape_tools = Some(cx.new(|cx| EscapeTools::new(window, cx)));
    }

    if let Some(ref view) = app.escape_tools {
        div().p_6().child(view.clone())
    } else {
        div().p_6().child("Loading...")
    }
}

fn render_base_encoding_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.base_encoding.is_none() {
        app.base_encoding = Some(cx.new(|cx| BaseEncodingConverter::new(window, cx)));
    }

    if let Some(ref view) = app.base_encoding {
        div().p_6().child(view.clone())
    } else {
        div().p_6().child("Loading...")
    }
}

fn render_utf8_unicode_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.utf8_unicode.is_none() {
        app.utf8_unicode = Some(cx.new(|cx| Utf8UnicodeConverter::new(window, cx)));
    }

    if let Some(ref view) = app.utf8_unicode {
        div().p_6().child(view.clone())
    } else {
        div().p_6().child("Loading...")
    }
}

fn render_symmetric_encrypt_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.symmetric_encrypt.is_none() {
        app.symmetric_encrypt = Some(cx.new(|cx| SymmetricEncryptor::new(window, cx)));
    }

    if let Some(ref view) = app.symmetric_encrypt {
        div().p_6().child(view.clone())
    } else {
        div().p_6().child("Loading...")
    }
}

fn render_case_converter_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.case_converter.is_none() {
        app.case_converter = Some(cx.new(|cx| CaseConverter::new(window, cx)));
    }

    if let Some(ref view) = app.case_converter {
        div().p_6().child(view.clone())
    } else {
        div().p_6().child("Loading...")
    }
}

fn render_text_tools_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.text_tools.is_none() {
        app.text_tools = Some(cx.new(|cx| TextTools::new(window, cx)));
    }

    if let Some(ref view) = app.text_tools {
        div().p_6().child(view.clone())
    } else {
        div().p_6().child("Loading...")
    }
}

fn render_str_hex_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.str_hex.is_none() {
        app.str_hex = Some(cx.new(|cx| StrHexConverter::new(window, cx)));
    }

    if let Some(ref view) = app.str_hex {
        div().p_6().child(view.clone())
    } else {
        div().p_6().child("Loading...")
    }
}

fn render_text_diff_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.text_diff.is_none() {
        app.text_diff = Some(cx.new(|cx| TextDiffTool::new(window, cx)));
    }

    if let Some(ref view) = app.text_diff {
        div().p_6().child(view.clone())
    } else {
        div().p_6().child("Loading...")
    }
}

fn render_random_string_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.random_string.is_none() {
        app.random_string = Some(cx.new(|cx| RandomStringGenerator::new(window, cx)));
    }

    if let Some(ref view) = app.random_string {
        div().p_6().child(view.clone())
    } else {
        div().p_6().child("Loading...")
    }
}

fn render_random_number_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.random_number.is_none() {
        app.random_number = Some(cx.new(|cx| RandomNumberGenerator::new(window, cx)));
    }

    if let Some(ref view) = app.random_number {
        div().p_6().child(view.clone())
    } else {
        div().p_6().child("Loading...")
    }
}

fn render_random_other_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.random_other.is_none() {
        app.random_other = Some(cx.new(|cx| RandomOtherGenerator::new(window, cx)));
    }

    if let Some(ref view) = app.random_other {
        div().p_6().child(view.clone())
    } else {
        div().p_6().child("Loading...")
    }
}

fn render_random_data_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.random_data.is_none() {
        app.random_data = Some(cx.new(|cx| RandomDataGenerator::new(window, cx)));
    }

    if let Some(ref view) = app.random_data {
        div().p_6().child(view.clone())
    } else {
        div().p_6().child("Loading...")
    }
}

fn render_bitwise_calculator_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.bitwise_calculator.is_none() {
        app.bitwise_calculator = Some(cx.new(|cx| BitwiseCalculator::new(window, cx)));
    }

    if let Some(ref view) = app.bitwise_calculator {
        div().p_6().child(view.clone())
    } else {
        div().p_6().child("Loading...")
    }
}

fn render_color_converter_view(app: &mut App, window: &mut Window, cx: &mut Context<App>) -> Div {
    if app.color_converter.is_none() {
        app.color_converter = Some(cx.new(|cx| ColorConverter::new(window, cx)));
    }

    if let Some(ref view) = app.color_converter {
        div().p_6().child(view.clone())
    } else {
        div().p_6().child("Loading...")
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
