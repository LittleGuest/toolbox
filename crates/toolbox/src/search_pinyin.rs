use gpui_kit::component::IconName;

use crate::ViewType;

pub struct ToolEntry {
    pub label: &'static str,
    pub group: &'static str,

    pub key: &'static str,
    pub view: ViewType,
    pub icon: IconName,
}

pub const TOOL_INDEX: &[ToolEntry] = &[
    ToolEntry {
        label: "系统监控",
        group: "",
        key: "/systemMonitor",
        view: ViewType::SystemMonitor,
        icon: IconName::ChartPie,
    },
    ToolEntry {
        label: "代码片段",
        group: "",
        key: "/codeSnippet",
        view: ViewType::CodeSnippet,
        icon: IconName::FileText,
    },
    ToolEntry {
        label: "待办事项",
        group: "",
        key: "/todo",
        view: ViewType::Todo,
        icon: IconName::Check,
    },
    ToolEntry {
        label: "文件格式转换",
        group: "转换",
        key: "/transform/filetype",
        view: ViewType::TransformFiletype,
        icon: IconName::File,
    },
    ToolEntry {
        label: "时间戳",
        group: "转换",
        key: "/transform/time",
        view: ViewType::TransformTime,
        icon: IconName::Calendar,
    },
    ToolEntry {
        label: "进制转换",
        group: "转换",
        key: "/transform/baseconversion",
        view: ViewType::TransformBaseConversion,
        icon: IconName::ALargeSmall,
    },
    ToolEntry {
        label: "Cron 表达式",
        group: "转换",
        key: "/transform/cron",
        view: ViewType::TransformCron,
        icon: IconName::Calendar,
    },
    ToolEntry {
        label: "人民币大小写",
        group: "转换",
        key: "/transform/rmbcase",
        view: ViewType::TransformRmb,
        icon: IconName::ALargeSmall,
    },
    ToolEntry {
        label: "Base 编码",
        group: "编码/解码",
        key: "/encodedecode/base",
        view: ViewType::EncodeDecodeBaseEncoding,
        icon: IconName::CaseSensitive,
    },
    ToolEntry {
        label: "URL",
        group: "编码/解码",
        key: "/encodedecode/url",
        view: ViewType::EncodeDecodeUrl,
        icon: IconName::ExternalLink,
    },
    ToolEntry {
        label: "JWT",
        group: "编码/解码",
        key: "/encodedecode/jwt",
        view: ViewType::EncodeDecodeJwt,
        icon: IconName::File,
    },
    ToolEntry {
        label: "文本编码",
        group: "编码/解码",
        key: "/encodedecode/textencode",
        view: ViewType::EncodeDecodeCharset,
        icon: IconName::CaseSensitive,
    },
    ToolEntry {
        label: "对称加密",
        group: "加密",
        key: "/crypto/symmetric",
        view: ViewType::EncodeDecodeSymmetric,
        icon: IconName::EyeOff,
    },
    ToolEntry {
        label: "国密算法",
        group: "加密",
        key: "/crypto/sm",
        view: ViewType::EncodeDecodeSmCrypto,
        icon: IconName::Asterisk,
    },
    ToolEntry {
        label: "JSON Editor",
        group: "格式化",
        key: "/formatter/jsoneditor",
        view: ViewType::FormatterJson,
        icon: IconName::File,
    },
    ToolEntry {
        label: "SQL",
        group: "格式化",
        key: "/formatter/sql",
        view: ViewType::FormatterSql,
        icon: IconName::SquareTerminal,
    },
    ToolEntry {
        label: "XML",
        group: "格式化",
        key: "/formatter/xml",
        view: ViewType::FormatterXml,
        icon: IconName::File,
    },
    ToolEntry {
        label: "JSON Schema",
        group: "格式化",
        key: "/formatter/jsonschema",
        view: ViewType::FormatterJsonSchema,
        icon: IconName::Check,
    },
    ToolEntry {
        label: "UUID",
        group: "生成器",
        key: "/generator/uuid",
        view: ViewType::GeneratorUuid,
        icon: IconName::ALargeSmall,
    },
    ToolEntry {
        label: "Hash 计算",
        group: "生成器",
        key: "/generator/hash",
        view: ViewType::GeneratorHash,
        icon: IconName::Asterisk,
    },
    ToolEntry {
        label: "假数据生成",
        group: "数据库",
        key: "/database/datafaker",
        view: ViewType::DatabaseDatafaker,
        icon: IconName::Folder,
    },
    ToolEntry {
        label: "数据库差异",
        group: "数据库",
        key: "/database/diff",
        view: ViewType::DatabaseDiff,
        icon: IconName::Folder,
    },
    ToolEntry {
        label: "Markdown",
        group: "文本",
        key: "/text/markdown",
        view: ViewType::TextMarkdown,
        icon: IconName::BookOpen,
    },
    ToolEntry {
        label: "文本工具",
        group: "文本",
        key: "/text/tools",
        view: ViewType::TextTextTools,
        icon: IconName::BookOpen,
    },
    ToolEntry {
        label: "去空行",
        group: "文本",
        key: "/text/blankline",
        view: ViewType::TextBlankLine,
        icon: IconName::FileText,
    },
    ToolEntry {
        label: "文本 / JSON 差异",
        group: "文本",
        key: "/text/diff",
        view: ViewType::TextDiff,
        icon: IconName::Replace,
    },
    ToolEntry {
        label: "随机字符串",
        group: "随机",
        key: "/random/string",
        view: ViewType::RandomString,
        icon: IconName::CaseSensitive,
    },
    ToolEntry {
        label: "随机数字",
        group: "随机",
        key: "/random/number",
        view: ViewType::RandomNumber,
        icon: IconName::Asterisk,
    },
    ToolEntry {
        label: "随机数据",
        group: "随机",
        key: "/random/data",
        view: ViewType::RandomData,
        icon: IconName::Folder,
    },
    ToolEntry {
        label: "IP 地址转换",
        group: "网络",
        key: "/network/ip",
        view: ViewType::NetworkIp,
        icon: IconName::Network,
    },
    ToolEntry {
        label: "随机 IP / MAC / 时间",
        group: "网络",
        key: "/network/ipmac",
        view: ViewType::RandomOther,
        icon: IconName::Calendar,
    },
    ToolEntry {
        label: "按位计算器",
        group: "",
        key: "/number/bitwise",
        view: ViewType::NumberBitwise,
        icon: IconName::SquareTerminal,
    },
    ToolEntry {
        label: "颜色转换",
        group: "",
        key: "/color",
        view: ViewType::ColorConverter,
        icon: IconName::Palette,
    },
    ToolEntry {
        label: "正则",
        group: "",
        key: "/regex",
        view: ViewType::RegexVisualizer,
        icon: IconName::Dash,
    },
    ToolEntry {
        label: "图片格式转换",
        group: "图像",
        key: "/graphic/convert",
        view: ViewType::ImageConvert,
        icon: IconName::Frame,
    },
    ToolEntry {
        label: "Excalidraw",
        group: "图像",
        key: "/graphic/excalidraw",
        view: ViewType::ImageExcalidraw,
        icon: IconName::Frame,
    },
    ToolEntry {
        label: "图片转 PDF",
        group: "PDF",
        key: "/pdf/images-to-pdf",
        view: ViewType::PdfImagesToPdf,
        icon: IconName::File,
    },
    ToolEntry {
        label: "PDF 合并",
        group: "PDF",
        key: "/pdf/merge",
        view: ViewType::PdfMerge,
        icon: IconName::File,
    },
    ToolEntry {
        label: "PDF 编辑",
        group: "PDF",
        key: "/pdf/edit",
        view: ViewType::PdfEdit,
        icon: IconName::File,
    },
    ToolEntry {
        label: "PDF 添加页码",
        group: "PDF",
        key: "/pdf/page-number",
        view: ViewType::PdfPageNumber,
        icon: IconName::File,
    },
    ToolEntry {
        label: "PDF 拆分",
        group: "PDF",
        key: "/pdf/split",
        view: ViewType::PdfSplit,
        icon: IconName::File,
    },
    ToolEntry {
        label: "编码格式转换",
        group: "编码/解码",
        key: "/encodedecode/encodingformat",
        view: ViewType::EncodeDecodeEncodingFormat,
        icon: IconName::Replace,
    },
    ToolEntry {
        label: "二维码",
        group: "编码/解码",
        key: "/encodedecode/qrcode",
        view: ViewType::OtherQrCode,
        icon: IconName::Frame,
    },
    ToolEntry {
        label: "二维码解码",
        group: "编码/解码",
        key: "/encodedecode/qrcodedecode",
        view: ViewType::OtherQrCodeDecode,
        icon: IconName::Search,
    },
    ToolEntry {
        label: "剪贴板管理",
        group: "其它",
        key: "/other/clipboard",
        view: ViewType::OtherClipboard,
        icon: IconName::Settings2,
    },
];

