use std::{
    collections::HashMap,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use database::{DatasourceInfo, Driver};
use datafaker::{Column, Faker, Locale};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::{AnyPool, Row};
use tauri::{AppHandle, Emitter};
use tokio::task::JoinSet;

type Result<T> = std::result::Result<T, String>;
const INSERT_BATCH_SIZE: usize = 200;
const RUN_LOG_EVENT: &str = "datafaker-run-log";
const COMPANY_SUFFIXES: [&str; 6] = [
    "科技有限公司",
    "信息技术有限公司",
    "网络科技有限公司",
    "软件有限公司",
    "数据服务有限公司",
    "智能科技有限公司",
];
const JOB_TITLES: [&str; 12] = [
    "产品经理",
    "后端工程师",
    "前端工程师",
    "测试工程师",
    "运维工程师",
    "数据分析师",
    "架构师",
    "项目经理",
    "运营专员",
    "销售经理",
    "财务专员",
    "人事专员",
];
const DEPARTMENTS: [&str; 10] = [
    "研发部",
    "产品部",
    "测试部",
    "运维部",
    "市场部",
    "销售部",
    "财务部",
    "人力资源部",
    "行政部",
    "客服部",
];
const CHINESE_TEXT_CHARS: [&str; 96] = [
    "数", "据", "服", "务", "平", "台", "系", "统", "用", "户", "订", "单", "产", "品", "接", "口",
    "配", "置", "日", "志", "任", "务", "审", "核", "状", "态", "消", "息", "通", "知", "统", "计",
    "报", "表", "资", "源", "权", "限", "角", "色", "组", "织", "部", "门", "项", "目", "节", "点",
    "流", "程", "规", "则", "模", "板", "标", "签", "分", "类", "内", "容", "描", "述", "备", "注",
    "测", "试", "示", "例", "文", "本", "信", "息", "详", "情", "中", "心", "管", "理", "云", "智",
    "能", "安", "全", "监", "控", "开", "发", "运", "维", "财", "务", "客", "户", "营", "销", "网",
];
const CHINESE_APP_PREFIXES: [&str; 12] = [
    "云", "智", "星", "数", "企", "微", "易", "优", "快", "轻", "全", "新",
];
const CHINESE_APP_NAMES: [&str; 16] = [
    "数据平台",
    "运营中心",
    "管理系统",
    "协同助手",
    "监控平台",
    "开放平台",
    "营销助手",
    "服务中心",
    "工单系统",
    "报表中心",
    "权限中心",
    "任务平台",
    "客户系统",
    "资源中心",
    "配置平台",
    "分析平台",
];
const PINYIN_TEXT_WORDS: [&str; 32] = [
    "shuju",
    "fuwu",
    "pingtai",
    "xitong",
    "yonghu",
    "dingdan",
    "chanpin",
    "jiekou",
    "peizhi",
    "rizhi",
    "renwu",
    "shenhe",
    "zhuangtai",
    "xiaoxi",
    "tongzhi",
    "tongji",
    "baobiao",
    "ziyuan",
    "quanxian",
    "juese",
    "zuzhi",
    "bumen",
    "xiangmu",
    "liucheng",
    "guize",
    "moban",
    "biaoqian",
    "neirong",
    "miaoshu",
    "beizhu",
    "ceshi",
    "xiangqing",
];
const ENGLISH_TEXT_WORDS: [&str; 32] = [
    "data",
    "service",
    "platform",
    "system",
    "user",
    "order",
    "product",
    "api",
    "config",
    "log",
    "task",
    "review",
    "status",
    "message",
    "notice",
    "report",
    "resource",
    "permission",
    "role",
    "team",
    "project",
    "workflow",
    "rule",
    "template",
    "tag",
    "content",
    "description",
    "remark",
    "test",
    "detail",
    "center",
    "manager",
];
const PINYIN_APP_PREFIXES: [&str; 8] = ["yun", "zhi", "xing", "shu", "qi", "wei", "yi", "you"];
const PINYIN_APP_NAMES: [&str; 12] = [
    "shuju-pingtai",
    "yunying-zhongxin",
    "guanli-xitong",
    "xietong-zhushou",
    "jiankong-pingtai",
    "kaifang-pingtai",
    "yingxiao-zhushou",
    "fuwu-zhongxin",
    "gongdan-xitong",
    "baobiao-zhongxin",
    "quanxian-zhongxin",
    "fenxi-pingtai",
];
const ENGLISH_APP_NAMES: [&str; 16] = [
    "DataHub",
    "OpsCenter",
    "AdminSuite",
    "FlowDesk",
    "MonitorCloud",
    "OpenPlatform",
    "MarketPilot",
    "ServiceDesk",
    "TicketPro",
    "ReportCenter",
    "AccessCenter",
    "TaskBoard",
    "ClientPortal",
    "ResourceHub",
    "ConfigCenter",
    "InsightLab",
];
const PINYIN_COMPANY_SUFFIXES: [&str; 4] = [
    "keji-youxian-gongsi",
    "xinxi-jishu-youxian-gongsi",
    "ruanjian-youxian-gongsi",
    "shuju-fuwu-youxian-gongsi",
];
const ENGLISH_COMPANY_SUFFIXES: [&str; 5] = [
    "Technologies",
    "Software",
    "Data Services",
    "Systems",
    "Networks",
];
const PINYIN_JOB_TITLES: [&str; 8] = [
    "chanpin-jingli",
    "houduan-gongchengshi",
    "qianduan-gongchengshi",
    "ceshi-gongchengshi",
    "yunwei-gongchengshi",
    "shuju-fenxishi",
    "xiangmu-jingli",
    "yunying-zhuanyuan",
];
const ENGLISH_JOB_TITLES: [&str; 8] = [
    "Product Manager",
    "Backend Engineer",
    "Frontend Engineer",
    "QA Engineer",
    "DevOps Engineer",
    "Data Analyst",
    "Project Manager",
    "Operations Specialist",
];
const PINYIN_DEPARTMENTS: [&str; 8] = [
    "yanfa-bu",
    "chanpin-bu",
    "ceshi-bu",
    "yunwei-bu",
    "shichang-bu",
    "xiaoshou-bu",
    "caiwu-bu",
    "kefu-bu",
];
const ENGLISH_DEPARTMENTS: [&str; 8] = [
    "Engineering",
    "Product",
    "Quality Assurance",
    "Operations",
    "Marketing",
    "Sales",
    "Finance",
    "Support",
];
const PINYIN_CITIES: [&str; 10] = [
    "beijing",
    "shanghai",
    "guangzhou",
    "shenzhen",
    "hangzhou",
    "nanjing",
    "chengdu",
    "wuhan",
    "xian",
    "suzhou",
];
const CHINESE_CITIES: [&str; 10] = [
    "北京市",
    "上海市",
    "广州市",
    "深圳市",
    "杭州市",
    "南京市",
    "成都市",
    "武汉市",
    "西安市",
    "苏州市",
];
const CHINESE_PROVINCES: [&str; 10] = [
    "北京市",
    "上海市",
    "广东省",
    "浙江省",
    "江苏省",
    "四川省",
    "湖北省",
    "陕西省",
    "山东省",
    "福建省",
];
const TIMEZONES: [&str; 16] = [
    "Asia/Shanghai",
    "Asia/Hong_Kong",
    "Asia/Taipei",
    "Asia/Tokyo",
    "Asia/Seoul",
    "UTC",
    "Europe/London",
    "Europe/Berlin",
    "Europe/Paris",
    "America/New_York",
    "America/Los_Angeles",
    "America/Chicago",
    "Australia/Sydney",
    "Asia/Singapore",
    "Asia/Bangkok",
    "Asia/Dubai",
];
const COUNTRY_REGIONS: [&str; 18] = [
    "中国",
    "中国香港",
    "中国澳门",
    "中国台湾",
    "美国",
    "英国",
    "德国",
    "法国",
    "日本",
    "韩国",
    "新加坡",
    "加拿大",
    "澳大利亚",
    "新西兰",
    "泰国",
    "越南",
    "马来西亚",
    "印度尼西亚",
];
const STOCKS: [(&str, &str); 10] = [
    ("600519", "贵州茅台"),
    ("000001", "平安银行"),
    ("000333", "美的集团"),
    ("600036", "招商银行"),
    ("601318", "中国平安"),
    ("600276", "恒瑞医药"),
    ("002415", "海康威视"),
    ("300750", "宁德时代"),
    ("600900", "长江电力"),
    ("601888", "中国中免"),
];
const FUNDS: [(&str, &str); 8] = [
    ("000001", "华夏成长混合"),
    ("110011", "易方达中小盘混合"),
    ("161725", "招商中证白酒指数"),
    ("000248", "汇添富中证主要消费"),
    ("519674", "银河创新成长混合"),
    ("320007", "诺安成长混合"),
    ("001410", "信达澳银新能源产业"),
    ("005827", "易方达蓝筹精选混合"),
];
const CURRENCIES: [(&str, &str, &str); 8] = [
    ("CNY", "人民币", "¥"),
    ("USD", "美元", "$"),
    ("EUR", "欧元", "€"),
    ("GBP", "英镑", "£"),
    ("JPY", "日元", "¥"),
    ("HKD", "港币", "HK$"),
    ("AUD", "澳元", "A$"),
    ("CAD", "加元", "C$"),
];
const PAYMENT_METHODS: [&str; 10] = [
    "支付宝",
    "微信支付",
    "银行卡",
    "信用卡",
    "现金",
    "PayPal",
    "Apple Pay",
    "Google Pay",
    "银行转账",
    "数字人民币",
];
const CREDIT_CARD_TYPES: [&str; 5] = ["Visa", "MasterCard", "American Express", "UnionPay", "JCB"];
const ETHNICITIES: [&str; 12] = [
    "汉族",
    "蒙古族",
    "回族",
    "藏族",
    "维吾尔族",
    "苗族",
    "彝族",
    "壮族",
    "布依族",
    "朝鲜族",
    "满族",
    "侗族",
];
const INDUSTRIES: [&str; 12] = [
    "互联网",
    "软件服务",
    "电子商务",
    "金融科技",
    "智能制造",
    "医疗健康",
    "教育培训",
    "物流运输",
    "文化传媒",
    "新能源",
    "房地产",
    "零售消费",
];
const PRODUCT_CATEGORIES: [&str; 10] = [
    "电子产品",
    "办公用品",
    "家用电器",
    "服饰鞋包",
    "食品饮料",
    "美妆个护",
    "图书音像",
    "运动户外",
    "汽车用品",
    "母婴用品",
];
const PRODUCT_NAMES: [&str; 14] = [
    "智能手环",
    "无线耳机",
    "机械键盘",
    "移动硬盘",
    "办公椅",
    "空气净化器",
    "保温杯",
    "运动背包",
    "护眼台灯",
    "蓝牙音箱",
    "路由器",
    "显示器",
    "充电宝",
    "咖啡机",
];
const SIZE_VALUES: [&str; 12] = [
    "XS", "S", "M", "L", "XL", "XXL", "36", "38", "40", "42", "44", "46",
];
const WEIGHT_UNITS: [&str; 8] = ["mg", "g", "kg", "t", "oz", "lb", "斤", "吨"];
const IDIOMS: [&str; 18] = [
    "一帆风顺",
    "万事如意",
    "日新月异",
    "精益求精",
    "厚积薄发",
    "海纳百川",
    "脚踏实地",
    "水到渠成",
    "锦上添花",
    "鹏程万里",
    "蒸蒸日上",
    "与时俱进",
    "迎刃而解",
    "持之以恒",
    "博采众长",
    "独具匠心",
    "风雨同舟",
    "众志成城",
];
const MOBILE_MODELS: [&str; 14] = [
    "iPhone 15 Pro",
    "iPhone 14",
    "Huawei Mate 60 Pro",
    "Huawei P60",
    "Xiaomi 14",
    "Redmi K70",
    "OPPO Find X7",
    "vivo X100",
    "Honor Magic6",
    "OnePlus 12",
    "Samsung Galaxy S24",
    "Samsung Galaxy Z Flip5",
    "iQOO 12",
    "Meizu 21",
];
const DATA_TOOLS: [&str; 10] = [
    "Faker",
    "Mockaroo",
    "Datafaker",
    "Mock.js",
    "Bogus",
    "FactoryBot",
    "Faker.js",
    "RandomUser",
    "Mimesis",
    "Chance.js",
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum LocaleMode {
    EnUs,
    ZhCn,
    ZhTw,
    ZhPinyin,
}

#[tauri::command]
pub async fn datafaker_providers() -> Result<HashMap<String, String>> {
    Ok(Faker::new().get_providers().clone())
}

#[tauri::command]
pub async fn datafaker_adapter(
    field_name: Option<String>,
    field_type: Option<String>,
) -> Result<String> {
    datafaker::datafaker_adapter(field_name, field_type)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn datafaker_adapter_columns(columns: Vec<Column>) -> Result<IndexMap<String, String>> {
    let mut res = IndexMap::new();
    for column in columns {
        let generator =
            datafaker_adapter(Some(column.name.clone()), Some(column.column_type.clone())).await?;
        res.insert(column.name, generator);
    }
    Ok(res)
}

#[tauri::command]
pub async fn preview_regex(pattern: String) -> Result<String> {
    datafaker::preview_regex(pattern)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn preview_date(config: Value) -> Result<String> {
    let start_year =
        year_from_date(config.get("startDate").and_then(Value::as_str)).unwrap_or(2000);
    let end_year = year_from_date(config.get("endDate").and_then(Value::as_str)).unwrap_or(2030);
    let year = random_i64(start_year.min(end_year), start_year.max(end_year)) as i32;
    let month = random_i64(1, 12) as u8;
    let day = random_i64(1, days_in_month(year, month) as i64) as u8;
    Ok(format!("{year:04}-{month:02}-{day:02}"))
}

#[tauri::command]
pub async fn preview_datetime(config: Value) -> Result<String> {
    Ok(format!("{} {}", preview_date(config).await?, random_time()))
}

#[tauri::command]
pub async fn preview_time(_config: Value) -> Result<String> {
    Ok(random_time())
}

#[tauri::command]
pub async fn preview_email(_config: Value) -> Result<String> {
    Ok(Faker::new().internet().standard_generic_email())
}

#[tauri::command]
pub async fn preview_enum(config: Value) -> Result<String> {
    let values = config
        .get("enumValues")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    Ok(pick(&values).unwrap_or_default())
}

#[tauri::command]
pub async fn preview_file_extension(config: Value) -> Result<String> {
    Ok(pick_string_array(&config, "fileExtension")
        .unwrap_or_else(|| Faker::new().file().file_extension()))
}

#[tauri::command]
pub async fn preview_file_name(config: Value) -> Result<String> {
    let include_extension = config
        .get("includeExtension")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let faker = Faker::new();
    if include_extension {
        Ok(faker.file().name())
    } else {
        Ok(faker
            .file()
            .name()
            .rsplit_once('.')
            .map(|(name, _)| name.to_string())
            .unwrap_or_else(|| faker.file().name()))
    }
}

#[tauri::command]
pub async fn preview_file_path(_config: Value) -> Result<String> {
    Ok(Faker::new().file().path())
}

#[tauri::command]
pub async fn preview_hostname(_config: Value) -> Result<String> {
    Ok(Faker::new().internet().domain())
}

#[tauri::command]
pub async fn preview_ip(config: Value) -> Result<String> {
    let internet = Faker::new().internet();
    match config.get("ipType").and_then(Value::as_str) {
        Some("ipv6") => Ok(internet.ipv6()),
        _ => Ok(internet.ipv4()),
    }
}

#[tauri::command]
pub async fn preview_mac(_config: Value) -> Result<String> {
    Ok(Faker::new().internet().mac())
}

#[tauri::command]
pub async fn preview_name(config: Value) -> Result<String> {
    let faker = Faker::new();
    let mut name = faker.name_with_locale(name_locale(&config));
    let value = match config
        .get("format")
        .and_then(Value::as_str)
        .unwrap_or("full_name")
    {
        "first_name" => name.first_name(),
        "last_name" => name.last_name(),
        "full_name" => name.full_name(),
        _ => name.full_name(),
    };
    Ok(localize_output(value, locale_mode(&config)))
}

#[tauri::command]
pub async fn preview_number(config: Value) -> Result<String> {
    let start = config.get("start").and_then(Value::as_i64).unwrap_or(0);
    let end = config.get("end").and_then(Value::as_i64).unwrap_or(1000);
    let number_type = config
        .get("numberType")
        .and_then(Value::as_str)
        .unwrap_or("integer");
    let min = start.min(end);
    let max = start.max(end);
    if number_type == "decimal" {
        let decimal_places = config
            .get("decimalPlaces")
            .and_then(Value::as_u64)
            .unwrap_or(2)
            .min(10) as usize;
        let value = random_i64(min, max) as f64 + random_fraction();
        Ok(format!("{value:.decimal_places$}"))
    } else {
        Ok(random_i64(min, max).to_string())
    }
}

#[tauri::command]
pub async fn preview_sequence(config: Value) -> Result<String> {
    Ok(config
        .get("start")
        .and_then(Value::as_i64)
        .unwrap_or(1)
        .to_string())
}

#[tauri::command]
pub async fn preview_text(config: Value) -> Result<String> {
    if let Some(pattern) = config.get("pattern").and_then(Value::as_str) {
        return preview_regex(pattern.to_string()).await;
    }
    let min = config.get("minLength").and_then(Value::as_u64).unwrap_or(8) as usize;
    let max = config
        .get("maxLength")
        .and_then(Value::as_u64)
        .unwrap_or(32) as usize;
    Ok(random_localized_text(
        min.min(max),
        min.max(max).min(256),
        locale_mode(&config),
    ))
}

#[tauri::command]
pub async fn preview_uuid(config: Value) -> Result<String> {
    let uuid = Faker::new().uuid();
    let mut value = match config
        .get("uuidVersion")
        .and_then(Value::as_u64)
        .unwrap_or(4)
    {
        1 => uuid.uuid_v1(),
        3 => uuid.uuid_v3(),
        5 => uuid.uuid_v5(),
        6 => uuid.uuid_v6(),
        7 => uuid.uuid_v7(),
        8 => uuid.uuid_v8(),
        _ => uuid.uuid_v4(),
    };
    if config
        .get("uppercase")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        value = value.to_uppercase();
    }
    if !config
        .get("includeHyphen")
        .and_then(Value::as_bool)
        .unwrap_or(true)
    {
        value = value.replace('-', "");
    }
    Ok(value)
}

#[tauri::command]
pub async fn preview_website(_config: Value) -> Result<String> {
    Ok(Faker::new().internet().static_url())
}

#[tauri::command]
pub async fn preview_generator(generator: String, config: Value) -> Result<String> {
    match generator.as_str() {
        "date" => return preview_date(config).await,
        "datetime" => return preview_datetime(config).await,
        "time" => return preview_time(config).await,
        "email" => return preview_email(config).await,
        "enum" => return preview_enum(config).await,
        "file_extension" => return preview_file_extension(config).await,
        "file_name" => return preview_file_name(config).await,
        "file_path" => return preview_file_path(config).await,
        "hostname" => return preview_hostname(config).await,
        "ip" => return preview_ip(config).await,
        "mac" => return preview_mac(config).await,
        "name" => return preview_name(config).await,
        "number" => return preview_number(config).await,
        "text" => return preview_text(config).await,
        "uuid" => return preview_uuid(config).await,
        "website" => return preview_website(config).await,
        _ => {}
    }

    let mode = locale_mode(&config);
    let faker = Faker::new();
    let person = faker.person();
    let internet = faker.internet();
    let education = faker.education();
    let value = match generator.as_str() {
        "mobile" => person.mobile(),
        "phone" => random_phone(),
        "id_card" => person.id_card(),
        "gender" => person.gender(),
        "username" => internet.username(),
        "password" => person.strong_password(),
        "qq" => person.qq(),
        "nickname" => person.qq_nick_name(),
        "money" => format!("{:.2}", random_i64(1, 100000) as f64 + random_fraction()),
        "boolean" => random_bool().to_string(),
        "json" => json!({
            "id": random_i64(1, 1_000_000),
            "name": person.qq_nick_name(),
            "enabled": random_bool()
        })
        .to_string(),
        "address" => random_localized_address(mode),
        "city" => random_localized_city(mode),
        "state" => random_localized_state(mode),
        "street_address" => random_localized_street_address(mode),
        "zip_code" => random_localized_zip_code(mode),
        "company" => random_localized_company(mode),
        "job_title" => random_localized_job_title(mode),
        "department" => random_localized_department(mode),
        "degree" => education.degree(),
        "primary_school" => education.primary_school_name(),
        "primary_school_grade" => education.primary_school_grade(),
        "high_school" => education.high_school_name(),
        "high_school_grade" => education.high_school_grade(),
        "school_class" => education.class_name(),
        "college" => education.college().to_string(),
        "major" => education.major().to_string(),
        "timestamp" => random_i64(946_684_800, 1_893_456_000).to_string(),
        "timezone" => pick_static(&TIMEZONES),
        "province_city" => format!(
            "{}{}",
            pick_static(&CHINESE_PROVINCES),
            pick_static(&CHINESE_CITIES)
        ),
        "country_region" => pick_static(&COUNTRY_REGIONS),
        "latitude_longitude" => random_lat_lon(),
        "phone_area_code" => format!("0{}", random_i64(10, 999)),
        "stock" => random_stock(),
        "stock_kline" => random_stock_kline(),
        "fund" => random_fund(),
        "currency" => random_currency(),
        "bank_card" => random_bank_card(),
        "payment_method" => pick_static(&PAYMENT_METHODS),
        "credit_card_type" => pick_static(&CREDIT_CARD_TYPES),
        "credit_card_number" => random_credit_card_number(),
        "credit_card_date" => random_credit_card_date(),
        "ethnicity" => pick_static(&ETHNICITIES),
        "industry" => random_localized_industry(mode),
        "product_name" => random_localized_product_name(mode),
        "product_category" => random_localized_product_category(mode),
        "size" => pick_static(&SIZE_VALUES),
        "weight_unit" => pick_static(&WEIGHT_UNITS),
        "barcode" => random_ean13(),
        "sku" => random_sku(),
        "app_name" => random_localized_app_name(mode),
        "app_bundle_id" => random_app_bundle_id(),
        "app_version" => internet.app_version(),
        "user_agent" => match random_i64(0, 2) {
            0 => internet.user_agent_pc(),
            1 => internet.user_agent_android(),
            _ => internet.user_agent_ios(),
        },
        "port" => internet.port(),
        "color" => random_color(),
        "chinese_char" => pick_static(&CHINESE_TEXT_CHARS),
        "idiom" => random_localized_idiom(mode),
        "license_plate" => random_license_plate(),
        "mobile_model" => pick_static(&MOBILE_MODELS),
        "unified_social_credit_code" => random_unified_social_credit_code(),
        "data_tool" => pick_static(&DATA_TOOLS),
        "binary" => random_binary(),
        _ => random_ascii(8, 32),
    };
    Ok(value)
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunTableConfig {
    schema: String,
    table_name: String,
    columns: Vec<RunColumnConfig>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RunColumnConfig {
    name: String,
    #[serde(default)]
    column_type: Option<String>,
    #[serde(default)]
    rust_type: Option<String>,
    generator: String,
    #[serde(default)]
    config: Value,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunResult {
    success: bool,
    inserted_rows: usize,
    failed_rows: usize,
    table_stats: Vec<TableRunStat>,
    logs: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TableRunStat {
    schema: String,
    table_name: String,
    inserted_rows: usize,
    failed_rows: usize,
    elapsed_ms: u128,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RunLogEvent {
    run_id: String,
    message: String,
}

#[derive(Clone, Debug, Default)]
struct RuntimeColumnMeta {
    data_type: String,
    udt_name: String,
    max_length: Option<usize>,
    foreign_key: Option<ForeignKeyMeta>,
}

#[derive(Clone, Debug)]
struct ForeignKeyMeta {
    schema: String,
    table: String,
    column: String,
}

struct TableRuntimeContext {
    index: usize,
    table: RunTableConfig,
    metadata: HashMap<String, RuntimeColumnMeta>,
    has_foreign_key: bool,
}

struct TableInsertResult {
    index: usize,
    success: bool,
    inserted_rows: usize,
    failed_rows: usize,
    stat: Option<TableRunStat>,
    logs: Vec<String>,
}

#[tauri::command]
pub async fn datafaker_run_config(
    app_handle: AppHandle,
    datasource_info: DatasourceInfo,
    row_count: usize,
    tables: Vec<RunTableConfig>,
    run_id: String,
) -> Result<RunResult> {
    if row_count == 0 {
        return Err("生成行数必须大于 0".to_string());
    }
    if tables.is_empty() {
        return Err("请至少配置一张表".to_string());
    }

    let pool = AnyPool::connect(&datasource_info.url())
        .await
        .map_err(|e| e.to_string())?;
    let mut inserted_rows = 0;
    let mut failed_rows = 0;
    let mut table_stats = Vec::new();
    let mut logs = Vec::new();
    push_run_log(
        &app_handle,
        &run_id,
        &mut logs,
        format!(
            "开始运行假数据配置: 表数量={}, 每张表行数={row_count}",
            tables.len()
        ),
    );

    let mut table_contexts = Vec::new();
    for (index, table) in tables.into_iter().enumerate() {
        if table.columns.is_empty() {
            push_run_log(
                &app_handle,
                &run_id,
                &mut logs,
                format!("跳过表 {}: 未配置字段", table.table_name),
            );
            continue;
        }
        let metadata = load_table_metadata(&pool, &datasource_info.driver, &table)
            .await
            .map_err(|error| {
                format!(
                    "读取 {}.{} 元数据失败: {error}",
                    table.schema, table.table_name
                )
            })?;
        push_run_log(
            &app_handle,
            &run_id,
            &mut logs,
            format!(
                "开始处理表 {}.{}，字段数量={}",
                table.schema,
                table.table_name,
                table.columns.len()
            ),
        );
        let has_foreign_key = table_has_foreign_key(&table, &metadata);
        table_contexts.push(TableRuntimeContext {
            index,
            table,
            metadata,
            has_foreign_key,
        });
    }

    if table_contexts
        .iter()
        .all(|context| !context.has_foreign_key)
    {
        push_run_log(
            &app_handle,
            &run_id,
            &mut logs,
            "未检测到外键依赖，启用多表并行批量插入".to_string(),
        );
        let result = run_independent_tables_parallel(
            app_handle.clone(),
            run_id.clone(),
            pool,
            datasource_info.driver.clone(),
            row_count,
            table_contexts,
        )
        .await?;
        inserted_rows += result.inserted_rows;
        failed_rows += result.failed_rows;
        table_stats = result.table_stats;
        logs.extend(result.logs);
        emit_table_stats(&app_handle, &run_id, &mut logs, &table_stats);
        push_run_log(
            &app_handle,
            &run_id,
            &mut logs,
            format!("运行结束: 成功插入 {inserted_rows} 行，失败 {failed_rows} 行"),
        );
        return Ok(RunResult {
            success: failed_rows == 0,
            inserted_rows,
            failed_rows,
            table_stats,
            logs,
        });
    }

    push_run_log(
        &app_handle,
        &run_id,
        &mut logs,
        "检测到外键依赖，按表顺序串行插入".to_string(),
    );
    let mut generated_values: HashMap<String, Vec<String>> = HashMap::new();
    for context in table_contexts {
        let table = context.table;
        let table_start = Instant::now();
        let mut table_inserted_rows = 0;
        let mut table_failed_rows = 0;
        for row_index in 0..row_count {
            let columns = table
                .columns
                .iter()
                .map(|column| quote_identifier(&datasource_info.driver, &column.name))
                .collect::<Vec<_>>();
            let mut values = Vec::with_capacity(table.columns.len());
            for column in &table.columns {
                let meta = context.metadata.get(&column.name);
                let value = generate_runtime_value(
                    &pool,
                    &datasource_info.driver,
                    &table,
                    column,
                    meta,
                    &generated_values,
                    row_index,
                )
                .await?;
                if let Some(value) = &value {
                    generated_values
                        .entry(reference_key(
                            &table.schema,
                            &table.table_name,
                            &column.name,
                        ))
                        .or_default()
                        .push(value.clone());
                }
                values.push(sql_value(value));
            }
            let sql = format!(
                "INSERT INTO {} ({}) VALUES ({})",
                quote_table(&datasource_info.driver, &table.schema, &table.table_name),
                columns.join(", "),
                values.join(", ")
            );
            match sqlx::query(&sql).execute(&pool).await {
                Ok(_) => {
                    inserted_rows += 1;
                    table_inserted_rows += 1;
                    push_run_log(
                        &app_handle,
                        &run_id,
                        &mut logs,
                        format!(
                            "插入成功: {}.{} 第 {} 行",
                            table.schema,
                            table.table_name,
                            row_index + 1
                        ),
                    );
                }
                Err(error) => {
                    failed_rows += 1;
                    table_failed_rows += 1;
                    push_run_log(
                        &app_handle,
                        &run_id,
                        &mut logs,
                        format!(
                            "插入失败: {}.{} 第 {} 行: {}",
                            table.schema,
                            table.table_name,
                            row_index + 1,
                            error
                        ),
                    );
                    push_run_log(&app_handle, &run_id, &mut logs, format!("失败 SQL: {sql}"));
                    let stat = TableRunStat {
                        schema: table.schema.clone(),
                        table_name: table.table_name.clone(),
                        inserted_rows: table_inserted_rows,
                        failed_rows: table_failed_rows,
                        elapsed_ms: table_start.elapsed().as_millis(),
                    };
                    push_run_log(&app_handle, &run_id, &mut logs, table_stat_log(&stat));
                    table_stats.push(stat);
                    return Ok(RunResult {
                        success: false,
                        inserted_rows,
                        failed_rows,
                        table_stats,
                        logs,
                    });
                }
            }
        }
        let stat = TableRunStat {
            schema: table.schema.clone(),
            table_name: table.table_name.clone(),
            inserted_rows: table_inserted_rows,
            failed_rows: table_failed_rows,
            elapsed_ms: table_start.elapsed().as_millis(),
        };
        push_run_log(&app_handle, &run_id, &mut logs, table_stat_log(&stat));
        table_stats.push(stat);
    }

    emit_table_stats(&app_handle, &run_id, &mut logs, &table_stats);
    push_run_log(
        &app_handle,
        &run_id,
        &mut logs,
        format!("运行结束: 成功插入 {inserted_rows} 行，失败 {failed_rows} 行"),
    );
    Ok(RunResult {
        success: failed_rows == 0,
        inserted_rows,
        failed_rows,
        table_stats,
        logs,
    })
}

async fn run_independent_tables_parallel(
    app_handle: AppHandle,
    run_id: String,
    pool: AnyPool,
    driver: Driver,
    row_count: usize,
    table_contexts: Vec<TableRuntimeContext>,
) -> Result<RunResult> {
    let mut join_set = JoinSet::new();
    for context in table_contexts {
        let pool = pool.clone();
        let driver = driver.clone();
        let app_handle = app_handle.clone();
        let run_id = run_id.clone();
        join_set.spawn(async move {
            run_independent_table(app_handle, run_id, pool, driver, row_count, context).await
        });
    }

    let mut table_results = Vec::new();
    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(Ok(table_result)) => table_results.push(table_result),
            Ok(Err(error)) => {
                table_results.push(TableInsertResult {
                    index: usize::MAX,
                    success: false,
                    inserted_rows: 0,
                    failed_rows: 1,
                    stat: None,
                    logs: vec![format!("并行插入任务失败: {error}")],
                });
            }
            Err(error) => {
                table_results.push(TableInsertResult {
                    index: usize::MAX,
                    success: false,
                    inserted_rows: 0,
                    failed_rows: 1,
                    stat: None,
                    logs: vec![format!("并行插入任务异常: {error}")],
                });
            }
        }
    }

    table_results.sort_by_key(|result| result.index);
    let mut inserted_rows = 0;
    let mut failed_rows = 0;
    let mut table_stats = Vec::new();
    let mut logs = Vec::new();
    let mut success = true;
    for result in table_results {
        inserted_rows += result.inserted_rows;
        failed_rows += result.failed_rows;
        success &= result.success;
        if let Some(stat) = result.stat {
            table_stats.push(stat);
        }
        logs.extend(result.logs);
    }

    Ok(RunResult {
        success,
        inserted_rows,
        failed_rows,
        table_stats,
        logs,
    })
}

async fn run_independent_table(
    app_handle: AppHandle,
    run_id: String,
    pool: AnyPool,
    driver: Driver,
    row_count: usize,
    context: TableRuntimeContext,
) -> Result<TableInsertResult> {
    let table_start = Instant::now();
    let mut result = TableInsertResult {
        index: context.index,
        success: true,
        inserted_rows: 0,
        failed_rows: 0,
        stat: None,
        logs: Vec::new(),
    };
    push_run_log(
        &app_handle,
        &run_id,
        &mut result.logs,
        format!(
            "开始并行处理表 {}.{}，字段数量={}",
            context.table.schema,
            context.table.table_name,
            context.table.columns.len()
        ),
    );
    let columns = context
        .table
        .columns
        .iter()
        .map(|column| quote_identifier(&driver, &column.name))
        .collect::<Vec<_>>();
    let empty_generated_values = HashMap::new();
    let mut row_index = 0;
    while row_index < row_count {
        let batch_size = INSERT_BATCH_SIZE.min(row_count - row_index);
        let mut batch_values = Vec::with_capacity(batch_size);
        for offset in 0..batch_size {
            let current_row_index = row_index + offset;
            let mut values = Vec::with_capacity(context.table.columns.len());
            for column in &context.table.columns {
                let meta = context.metadata.get(&column.name);
                values.push(
                    generate_runtime_value(
                        &pool,
                        &driver,
                        &context.table,
                        column,
                        meta,
                        &empty_generated_values,
                        current_row_index,
                    )
                    .await?,
                );
            }
            batch_values.push(values);
        }

        let sql = batch_insert_sql(&driver, &context.table, &columns, &batch_values);
        match sqlx::query(&sql).execute(&pool).await {
            Ok(_) => {
                result.inserted_rows += batch_values.len();
                push_run_log(
                    &app_handle,
                    &run_id,
                    &mut result.logs,
                    format!(
                        "批量插入成功: {}.{} 第 {}-{} 行",
                        context.table.schema,
                        context.table.table_name,
                        row_index + 1,
                        row_index + batch_values.len()
                    ),
                );
            }
            Err(error) => {
                push_run_log(
                    &app_handle,
                    &run_id,
                    &mut result.logs,
                    format!(
                        "批量插入失败: {}.{} 第 {}-{} 行: {}，回退逐行插入定位错误",
                        context.table.schema,
                        context.table.table_name,
                        row_index + 1,
                        row_index + batch_values.len(),
                        error
                    ),
                );
                if !fallback_insert_rows(
                    &app_handle,
                    &run_id,
                    &pool,
                    &driver,
                    &context.table,
                    &columns,
                    &batch_values,
                    row_index,
                    table_start,
                    &mut result,
                )
                .await
                {
                    return Ok(result);
                }
            }
        }

        row_index += batch_size;
    }

    let stat = TableRunStat {
        schema: context.table.schema.clone(),
        table_name: context.table.table_name.clone(),
        inserted_rows: result.inserted_rows,
        failed_rows: result.failed_rows,
        elapsed_ms: table_start.elapsed().as_millis(),
    };
    push_run_log(
        &app_handle,
        &run_id,
        &mut result.logs,
        table_stat_log(&stat),
    );
    result.stat = Some(stat);
    Ok(result)
}

async fn fallback_insert_rows(
    app_handle: &AppHandle,
    run_id: &str,
    pool: &AnyPool,
    driver: &Driver,
    table: &RunTableConfig,
    columns: &[String],
    batch_values: &[Vec<Option<String>>],
    start_row_index: usize,
    table_start: Instant,
    result: &mut TableInsertResult,
) -> bool {
    for (offset, values) in batch_values.iter().enumerate() {
        let sql = insert_sql(driver, table, columns, values);
        match sqlx::query(&sql).execute(pool).await {
            Ok(_) => {
                result.inserted_rows += 1;
                push_run_log(
                    app_handle,
                    run_id,
                    &mut result.logs,
                    format!(
                        "插入成功: {}.{} 第 {} 行",
                        table.schema,
                        table.table_name,
                        start_row_index + offset + 1
                    ),
                );
            }
            Err(error) => {
                result.success = false;
                result.failed_rows += 1;
                push_run_log(
                    app_handle,
                    run_id,
                    &mut result.logs,
                    format!(
                        "插入失败: {}.{} 第 {} 行: {}",
                        table.schema,
                        table.table_name,
                        start_row_index + offset + 1,
                        error
                    ),
                );
                push_run_log(
                    app_handle,
                    run_id,
                    &mut result.logs,
                    format!("失败 SQL: {sql}"),
                );
                result.stat = Some(TableRunStat {
                    schema: table.schema.clone(),
                    table_name: table.table_name.clone(),
                    inserted_rows: result.inserted_rows,
                    failed_rows: result.failed_rows,
                    elapsed_ms: table_start.elapsed().as_millis(),
                });
                return false;
            }
        }
    }
    true
}

fn table_has_foreign_key(
    table: &RunTableConfig,
    metadata: &HashMap<String, RuntimeColumnMeta>,
) -> bool {
    table
        .columns
        .iter()
        .any(|column| column.generator == "foreign_key")
        || metadata.values().any(|meta| meta.foreign_key.is_some())
}

fn batch_insert_sql(
    driver: &Driver,
    table: &RunTableConfig,
    columns: &[String],
    batch_values: &[Vec<Option<String>>],
) -> String {
    let values = batch_values
        .iter()
        .map(|values| {
            format!(
                "({})",
                values
                    .iter()
                    .cloned()
                    .map(sql_value)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })
        .collect::<Vec<_>>();
    format!(
        "INSERT INTO {} ({}) VALUES {}",
        quote_table(driver, &table.schema, &table.table_name),
        columns.join(", "),
        values.join(", ")
    )
}

fn insert_sql(
    driver: &Driver,
    table: &RunTableConfig,
    columns: &[String],
    values: &[Option<String>],
) -> String {
    format!(
        "INSERT INTO {} ({}) VALUES ({})",
        quote_table(driver, &table.schema, &table.table_name),
        columns.join(", "),
        values
            .iter()
            .cloned()
            .map(sql_value)
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn push_run_log(app_handle: &AppHandle, run_id: &str, logs: &mut Vec<String>, message: String) {
    logs.push(message.clone());
    let _ = app_handle.emit(
        RUN_LOG_EVENT,
        RunLogEvent {
            run_id: run_id.to_string(),
            message,
        },
    );
}

fn emit_table_stats(
    app_handle: &AppHandle,
    run_id: &str,
    logs: &mut Vec<String>,
    table_stats: &[TableRunStat],
) {
    if table_stats.is_empty() {
        return;
    }
    push_run_log(app_handle, run_id, logs, "分表统计:".to_string());
    for stat in table_stats {
        push_run_log(app_handle, run_id, logs, table_stat_log(stat));
    }
}

fn table_stat_log(stat: &TableRunStat) -> String {
    format!(
        "{}.{}: 成功 {} 行，失败 {} 行，耗时 {} ms",
        stat.schema, stat.table_name, stat.inserted_rows, stat.failed_rows, stat.elapsed_ms
    )
}

async fn load_table_metadata(
    pool: &AnyPool,
    driver: &Driver,
    table: &RunTableConfig,
) -> Result<HashMap<String, RuntimeColumnMeta>> {
    match driver {
        Driver::Postgres => load_postgres_table_metadata(pool, table).await,
        Driver::Mysql | Driver::Sqlite => Ok(HashMap::new()),
    }
}

async fn load_postgres_table_metadata(
    pool: &AnyPool,
    table: &RunTableConfig,
) -> Result<HashMap<String, RuntimeColumnMeta>> {
    let sql = format!(
        "SELECT column_name::text AS column_name, \
                data_type::text AS data_type, \
                udt_name::text AS udt_name, \
                character_maximum_length::text AS character_maximum_length \
         FROM information_schema.columns \
         WHERE table_schema = {} AND table_name = {}",
        sql_value(Some(table.schema.clone())),
        sql_value(Some(table.table_name.clone()))
    );
    let rows = sqlx::query(&sql)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;
    let mut metadata = HashMap::new();
    for row in rows {
        let column_name: String = row.try_get("column_name").map_err(|e| e.to_string())?;
        metadata.insert(
            column_name,
            RuntimeColumnMeta {
                data_type: row.try_get("data_type").map_err(|e| e.to_string())?,
                udt_name: row.try_get("udt_name").map_err(|e| e.to_string())?,
                max_length: row
                    .try_get::<Option<String>, _>("character_maximum_length")
                    .map_err(|e| e.to_string())?
                    .and_then(|value| value.parse().ok()),
                foreign_key: None,
            },
        );
    }

    let fk_sql = format!(
        "SELECT kcu.column_name::text AS column_name, \
                ccu.table_schema::text AS foreign_table_schema, \
                ccu.table_name::text AS foreign_table_name, \
                ccu.column_name::text AS foreign_column_name \
         FROM information_schema.table_constraints AS tc \
         JOIN information_schema.key_column_usage AS kcu \
           ON tc.constraint_name = kcu.constraint_name \
          AND tc.table_schema = kcu.table_schema \
         JOIN information_schema.constraint_column_usage AS ccu \
           ON ccu.constraint_name = tc.constraint_name \
          AND ccu.table_schema = tc.table_schema \
         WHERE tc.constraint_type = 'FOREIGN KEY' \
           AND tc.table_schema = {} \
           AND tc.table_name = {}",
        sql_value(Some(table.schema.clone())),
        sql_value(Some(table.table_name.clone()))
    );
    let fk_rows = sqlx::query(&fk_sql)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;
    for row in fk_rows {
        let column_name: String = row.try_get("column_name").map_err(|e| e.to_string())?;
        if let Some(meta) = metadata.get_mut(&column_name) {
            meta.foreign_key = Some(ForeignKeyMeta {
                schema: row
                    .try_get("foreign_table_schema")
                    .map_err(|e| e.to_string())?,
                table: row
                    .try_get("foreign_table_name")
                    .map_err(|e| e.to_string())?,
                column: row
                    .try_get("foreign_column_name")
                    .map_err(|e| e.to_string())?,
            });
        }
    }

    Ok(metadata)
}

async fn generate_runtime_value(
    pool: &AnyPool,
    driver: &Driver,
    table: &RunTableConfig,
    column: &RunColumnConfig,
    meta: Option<&RuntimeColumnMeta>,
    generated_values: &HashMap<String, Vec<String>>,
    row_index: usize,
) -> Result<Option<String>> {
    if column.generator == "foreign_key" {
        return foreign_key_value(&column.config, generated_values);
    }
    if let Some(foreign_key) = meta.and_then(|meta| meta.foreign_key.as_ref()) {
        return foreign_key_runtime_value(
            pool,
            driver,
            table,
            column,
            foreign_key,
            generated_values,
        )
        .await;
    }
    if is_uuid_column(column, meta) {
        return Ok(apply_column_constraints(
            Some(preview_uuid(json!({ "uuidVersion": 4, "includeHyphen": true })).await?),
            meta,
        ));
    }
    if is_json_column(meta) {
        return Ok(apply_column_constraints(
            Some(json_default_for_column(&column.name).to_string()),
            meta,
        ));
    }
    if is_timestamp_column(meta) {
        return Ok(apply_column_constraints(
            Some("2026-01-01 00:00:00+00".to_string()),
            meta,
        ));
    }
    if is_date_column(meta) {
        return Ok(apply_column_constraints(
            Some(preview_date(column.config.clone()).await?),
            meta,
        ));
    }
    if is_time_column(meta) {
        return Ok(apply_column_constraints(
            Some(preview_time(column.config.clone()).await?),
            meta,
        ));
    }
    if is_bool_column(meta) {
        return Ok(apply_column_constraints(
            Some((random_i64(0, 1) == 1).to_string()),
            meta,
        ));
    }
    if is_number_column(meta) {
        return Ok(apply_column_constraints(
            Some(preview_number(column.config.clone()).await?),
            meta,
        ));
    }
    let value = generate_value(&column.generator, &column.config, row_index).await?;
    Ok(apply_column_constraints(value, meta))
}

async fn foreign_key_runtime_value(
    pool: &AnyPool,
    driver: &Driver,
    table: &RunTableConfig,
    column: &RunColumnConfig,
    foreign_key: &ForeignKeyMeta,
    generated_values: &HashMap<String, Vec<String>>,
) -> Result<Option<String>> {
    let key = reference_key(&foreign_key.schema, &foreign_key.table, &foreign_key.column);
    if let Some(values) = generated_values.get(&key)
        && !values.is_empty()
    {
        return Ok(Some(
            values[(random_seed() as usize) % values.len()].clone(),
        ));
    }

    let sql = format!(
        "SELECT {}::text AS value FROM {} ORDER BY random() LIMIT 1",
        quote_identifier(driver, &foreign_key.column),
        quote_table(driver, &foreign_key.schema, &foreign_key.table)
    );
    let row = sqlx::query(&sql).fetch_optional(pool).await.map_err(|e| {
        format!(
            "读取外键 {}.{} -> {}.{}.{} 失败: {e}",
            table.table_name,
            column.name,
            foreign_key.schema,
            foreign_key.table,
            foreign_key.column
        )
    })?;
    let Some(row) = row else {
        return Err(format!(
            "外键字段 {}.{}.{} 需要引用 {}.{}.{}, 但被引用表没有可用数据",
            table.schema,
            table.table_name,
            column.name,
            foreign_key.schema,
            foreign_key.table,
            foreign_key.column
        ));
    };
    row.try_get::<String, _>("value")
        .map(Some)
        .map_err(|e| e.to_string())
}

fn is_uuid_column(column: &RunColumnConfig, meta: Option<&RuntimeColumnMeta>) -> bool {
    column
        .rust_type
        .as_deref()
        .is_some_and(|value| value.eq_ignore_ascii_case("uuid"))
        || column
            .column_type
            .as_deref()
            .is_some_and(|value| value.eq_ignore_ascii_case("uuid"))
        || meta.is_some_and(|meta| {
            meta.data_type.eq_ignore_ascii_case("uuid")
                || meta.udt_name.eq_ignore_ascii_case("uuid")
        })
        || column.name.eq_ignore_ascii_case("uuid")
}

fn is_json_column(meta: Option<&RuntimeColumnMeta>) -> bool {
    meta.is_some_and(|meta| matches!(meta.data_type.as_str(), "json" | "jsonb"))
}

fn is_timestamp_column(meta: Option<&RuntimeColumnMeta>) -> bool {
    meta.is_some_and(|meta| meta.data_type.starts_with("timestamp"))
}

fn is_date_column(meta: Option<&RuntimeColumnMeta>) -> bool {
    meta.is_some_and(|meta| meta.data_type == "date")
}

fn is_time_column(meta: Option<&RuntimeColumnMeta>) -> bool {
    meta.is_some_and(|meta| meta.data_type.starts_with("time"))
}

fn is_bool_column(meta: Option<&RuntimeColumnMeta>) -> bool {
    meta.is_some_and(|meta| meta.data_type == "boolean")
}

fn is_number_column(meta: Option<&RuntimeColumnMeta>) -> bool {
    meta.is_some_and(|meta| {
        matches!(
            meta.data_type.as_str(),
            "smallint" | "integer" | "bigint" | "numeric" | "real" | "double precision" | "decimal"
        )
    })
}

fn apply_column_constraints(
    value: Option<String>,
    meta: Option<&RuntimeColumnMeta>,
) -> Option<String> {
    let Some(value) = value else {
        return None;
    };
    let Some(meta) = meta else {
        return Some(value);
    };
    if !is_limited_text_column(meta) {
        return Some(value);
    }
    let Some(max_length) = meta.max_length else {
        return Some(value);
    };
    if value.chars().count() <= max_length {
        return Some(value);
    }
    Some(value.chars().take(max_length).collect())
}

fn is_limited_text_column(meta: &RuntimeColumnMeta) -> bool {
    matches!(
        meta.data_type.as_str(),
        "character varying" | "character" | "varchar" | "char"
    ) || matches!(meta.udt_name.as_str(), "varchar" | "bpchar")
}

fn json_default_for_column(column_name: &str) -> &'static str {
    if column_name.ends_with('s')
        || column_name.contains("list")
        || column_name.contains("rules")
        || column_name.contains("configs")
        || column_name.contains("parameters")
    {
        "[]"
    } else {
        "{}"
    }
}

fn foreign_key_value(
    config: &Value,
    generated_values: &HashMap<String, Vec<String>>,
) -> Result<Option<String>> {
    if should_use_null(config) {
        return Ok(None);
    }

    let schema = config
        .get("referenceSchema")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let table = config
        .get("referenceTable")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let column = config
        .get("referenceColumn")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if table.is_empty() || column.is_empty() {
        return Err("外键生成器未选择引用字段".to_string());
    }

    let key = reference_key(schema, table, column);
    let Some(values) = generated_values.get(&key) else {
        return Err(format!(
            "外键引用字段 {schema}.{table}.{column} 尚未生成，请先把被引用表放在画布更靠前的位置"
        ));
    };
    if values.is_empty() {
        return Err(format!("外键引用字段 {schema}.{table}.{column} 没有可用值"));
    }

    Ok(Some(
        values[(random_seed() as usize) % values.len()].clone(),
    ))
}

fn reference_key(schema: &str, table: &str, column: &str) -> String {
    format!("{schema}.{table}.{column}")
}

async fn generate_value(
    generator: &str,
    config: &Value,
    row_index: usize,
) -> Result<Option<String>> {
    if should_use_null(config) {
        return Ok(None);
    }
    if should_use_default(config)
        && let Some(value) = config.get("defaultValue").and_then(Value::as_str)
    {
        return Ok(Some(value.to_string()));
    }

    let value = match generator {
        "date" => preview_date(config.clone()).await?,
        "datetime" => preview_datetime(config.clone()).await?,
        "time" => preview_time(config.clone()).await?,
        "email" => preview_email(config.clone()).await?,
        "enum" => preview_enum(config.clone()).await?,
        "file_extension" => preview_file_extension(config.clone()).await?,
        "file_name" => preview_file_name(config.clone()).await?,
        "file_path" => preview_file_path(config.clone()).await?,
        "hostname" => preview_hostname(config.clone()).await?,
        "ip" => preview_ip(config.clone()).await?,
        "mac" => preview_mac(config.clone()).await?,
        "name" => preview_name(config.clone()).await?,
        "number" => preview_number(config.clone()).await?,
        "regex" => {
            let pattern = config
                .get("pattern")
                .and_then(Value::as_str)
                .unwrap_or("[A-Za-z0-9]{10}");
            preview_regex(pattern.to_string()).await?
        }
        "sequence" => sequence_value(config, row_index),
        "text" => preview_text(config.clone()).await?,
        "uuid" => preview_uuid(config.clone()).await?,
        "website" => preview_website(config.clone()).await?,
        "foreign_key" => config
            .get("defaultValue")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        _ => preview_generator(generator.to_string(), config.clone()).await?,
    };
    Ok(Some(value))
}

fn sequence_value(config: &Value, row_index: usize) -> String {
    let start = config.get("start").and_then(Value::as_i64).unwrap_or(1);
    let step = config.get("step").and_then(Value::as_i64).unwrap_or(1);
    (start + row_index as i64 * step).to_string()
}

fn should_use_null(config: &Value) -> bool {
    config
        .get("includeNull")
        .and_then(Value::as_bool)
        .unwrap_or(false)
        && random_i64(1, 100)
            <= config
                .get("nullPercentage")
                .and_then(Value::as_i64)
                .unwrap_or(0)
}

fn should_use_default(config: &Value) -> bool {
    config
        .get("includeDefault")
        .and_then(Value::as_bool)
        .unwrap_or(false)
        && random_i64(1, 100)
            <= config
                .get("defaultPercentage")
                .and_then(Value::as_i64)
                .unwrap_or(0)
}

fn sql_value(value: Option<String>) -> String {
    match value {
        Some(value) => format!("'{}'", value.replace('\'', "''")),
        None => "NULL".to_string(),
    }
}

fn quote_table(driver: &Driver, schema: &str, table: &str) -> String {
    if schema.is_empty() {
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

fn year_from_date(date: Option<&str>) -> Option<i64> {
    date?.get(0..4)?.parse().ok()
}

fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 => 29,
        2 => 28,
        _ => 30,
    }
}

fn random_time() -> String {
    format!(
        "{:02}:{:02}:{:02}",
        random_i64(0, 23),
        random_i64(0, 59),
        random_i64(0, 59)
    )
}

fn random_i64(min: i64, max: i64) -> i64 {
    if min == max {
        return min;
    }
    let seed = random_seed();
    min + (seed % ((max - min + 1) as u64)) as i64
}

fn random_fraction() -> f64 {
    (random_seed() % 10_000) as f64 / 10_000.0
}

fn random_bool() -> bool {
    random_seed().is_multiple_of(2)
}

fn random_localized_text(min: usize, max: usize, mode: LocaleMode) -> String {
    match mode {
        LocaleMode::EnUs => random_words(min, max, &ENGLISH_TEXT_WORDS, " "),
        LocaleMode::ZhPinyin => random_words(min, max, &PINYIN_TEXT_WORDS, " "),
        LocaleMode::ZhTw => to_traditional(random_chinese_text(min, max)),
        LocaleMode::ZhCn => random_chinese_text(min, max),
    }
}

fn random_chinese_text(min: usize, max: usize) -> String {
    let len = random_i64(min as i64, max as i64) as usize;
    (0..len)
        .map(|_| pick_static(&CHINESE_TEXT_CHARS))
        .collect::<Vec<_>>()
        .join("")
}

fn random_words(min: usize, max: usize, words: &[&str], separator: &str) -> String {
    let target = random_i64(min as i64, max as i64) as usize;
    let mut value = String::new();
    while value.len() < target {
        if !value.is_empty() {
            value.push_str(separator);
        }
        value.push_str(&pick_static(words));
    }
    value.chars().take(max).collect()
}

fn random_localized_app_name(mode: LocaleMode) -> String {
    match mode {
        LocaleMode::EnUs => pick_static(&ENGLISH_APP_NAMES),
        LocaleMode::ZhPinyin => format!(
            "{}-{}",
            pick_static(&PINYIN_APP_PREFIXES),
            pick_static(&PINYIN_APP_NAMES)
        ),
        LocaleMode::ZhTw => to_traditional(random_chinese_app_name()),
        LocaleMode::ZhCn => random_chinese_app_name(),
    }
}

fn random_chinese_app_name() -> String {
    format!(
        "{}{}",
        pick_static(&CHINESE_APP_PREFIXES),
        pick_static(&CHINESE_APP_NAMES)
    )
}

fn random_localized_company(mode: LocaleMode) -> String {
    match mode {
        LocaleMode::EnUs => format!(
            "{} {}",
            capitalize(&pick_static(&ENGLISH_TEXT_WORDS)),
            pick_static(&ENGLISH_COMPANY_SUFFIXES)
        ),
        LocaleMode::ZhPinyin => format!(
            "{}-{}",
            pick_static(&PINYIN_CITIES),
            pick_static(&PINYIN_COMPANY_SUFFIXES)
        ),
        LocaleMode::ZhTw => to_traditional(random_chinese_company()),
        LocaleMode::ZhCn => random_chinese_company(),
    }
}

fn random_chinese_company() -> String {
    format!(
        "{}{}",
        random_chinese_city(),
        pick_static(&COMPANY_SUFFIXES)
    )
}

fn random_localized_job_title(mode: LocaleMode) -> String {
    match mode {
        LocaleMode::EnUs => pick_static(&ENGLISH_JOB_TITLES),
        LocaleMode::ZhPinyin => pick_static(&PINYIN_JOB_TITLES),
        LocaleMode::ZhTw => to_traditional(pick_static(&JOB_TITLES)),
        LocaleMode::ZhCn => pick_static(&JOB_TITLES),
    }
}

fn random_localized_department(mode: LocaleMode) -> String {
    match mode {
        LocaleMode::EnUs => pick_static(&ENGLISH_DEPARTMENTS),
        LocaleMode::ZhPinyin => pick_static(&PINYIN_DEPARTMENTS),
        LocaleMode::ZhTw => to_traditional(pick_static(&DEPARTMENTS)),
        LocaleMode::ZhCn => pick_static(&DEPARTMENTS),
    }
}

fn random_localized_industry(mode: LocaleMode) -> String {
    match mode {
        LocaleMode::EnUs => format!("{} Services", capitalize(&pick_static(&ENGLISH_TEXT_WORDS))),
        LocaleMode::ZhPinyin => format!("{}-hangye", pick_static(&PINYIN_TEXT_WORDS)),
        LocaleMode::ZhTw => to_traditional(pick_static(&INDUSTRIES)),
        LocaleMode::ZhCn => pick_static(&INDUSTRIES),
    }
}

fn random_localized_product_name(mode: LocaleMode) -> String {
    match mode {
        LocaleMode::EnUs => format!(
            "{} {}",
            capitalize(&pick_static(&ENGLISH_TEXT_WORDS)),
            capitalize(&pick_static(&ENGLISH_TEXT_WORDS))
        ),
        LocaleMode::ZhPinyin => format!(
            "{}-{}",
            pick_static(&PINYIN_TEXT_WORDS),
            pick_static(&PINYIN_TEXT_WORDS)
        ),
        LocaleMode::ZhTw => to_traditional(pick_static(&PRODUCT_NAMES)),
        LocaleMode::ZhCn => pick_static(&PRODUCT_NAMES),
    }
}

fn random_localized_product_category(mode: LocaleMode) -> String {
    match mode {
        LocaleMode::EnUs => capitalize(&pick_static(&ENGLISH_TEXT_WORDS)),
        LocaleMode::ZhPinyin => pick_static(&PINYIN_TEXT_WORDS),
        LocaleMode::ZhTw => to_traditional(pick_static(&PRODUCT_CATEGORIES)),
        LocaleMode::ZhCn => pick_static(&PRODUCT_CATEGORIES),
    }
}

fn random_localized_idiom(mode: LocaleMode) -> String {
    match mode {
        LocaleMode::EnUs => random_words(12, 24, &ENGLISH_TEXT_WORDS, " "),
        LocaleMode::ZhPinyin => random_words(12, 24, &PINYIN_TEXT_WORDS, " "),
        LocaleMode::ZhTw => to_traditional(pick_static(&IDIOMS)),
        LocaleMode::ZhCn => pick_static(&IDIOMS),
    }
}

fn random_localized_city(mode: LocaleMode) -> String {
    match mode {
        LocaleMode::EnUs => Faker::new().address().city(),
        LocaleMode::ZhPinyin => pick_static(&PINYIN_CITIES),
        LocaleMode::ZhTw => to_traditional(random_chinese_city()),
        LocaleMode::ZhCn => random_chinese_city(),
    }
}

fn random_localized_state(mode: LocaleMode) -> String {
    match mode {
        LocaleMode::EnUs => Faker::new().address().state(),
        LocaleMode::ZhPinyin => pick_static(&PINYIN_CITIES),
        LocaleMode::ZhTw => to_traditional(pick_static(&CHINESE_PROVINCES)),
        LocaleMode::ZhCn => pick_static(&CHINESE_PROVINCES),
    }
}

fn random_localized_street_address(mode: LocaleMode) -> String {
    match mode {
        LocaleMode::EnUs => Faker::new().address().street_address(),
        LocaleMode::ZhPinyin => format!(
            "{}-{}-lu-{}-hao",
            pick_static(&PINYIN_CITIES),
            pick_static(&PINYIN_TEXT_WORDS),
            random_i64(1, 999)
        ),
        LocaleMode::ZhTw => to_traditional(random_chinese_street_address()),
        LocaleMode::ZhCn => random_chinese_street_address(),
    }
}

fn random_localized_address(mode: LocaleMode) -> String {
    match mode {
        LocaleMode::EnUs => Faker::new().address().full_address(),
        LocaleMode::ZhPinyin => format!(
            "{} {} {}",
            random_localized_state(mode),
            random_localized_city(mode),
            random_localized_street_address(mode)
        ),
        LocaleMode::ZhTw => to_traditional(random_chinese_address()),
        LocaleMode::ZhCn => random_chinese_address(),
    }
}

fn random_localized_zip_code(mode: LocaleMode) -> String {
    match mode {
        LocaleMode::EnUs => {
            let address = Faker::new().address();
            address.zip_code(&address.state())
        }
        _ => random_i64(100000, 999999).to_string(),
    }
}

fn random_chinese_city() -> String {
    pick_static(&CHINESE_CITIES)
}

fn random_chinese_street_address() -> String {
    format!(
        "{}{}路{}号",
        random_chinese_city(),
        pick_static(&CHINESE_TEXT_CHARS),
        random_i64(1, 999)
    )
}

fn random_chinese_address() -> String {
    format!(
        "{}{}{}",
        pick_static(&CHINESE_PROVINCES),
        random_chinese_city(),
        random_chinese_street_address()
    )
}

fn random_ascii(min: usize, max: usize) -> String {
    let len = random_i64(min as i64, max as i64) as usize;
    (0..len)
        .map(|_| {
            let n = random_i64(0, 61) as u8;
            match n {
                0..=9 => char::from(b'0' + n),
                10..=35 => char::from(b'a' + n - 10),
                _ => char::from(b'A' + n - 36),
            }
        })
        .collect()
}

fn random_phone() -> String {
    format!(
        "0{}-{}",
        random_i64(10, 999),
        random_i64(10_000_000, 99_999_999)
    )
}

fn random_color() -> String {
    format!("#{:06X}", random_i64(0, 0xFF_FFFF))
}

fn random_lat_lon() -> String {
    let lat = 3.86 + (53.55 - 3.86) * random_fraction();
    let lon = 73.66 + (135.05 - 73.66) * random_fraction();
    format!("{lat:.6},{lon:.6}")
}

fn random_stock() -> String {
    let (code, name) = STOCKS[(random_seed() as usize) % STOCKS.len()];
    format!("{name}({code})")
}

fn random_stock_kline() -> String {
    let open = random_i64(500, 50_000) as f64 / 100.0;
    let close = (open + random_i64(-500, 500) as f64 / 100.0).max(0.01);
    let high = open.max(close) + random_i64(0, 300) as f64 / 100.0;
    let low = (open.min(close) - random_i64(0, 300) as f64 / 100.0).max(0.01);
    json!({
        "date": format!("2026-{:02}-{:02}", random_i64(1, 12), random_i64(1, 28)),
        "open": format!("{open:.2}"),
        "high": format!("{high:.2}"),
        "low": format!("{low:.2}"),
        "close": format!("{close:.2}"),
        "volume": random_i64(10_000, 10_000_000)
    })
    .to_string()
}

fn random_fund() -> String {
    let (code, name) = FUNDS[(random_seed() as usize) % FUNDS.len()];
    format!("{name}({code})")
}

fn random_currency() -> String {
    let (code, name, symbol) = CURRENCIES[(random_seed() as usize) % CURRENCIES.len()];
    format!("{code} {name} {symbol}")
}

fn random_bank_card() -> String {
    random_luhn_number(
        &pick_static(&["622202", "621226", "622848", "622700", "621700"]),
        19,
    )
}

fn random_credit_card_number() -> String {
    match pick_static(&CREDIT_CARD_TYPES).as_str() {
        "American Express" => random_luhn_number(&pick_static(&["34", "37"]), 15),
        "MasterCard" => random_luhn_number(&pick_static(&["51", "52", "53", "54", "55"]), 16),
        "UnionPay" => random_luhn_number("62", 16),
        "JCB" => random_luhn_number("35", 16),
        _ => random_luhn_number("4", 16),
    }
}

fn random_credit_card_date() -> String {
    format!("{:02}/{}", random_i64(1, 12), random_i64(26, 35))
}

fn random_luhn_number(prefix: &str, len: usize) -> String {
    let mut digits = prefix
        .chars()
        .filter_map(|item| item.to_digit(10))
        .collect::<Vec<_>>();
    while digits.len() + 1 < len {
        digits.push(random_i64(0, 9) as u32);
    }
    digits.push(luhn_check_digit(&digits));
    digits
        .into_iter()
        .map(|item| char::from(b'0' + item as u8))
        .collect()
}

fn luhn_check_digit(digits: &[u32]) -> u32 {
    let sum = digits
        .iter()
        .rev()
        .enumerate()
        .map(|(idx, digit)| {
            if idx.is_multiple_of(2) {
                let doubled = digit * 2;
                if doubled > 9 { doubled - 9 } else { doubled }
            } else {
                *digit
            }
        })
        .sum::<u32>();
    (10 - sum % 10) % 10
}

fn random_ean13() -> String {
    let mut digits = (0..12).map(|_| random_i64(0, 9) as u32).collect::<Vec<_>>();
    let sum = digits
        .iter()
        .enumerate()
        .map(|(idx, digit)| {
            if idx.is_multiple_of(2) {
                *digit
            } else {
                digit * 3
            }
        })
        .sum::<u32>();
    digits.push((10 - sum % 10) % 10);
    digits
        .into_iter()
        .map(|item| char::from(b'0' + item as u8))
        .collect()
}

fn random_sku() -> String {
    format!(
        "{}-{}-{}",
        random_upper_alnum(3),
        random_upper_alnum(4),
        random_i64(100, 999)
    )
}

fn random_app_bundle_id() -> String {
    format!(
        "com.{}.{}",
        random_ascii_lower(5, 10),
        random_ascii_lower(4, 10)
    )
}

fn random_license_plate() -> String {
    let province = pick_static(&[
        "京", "沪", "粤", "浙", "苏", "川", "鄂", "陕", "鲁", "闽", "湘", "皖",
    ]);
    let letter = char::from(b'A' + random_i64(0, 25) as u8);
    format!("{province}{letter}{}", random_upper_alnum(5))
}

fn random_unified_social_credit_code() -> String {
    const CHARS: &[u8] = b"0123456789ABCDEFGHJKLMNPQRTUWXY";
    const WEIGHTS: [u32; 17] = [
        1, 3, 9, 27, 19, 26, 16, 17, 20, 29, 25, 13, 8, 24, 10, 30, 28,
    ];
    let mut indexes = Vec::with_capacity(18);
    indexes.push(9);
    indexes.push(random_i64(1, 5) as usize);
    while indexes.len() < 17 {
        indexes.push(random_i64(0, (CHARS.len() - 1) as i64) as usize);
    }
    let sum = indexes
        .iter()
        .zip(WEIGHTS)
        .map(|(idx, weight)| *idx as u32 * weight)
        .sum::<u32>();
    let check_idx = (31 - sum % 31) % 31;
    indexes.push(check_idx as usize);
    indexes.into_iter().map(|idx| CHARS[idx] as char).collect()
}

fn random_binary() -> String {
    (0..16)
        .map(|_| format!("{:02X}", random_i64(0, 255)))
        .collect::<Vec<_>>()
        .join("")
}

fn random_upper_alnum(len: usize) -> String {
    (0..len)
        .map(|_| {
            let n = random_i64(0, 35) as u8;
            if n < 10 {
                char::from(b'0' + n)
            } else {
                char::from(b'A' + n - 10)
            }
        })
        .collect()
}

fn random_ascii_lower(min: usize, max: usize) -> String {
    let len = random_i64(min as i64, max as i64) as usize;
    (0..len)
        .map(|_| char::from(b'a' + random_i64(0, 25) as u8))
        .collect()
}

fn capitalize(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn random_seed() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or_default()
}

fn pick(values: &[String]) -> Option<String> {
    if values.is_empty() {
        None
    } else {
        Some(values[(random_seed() as usize) % values.len()].clone())
    }
}

fn pick_static(values: &[&str]) -> String {
    if values.is_empty() {
        String::new()
    } else {
        values[(random_seed() as usize) % values.len()].to_string()
    }
}

fn pick_string_array(config: &Value, key: &str) -> Option<String> {
    let values = config
        .get(key)?
        .as_array()?
        .iter()
        .filter_map(Value::as_str)
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    pick(&values)
}

fn name_locale(config: &Value) -> Locale {
    config
        .get("locale")
        .and_then(Value::as_str)
        .or_else(|| {
            config
                .get("locales")
                .and_then(Value::as_array)
                .and_then(|values| values.iter().filter_map(Value::as_str).next())
        })
        .map(locale_from_str)
        .unwrap_or(Locale::ZhCn)
}

fn locale_mode(config: &Value) -> LocaleMode {
    config
        .get("locale")
        .and_then(Value::as_str)
        .or_else(|| {
            config
                .get("locales")
                .and_then(Value::as_array)
                .and_then(|values| values.iter().filter_map(Value::as_str).next())
        })
        .map(locale_mode_from_str)
        .unwrap_or(LocaleMode::ZhCn)
}

fn localize_output(value: String, mode: LocaleMode) -> String {
    match mode {
        LocaleMode::ZhTw => to_traditional(value),
        _ => value,
    }
}

fn to_traditional(value: String) -> String {
    value
        .chars()
        .map(|item| match item {
            '万' => '萬',
            '与' => '與',
            '专' => '專',
            '业' => '業',
            '东' => '東',
            '丝' => '絲',
            '丢' => '丟',
            '两' => '兩',
            '严' => '嚴',
            '丧' => '喪',
            '个' => '個',
            '临' => '臨',
            '为' => '為',
            '丽' => '麗',
            '举' => '舉',
            '义' => '義',
            '乌' => '烏',
            '乐' => '樂',
            '乔' => '喬',
            '习' => '習',
            '乡' => '鄉',
            '书' => '書',
            '买' => '買',
            '乱' => '亂',
            '争' => '爭',
            '于' => '於',
            '亏' => '虧',
            '云' => '雲',
            '亚' => '亞',
            '产' => '產',
            '亩' => '畝',
            '亲' => '親',
            '亿' => '億',
            '仅' => '僅',
            '从' => '從',
            '仑' => '侖',
            '仓' => '倉',
            '仪' => '儀',
            '们' => '們',
            '价' => '價',
            '众' => '眾',
            '优' => '優',
            '会' => '會',
            '伟' => '偉',
            '传' => '傳',
            '伤' => '傷',
            '伦' => '倫',
            '伪' => '偽',
            '体' => '體',
            '余' => '餘',
            '佛' => '佛',
            '佣' => '傭',
            '佥' => '僉',
            '侠' => '俠',
            '侣' => '侶',
            '侥' => '僥',
            '侦' => '偵',
            '侧' => '側',
            '侨' => '僑',
            '侩' => '儈',
            '侪' => '儕',
            '侬' => '儂',
            '俣' => '俁',
            '俦' => '儔',
            '俨' => '儼',
            '俩' => '倆',
            '俪' => '儷',
            '俭' => '儉',
            '债' => '債',
            '倾' => '傾',
            '偬' => '傯',
            '偻' => '僂',
            '偾' => '僨',
            '偿' => '償',
            '傥' => '儻',
            '傧' => '儐',
            '储' => '儲',
            '儿' => '兒',
            '兑' => '兌',
            '兖' => '兗',
            '兰' => '蘭',
            '关' => '關',
            '兴' => '興',
            '养' => '養',
            '兽' => '獸',
            '内' => '內',
            '冈' => '岡',
            '册' => '冊',
            '写' => '寫',
            '军' => '軍',
            '农' => '農',
            '冯' => '馮',
            '冲' => '沖',
            '决' => '決',
            '况' => '況',
            '冻' => '凍',
            '净' => '淨',
            '凉' => '涼',
            '减' => '減',
            '凑' => '湊',
            '凛' => '凜',
            '凤' => '鳳',
            '凫' => '鳧',
            '凭' => '憑',
            '凯' => '凱',
            '击' => '擊',
            '凿' => '鑿',
            '刍' => '芻',
            '划' => '劃',
            '刘' => '劉',
            '则' => '則',
            '刚' => '剛',
            '创' => '創',
            '删' => '刪',
            '别' => '別',
            '刬' => '剗',
            '刭' => '剄',
            '刹' => '剎',
            '刽' => '劊',
            '刿' => '劌',
            '剀' => '剴',
            '剂' => '劑',
            '剐' => '剮',
            '剑' => '劍',
            '剥' => '剝',
            '剧' => '劇',
            '劝' => '勸',
            '办' => '辦',
            '务' => '務',
            '动' => '動',
            '励' => '勵',
            '劲' => '勁',
            '劳' => '勞',
            '势' => '勢',
            '勋' => '勳',
            '匀' => '勻',
            '匦' => '匭',
            '匮' => '匱',
            '区' => '區',
            '医' => '醫',
            '华' => '華',
            '协' => '協',
            '单' => '單',
            '卖' => '賣',
            '卢' => '盧',
            '卫' => '衛',
            '却' => '卻',
            '厂' => '廠',
            '厅' => '廳',
            '历' => '歷',
            '厉' => '厲',
            '压' => '壓',
            '厌' => '厭',
            '厦' => '廈',
            '厨' => '廚',
            '县' => '縣',
            '叁' => '參',
            '参' => '參',
            '双' => '雙',
            '发' => '發',
            '变' => '變',
            '叙' => '敘',
            '叠' => '疊',
            '叶' => '葉',
            '号' => '號',
            '叹' => '嘆',
            '叽' => '嘰',
            '吁' => '籲',
            '后' => '後',
            '吓' => '嚇',
            '吕' => '呂',
            '吗' => '嗎',
            '启' => '啟',
            '吴' => '吳',
            '员' => '員',
            '呐' => '吶',
            '呒' => '嘸',
            '呓' => '囈',
            '呕' => '嘔',
            '呖' => '嚦',
            '呗' => '唄',
            '周' => '周',
            '咨' => '諮',
            '咙' => '嚨',
            '咛' => '嚀',
            '咝' => '噝',
            '响' => '響',
            '哑' => '啞',
            '哒' => '噠',
            '哓' => '嘵',
            '哔' => '嗶',
            '哕' => '噦',
            '哗' => '譁',
            '哙' => '噲',
            '哜' => '嚌',
            '哝' => '噥',
            '哟' => '喲',
            '唛' => '嘜',
            '唠' => '嘮',
            '唡' => '啢',
            '唢' => '嗩',
            '唤' => '喚',
            '啧' => '嘖',
            '啬' => '嗇',
            '啭' => '囀',
            '啮' => '嚙',
            '啰' => '囉',
            '啸' => '嘯',
            '喷' => '噴',
            '喽' => '嘍',
            '嗳' => '噯',
            '嘘' => '噓',
            '嘤' => '嚶',
            '嘱' => '囑',
            '噜' => '嚕',
            '嚣' => '囂',
            '团' => '團',
            '园' => '園',
            '围' => '圍',
            '国' => '國',
            '图' => '圖',
            '圆' => '圓',
            '圣' => '聖',
            '场' => '場',
            '坏' => '壞',
            '块' => '塊',
            '坚' => '堅',
            '坛' => '壇',
            '坝' => '壩',
            '坞' => '塢',
            '坟' => '墳',
            '坠' => '墜',
            '垄' => '壟',
            '垅' => '壟',
            '垆' => '壚',
            '垒' => '壘',
            '垦' => '墾',
            '垩' => '堊',
            '垫' => '墊',
            '垭' => '埡',
            '垲' => '塏',
            '垴' => '堖',
            '埘' => '塒',
            '埙' => '塤',
            '埚' => '堝',
            '埯' => '垵',
            '堑' => '塹',
            '堕' => '墮',
            '墙' => '牆',
            '壮' => '壯',
            '声' => '聲',
            '壳' => '殼',
            '壶' => '壺',
            '处' => '處',
            '备' => '備',
            '复' => '復',
            '够' => '夠',
            '头' => '頭',
            '夹' => '夾',
            '夺' => '奪',
            '奋' => '奮',
            '奖' => '獎',
            '奥' => '奧',
            '妆' => '妝',
            '妇' => '婦',
            '妈' => '媽',
            '妩' => '嫵',
            '妪' => '嫗',
            '妫' => '媯',
            '姗' => '姍',
            '姜' => '姜',
            '娄' => '婁',
            '娅' => '婭',
            '娆' => '嬈',
            '娇' => '嬌',
            '娈' => '孌',
            '娱' => '娛',
            '娲' => '媧',
            '娴' => '嫻',
            '婳' => '嫿',
            '婴' => '嬰',
            '婵' => '嬋',
            '婶' => '嬸',
            '媪' => '媼',
            '嫒' => '嬡',
            '嫔' => '嬪',
            '嫱' => '嬙',
            '孙' => '孫',
            '学' => '學',
            '宁' => '寧',
            '宝' => '寶',
            '实' => '實',
            '宠' => '寵',
            '审' => '審',
            '宪' => '憲',
            '宫' => '宮',
            '宽' => '寬',
            '宾' => '賓',
            '寝' => '寢',
            '对' => '對',
            '寻' => '尋',
            '导' => '導',
            '寿' => '壽',
            '将' => '將',
            '尔' => '爾',
            '尘' => '塵',
            '尝' => '嘗',
            '尧' => '堯',
            '尴' => '尷',
            '尸' => '屍',
            '尽' => '盡',
            '层' => '層',
            '屉' => '屜',
            '届' => '屆',
            '属' => '屬',
            '屡' => '屢',
            '屦' => '屨',
            '屿' => '嶼',
            '岁' => '歲',
            '岂' => '豈',
            '岖' => '嶇',
            '岗' => '崗',
            '岘' => '峴',
            '岙' => '嶴',
            '岛' => '島',
            '岭' => '嶺',
            '岳' => '嶽',
            '峡' => '峽',
            '峣' => '嶢',
            '峤' => '嶠',
            '峥' => '崢',
            '峦' => '巒',
            '崂' => '嶗',
            '崃' => '崍',
            '崄' => '嶮',
            '嵘' => '嶸',
            '嵚' => '嶔',
            '嵝' => '嶁',
            '巅' => '巔',
            '巩' => '鞏',
            '币' => '幣',
            '帅' => '帥',
            '师' => '師',
            '帐' => '帳',
            '帘' => '簾',
            '帜' => '幟',
            '带' => '帶',
            '帮' => '幫',
            '帱' => '幬',
            '帻' => '幘',
            '帼' => '幗',
            '幂' => '冪',
            '庄' => '莊',
            '庆' => '慶',
            '庐' => '廬',
            '庑' => '廡',
            '库' => '庫',
            '应' => '應',
            '庙' => '廟',
            '庞' => '龐',
            '废' => '廢',
            '广' => '廣',
            '廪' => '廩',
            '开' => '開',
            '异' => '異',
            '弃' => '棄',
            '张' => '張',
            '弥' => '彌',
            '弯' => '彎',
            '弹' => '彈',
            '强' => '強',
            '归' => '歸',
            '当' => '當',
            '录' => '錄',
            '彦' => '彥',
            '彻' => '徹',
            '径' => '徑',
            '徕' => '徠',
            '忆' => '憶',
            '忏' => '懺',
            '忧' => '憂',
            '怀' => '懷',
            '态' => '態',
            '怂' => '慫',
            '怃' => '憮',
            '怄' => '慪',
            '怅' => '悵',
            '怆' => '愴',
            '怜' => '憐',
            '总' => '總',
            '怼' => '懟',
            '恋' => '戀',
            '恒' => '恆',
            '恳' => '懇',
            '恶' => '惡',
            '恸' => '慟',
            '恹' => '懨',
            '恺' => '愷',
            '恻' => '惻',
            '恼' => '惱',
            '恽' => '惲',
            '悦' => '悅',
            '悬' => '懸',
            '悭' => '慳',
            '悯' => '憫',
            '惊' => '驚',
            '惧' => '懼',
            '惨' => '慘',
            '惩' => '懲',
            '惫' => '憊',
            '惬' => '愜',
            '惭' => '慚',
            '惮' => '憚',
            '惯' => '慣',
            '愠' => '慍',
            '愤' => '憤',
            '愿' => '願',
            '慑' => '懾',
            '懑' => '懣',
            '懒' => '懶',
            '戆' => '戇',
            '戋' => '戔',
            '戏' => '戲',
            '战' => '戰',
            '戬' => '戩',
            '户' => '戶',
            '扎' => '紮',
            '扑' => '撲',
            '执' => '執',
            '扩' => '擴',
            '扫' => '掃',
            '扬' => '揚',
            '扰' => '擾',
            '抚' => '撫',
            '抛' => '拋',
            '抟' => '摶',
            '抢' => '搶',
            '护' => '護',
            '报' => '報',
            '担' => '擔',
            '拟' => '擬',
            '拢' => '攏',
            '拣' => '揀',
            '拥' => '擁',
            '拦' => '攔',
            '拧' => '擰',
            '拨' => '撥',
            '择' => '擇',
            '挂' => '掛',
            '挚' => '摯',
            '挛' => '攣',
            '挜' => '掗',
            '挝' => '撾',
            '挞' => '撻',
            '挟' => '挾',
            '挠' => '撓',
            '挡' => '擋',
            '挢' => '撟',
            '挣' => '掙',
            '挤' => '擠',
            '挥' => '揮',
            '挦' => '撏',
            '捞' => '撈',
            '损' => '損',
            '捡' => '撿',
            '换' => '換',
            '捣' => '搗',
            '据' => '據',
            '捻' => '撚',
            '掳' => '擄',
            '掴' => '摑',
            '掷' => '擲',
            '掸' => '撣',
            '掺' => '摻',
            '掼' => '摜',
            '揽' => '攬',
            '揾' => '搵',
            '揿' => '撳',
            '搀' => '攙',
            '搁' => '擱',
            '搂' => '摟',
            '搅' => '攪',
            '携' => '攜',
            '摄' => '攝',
            '摅' => '攄',
            '摆' => '擺',
            '摇' => '搖',
            '摈' => '擯',
            '摊' => '攤',
            '撄' => '攖',
            '撑' => '撐',
            '撵' => '攆',
            '撷' => '擷',
            '撸' => '擼',
            '撺' => '攛',
            '擞' => '擻',
            '攒' => '攢',
            '敌' => '敵',
            '敛' => '斂',
            '数' => '數',
            '斋' => '齋',
            '斓' => '斕',
            '斗' => '鬥',
            '斩' => '斬',
            '断' => '斷',
            '无' => '無',
            '旧' => '舊',
            '时' => '時',
            '旷' => '曠',
            '昙' => '曇',
            '昼' => '晝',
            '显' => '顯',
            '晋' => '晉',
            '晒' => '曬',
            '晓' => '曉',
            '晔' => '曄',
            '晕' => '暈',
            '晖' => '暉',
            '暂' => '暫',
            '暧' => '曖',
            '术' => '術',
            '机' => '機',
            '杀' => '殺',
            '杂' => '雜',
            '权' => '權',
            '杆' => '桿',
            '条' => '條',
            '来' => '來',
            '杨' => '楊',
            '杰' => '傑',
            '极' => '極',
            '构' => '構',
            '枞' => '樅',
            '枢' => '樞',
            '枣' => '棗',
            '枥' => '櫪',
            '枧' => '梘',
            '枨' => '棖',
            '枪' => '槍',
            '枫' => '楓',
            '枭' => '梟',
            '柜' => '櫃',
            '柠' => '檸',
            '柽' => '檉',
            '栀' => '梔',
            '栅' => '柵',
            '标' => '標',
            '栈' => '棧',
            '栉' => '櫛',
            '栋' => '棟',
            '栌' => '櫨',
            '栎' => '櫟',
            '栏' => '欄',
            '树' => '樹',
            '栖' => '棲',
            '样' => '樣',
            '栾' => '欒',
            '桠' => '椏',
            '桡' => '橈',
            '桢' => '楨',
            '档' => '檔',
            '桤' => '榿',
            '桥' => '橋',
            '桦' => '樺',
            '桧' => '檜',
            '桨' => '槳',
            '桩' => '樁',
            '梦' => '夢',
            '梼' => '檮',
            '梾' => '棶',
            '检' => '檢',
            '棂' => '櫺',
            '椁' => '槨',
            '椟' => '櫝',
            '椠' => '槧',
            '椤' => '欏',
            '椭' => '橢',
            '楼' => '樓',
            '榄' => '欖',
            '榇' => '櫬',
            '榈' => '櫚',
            '榉' => '櫸',
            '槚' => '檟',
            '槛' => '檻',
            '槟' => '檳',
            '槠' => '櫧',
            '横' => '橫',
            '樯' => '檣',
            '樱' => '櫻',
            '橥' => '櫫',
            '橱' => '櫥',
            '橹' => '櫓',
            '橼' => '櫞',
            '檩' => '檁',
            '欢' => '歡',
            '欧' => '歐',
            '歼' => '殲',
            '殁' => '歿',
            '殇' => '殤',
            '残' => '殘',
            '殒' => '殞',
            '殓' => '殮',
            '殚' => '殫',
            '殡' => '殯',
            '殴' => '毆',
            '毁' => '毀',
            '毂' => '轂',
            '毕' => '畢',
            '毙' => '斃',
            '毡' => '氈',
            '毵' => '毿',
            '气' => '氣',
            '氢' => '氫',
            '氩' => '氬',
            '氲' => '氳',
            '汉' => '漢',
            '汤' => '湯',
            '汹' => '洶',
            '沟' => '溝',
            '没' => '沒',
            '沣' => '灃',
            '沤' => '漚',
            '沥' => '瀝',
            '沦' => '淪',
            '沧' => '滄',
            '沪' => '滬',
            '泞' => '濘',
            '泪' => '淚',
            '泶' => '澩',
            '泷' => '瀧',
            '泸' => '瀘',
            '泺' => '濼',
            '泻' => '瀉',
            '泼' => '潑',
            '泽' => '澤',
            '泾' => '涇',
            '洁' => '潔',
            '洒' => '灑',
            '洼' => '窪',
            '浃' => '浹',
            '浅' => '淺',
            '浆' => '漿',
            '浇' => '澆',
            '浈' => '湞',
            '浊' => '濁',
            '测' => '測',
            '济' => '濟',
            '浏' => '瀏',
            '浐' => '滻',
            '浑' => '渾',
            '浓' => '濃',
            '浔' => '潯',
            '涛' => '濤',
            '涝' => '澇',
            '涞' => '淶',
            '涟' => '漣',
            '涠' => '潿',
            '涡' => '渦',
            '涢' => '溳',
            '涣' => '渙',
            '涤' => '滌',
            '润' => '潤',
            '涧' => '澗',
            '涨' => '漲',
            '涩' => '澀',
            '淀' => '澱',
            '渊' => '淵',
            '渌' => '淥',
            '渍' => '漬',
            '渎' => '瀆',
            '渐' => '漸',
            '渑' => '澠',
            '渔' => '漁',
            '渖' => '瀋',
            '渗' => '滲',
            '温' => '溫',
            '湾' => '灣',
            '湿' => '濕',
            '溃' => '潰',
            '溅' => '濺',
            '溆' => '漵',
            '滞' => '滯',
            '滟' => '灧',
            '滠' => '灄',
            '满' => '滿',
            '滢' => '瀅',
            '滤' => '濾',
            '滥' => '濫',
            '滦' => '灤',
            '滨' => '濱',
            '滩' => '灘',
            '潆' => '瀠',
            '潇' => '瀟',
            '潋' => '瀲',
            '潍' => '濰',
            '潜' => '潛',
            '潴' => '瀦',
            '澜' => '瀾',
            '濑' => '瀨',
            '濒' => '瀕',
            '灏' => '灝',
            '灭' => '滅',
            '灯' => '燈',
            '灵' => '靈',
            '灾' => '災',
            '灿' => '燦',
            '炀' => '煬',
            '炉' => '爐',
            '炖' => '燉',
            '炜' => '煒',
            '炝' => '熗',
            '点' => '點',
            '炼' => '煉',
            '炽' => '熾',
            '烁' => '爍',
            '烂' => '爛',
            '烃' => '烴',
            '烛' => '燭',
            '烟' => '煙',
            '烦' => '煩',
            '烧' => '燒',
            '烨' => '燁',
            '烩' => '燴',
            '烫' => '燙',
            '烬' => '燼',
            '热' => '熱',
            '焕' => '煥',
            '焖' => '燜',
            '焘' => '燾',
            '爱' => '愛',
            '爷' => '爺',
            '牍' => '牘',
            '牵' => '牽',
            '牺' => '犧',
            '犊' => '犢',
            '状' => '狀',
            '犷' => '獷',
            '犸' => '獁',
            '犹' => '猶',
            '狈' => '狽',
            '狝' => '獮',
            '狞' => '獰',
            '独' => '獨',
            '狭' => '狹',
            '狮' => '獅',
            '狯' => '獪',
            '狰' => '猙',
            '狱' => '獄',
            '狲' => '猻',
            '猃' => '獫',
            '猎' => '獵',
            '猕' => '獼',
            '猡' => '玀',
            '猪' => '豬',
            '猫' => '貓',
            '献' => '獻',
            '玑' => '璣',
            '玛' => '瑪',
            '玮' => '瑋',
            '环' => '環',
            '现' => '現',
            '玱' => '瑲',
            '玺' => '璽',
            '珉' => '珉',
            '珏' => '玨',
            '珐' => '琺',
            '珑' => '瓏',
            '珰' => '璫',
            '珲' => '琿',
            '琏' => '璉',
            '琐' => '瑣',
            '琼' => '瓊',
            '瑶' => '瑤',
            '瑷' => '璦',
            '璎' => '瓔',
            '瓒' => '瓚',
            '台' => '臺',
            '统' => '統',
            '订' => '訂',
            '品' => '品',
            '接' => '接',
            '口' => '口',
            '配' => '配',
            '置' => '置',
            '日' => '日',
            '志' => '誌',
            '任' => '任',
            '核' => '核',
            '消' => '消',
            '息' => '息',
            '通' => '通',
            '知' => '知',
            '计' => '計',
            '表' => '表',
            '资' => '資',
            '源' => '源',
            '限' => '限',
            '组' => '組',
            '织' => '織',
            '门' => '門',
            '项' => '項',
            '节' => '節',
            '流' => '流',
            '程' => '程',
            '规' => '規',
            '签' => '籤',
            '类' => '類',
            '容' => '容',
            '描' => '描',
            '述' => '述',
            '注' => '註',
            '试' => '試',
            '详' => '詳',
            '情' => '情',
            '中' => '中',
            '心' => '心',
            '管' => '管',
            '理' => '理',
            '智' => '智',
            '能' => '能',
            '安' => '安',
            '全' => '全',
            '监' => '監',
            '控' => '控',
            '运' => '運',
            '维' => '維',
            '财' => '財',
            '销' => '銷',
            '网' => '網',
            '浙' => '浙',
            '苏' => '蘇',
            '陕' => '陝',
            '软' => '軟',
            '件' => '件',
            '营' => '營',
            '职' => '職',
            _ => item,
        })
        .collect()
}

fn locale_mode_from_str(locale: &str) -> LocaleMode {
    match locale {
        "en_us" => LocaleMode::EnUs,
        "zh_traditional" | "zh_tw" => LocaleMode::ZhTw,
        "zh_pinyin" => LocaleMode::ZhPinyin,
        _ => LocaleMode::ZhCn,
    }
}

fn locale_from_str(locale: &str) -> Locale {
    match locale {
        "en_us" => Locale::EnUs,
        "zh_traditional" | "zh_tw" => Locale::ZhTw,
        "zh_pinyin" => Locale::ZhPinyin,
        _ => Locale::ZhCn,
    }
}
