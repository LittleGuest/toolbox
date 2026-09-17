import { NIcon } from "naive-ui";
import { h } from "vue";
import {
  Home,
  LetterUu,
  Link,
  Sql,
  Xml,
  Time,
  Json,
  Barcode,
  DataFormat,
  DataBase,
  DataStructured,
  CdCreateExchange,
  Code,
  CloudMonitoring,
  NetworkPublic,
  Image,
  ToolKit,
  QrCode,
  Settings,
  TextCreation,
  TypePattern,
  Number4,
  FunctionMath,
  Translate,
  Security,
  Compare,
  ColorPalette,
  Shuffle,
  DocumentBlank,
  DirectionMerge,
  PageBreak,
  PageNumber,
  ListNumbered,
  DocumentPdf,
} from "@vicons/carbon";
import { Binary, File, Hash, Markdown } from "@vicons/tabler";
import { TransformFilled } from "@vicons/material";

import TodoIcon from '@/assets/todo.svg';

const renderMenuIcon = (icon) => {
  // 如果是字符串路径，则渲染为SVG图像
  if (typeof icon === "string") {
    return () =>
      h("img", {
        src: icon,
        width: "20",
        height: "20",
        style: "vertical-align: middle;",
      });
  }
  return () => h(NIcon, null, { default: () => h(icon) });
};