pub fn label_pinyin(label: &str) -> String {
    use pinyin::ToPinyin;
    let mut out = String::with_capacity(label.len());
    for p in label.to_pinyin().flatten() {
        out.push_str(p.plain());
    }
    out
}

pub fn label_pinyin_initials(label: &str) -> String {
    use pinyin::ToPinyin;
    let mut out = String::with_capacity(label.len());
    for p in label.to_pinyin().flatten() {
        out.push_str(p.first_letter());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pinyin_full_and_initials() {
        assert_eq!(label_pinyin("人民币大小写"), "renminbidaxiaoxie");
        assert_eq!(label_pinyin_initials("人民币大小写"), "rmbdxx");

        assert_eq!(label_pinyin("Base 编码"), "bianma");
        assert_eq!(label_pinyin_initials("Cron 表达式"), "bds");

        assert_eq!(label_pinyin("URL"), "");
        assert_eq!(label_pinyin_initials("JWT"), "");
    }

    #[test]
    fn every_tool_is_reachable_by_pinyin() {
        for tool in TOOL_INDEX {
            let has_cjk = tool
                .label
                .chars()
                .any(|c| ('\u{4E00}'..='\u{9FFF}').contains(&c));
            if has_cjk {
                assert!(
                    !label_pinyin(tool.label).is_empty(),
                    "label 全拼为空: {}",
                    tool.label
                );
                assert!(
                    !label_pinyin_initials(tool.label).is_empty(),
                    "label 首字母为空: {}",
                    tool.label
                );
            }
        }
    }
}