// 定义菜单树
export const menus = [
  {
    label: "首页",
    key: "/home",
    icon: renderMenuIcon(Home),
    closable: true,
  },
  {
    label: "系统监控",
    key: "/systemMonitor",
    icon: renderMenuIcon(CloudMonitoring),
  },
  {
    label: "代码片段",
    key: "/codeSnippet",
    icon: renderMenuIcon(Code),
  },
  {
    label: "待办事项",
    key: "/todo",
    icon: renderMenuIcon(TodoIcon),
  },
  {
    label: "转换",
    key: "/transform",
    icon: renderMenuIcon(TransformFilled),
    children: [
      {
        label: "文件格式转换",
        key: "/transform/filetype",
        icon: renderMenuIcon(File),
      },
      {
        label: "时间戳",
        key: "/transform/time",
        icon: renderMenuIcon(Time),
      },
      {
        label: "进制转换",
        key: "/transform/baseconversion",
        icon: renderMenuIcon(Binary),
      },
      {
        label: "Cron 表达式",
        key: "/transform/cron",
        icon: renderMenuIcon(Time),
      },
    ],
  },
  {
    label: "编码/解码",
    key: "/encodedecode",
    icon: renderMenuIcon(Barcode),
    children: [
      {
        label: "Base 编码",
        key: "/encodedecode/base",
        icon: renderMenuIcon(Barcode),
      },
      {
        label: "URL",
        key: "/encodedecode/url",
        icon: renderMenuIcon(Link),
      },
      {
        label: "JWT",
        key: "/encodedecode/jwt",
        icon: renderMenuIcon(Code),
      },
      {
        label: "文本编码",
        key: "/encodedecode/textencode",
        icon: renderMenuIcon(Translate),
      },
    ],
  },
  {
    label: "加密",
    key: "/crypto",
    icon: renderMenuIcon(Security),
  },
  {
    label: "格式化",
    key: "/formatter",
    icon: renderMenuIcon(DataFormat),
    children: [
      {
        label: "JSON Editor",
        key: "/formatter/jsoneditor",
        icon: renderMenuIcon(Json),
      },
      {
        label: "SQL",
        key: "/formatter/sql",
        icon: renderMenuIcon(Sql),
      },
      {
        label: "XML",
        key: "/formatter/xml",
        icon: renderMenuIcon(Xml),
      },
    ],
  },
  {
    label: "生成器",
    key: "/generator",
    icon: renderMenuIcon(CdCreateExchange),
    children: [
      {
        label: "UUID",
        key: "/generator/uuid",
        icon: renderMenuIcon(LetterUu),
      },
      {
        label: "Hash 计算",
        key: "/generator/hash",
        icon: renderMenuIcon(Hash),
      },
    ],
  },
  {
    label: "数据库",
    key: "/database",
    icon: renderMenuIcon(DataBase),
    children: [
      {
        label: "假数据生成",
        key: "/database/datafaker",
        icon: renderMenuIcon(DataStructured),
      },
      {
        label: "数据库差异",
        key: "/database/diff",
        icon: renderMenuIcon(DataStructured),
      },
    ],
  },
  {
    label: "文本",
    key: "/text",
    icon: renderMenuIcon(TextCreation),
    children: [
      {
        label: "Markdown",
        key: "/text/markdown",
        icon: renderMenuIcon(Markdown),
      },
      {
        label: "文本工具",
        key: "/text/tools",
        icon: renderMenuIcon(TextCreation),
      },
      {
        label: "文本 / JSON 差异",
        key: "/text/diff",
        icon: renderMenuIcon(Compare),
      },
    ],
  },
  {
    label: "随机",
    key: "/random",
    icon: renderMenuIcon(Shuffle),
    children: [
      {
        label: "随机字符串",
        key: "/random/string",
        icon: renderMenuIcon(TypePattern),
      },
      {
        label: "随机数字",
        key: "/random/number",
        icon: renderMenuIcon(Number4),
      },
      {
        label: "随机数据",
        key: "/random/data",
        icon: renderMenuIcon(DataStructured),
      },
    ],
  },
  {
    label: "网络",
    key: "/network",
    icon: renderMenuIcon(NetworkPublic),
    children: [
      {
        label: "IP 地址转换",
        key: "/network/ip",
        icon: renderMenuIcon(NetworkPublic),
      },
      {
        label: "随机 IP / MAC / 时间",
        key: "/network/ipmac",
        icon: renderMenuIcon(Time),
      },
    ],
  },
  {
    label: "按位计算器",
    key: "/number/bitwise",
    icon: renderMenuIcon(FunctionMath),
  },
  {
    label: "颜色转换",
    key: "/color",
    icon: renderMenuIcon(ColorPalette),
  },
  {
    label: "正则",
    key: "/regex",
    icon: renderMenuIcon(Barcode),
  },
  {
    label: "图像",
    key: "/image",
    icon: renderMenuIcon(Image),
    children: [
      {
        label: "图片格式转换",
        key: "/graphic/convert",
        icon: renderMenuIcon(Image),
      },
      {
        label: "Excalidraw",
        key: "/graphic/excalidraw",
        icon: renderMenuIcon(Image),
      },
    ],
  },
  {
    label: "PDF",
    key: "/pdf",
    icon: renderMenuIcon(DocumentPdf),
    children: [
      {
        label: "图片转 PDF",
        key: "/pdf/images-to-pdf",
        icon: renderMenuIcon(DocumentBlank),
      },
      {
        label: "PDF 合并",
        key: "/pdf/merge",
        icon: renderMenuIcon(DirectionMerge),
      },
      {
        label: "PDF 编辑",
        key: "/pdf/edit",
        icon: renderMenuIcon(PageBreak),
      },
      {
        label: "PDF 添加页码",
        key: "/pdf/page-number",
        icon: renderMenuIcon(PageNumber),
      },
      {
        label: "PDF 拆分",
        key: "/pdf/split",
        icon: renderMenuIcon(ListNumbered),
      },
    ],
  },
  {
    label: "其它",
    key: "/other",
    icon: renderMenuIcon(ToolKit),
    children: [
      {
        label: "二维码",
        key: "/other/qrcode",
        icon: renderMenuIcon(QrCode),
      },
      {
        label: "剪贴板管理",
        key: "/other/clipboard",
        icon: renderMenuIcon(ToolKit),
      },
    ],
  },
  // {
  //   label: "设置",
  //   key: "/setting",
  //   icon: renderMenuIcon(Settings),
  // }
];

// 所有菜单，包含子菜单，移除首页
export const menuAll = menus
  .filter((item) => item.key !== "/home")
  .flatMap((item) => {
    if (item.children) {
      return item.children;
    }
    return item;
  });

// 菜单跳转
export const navigateToMenu = (router) => {
  return (key) => {
    router.push(key);
  };
};
