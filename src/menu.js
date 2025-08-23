import { NIcon } from "naive-ui";
import {
  Home,
  LetterUu,
  Link,
  Sql,
  TextUnderline,
  Xml,
  Time,
  Json,
  Barcode,
  DataFormat,
  TextItalic,
  DocumentExport,
  DataBase,
  DataStructured,
  CdCreateExchange,
  Code,
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
        label: "OpenApi",
        key: "/transform/openapi",
        icon: renderMenuIcon(DocumentExport),
      },
    ],
  },
  {
    label: "编码/解码",
    key: "/encodedecode",
    icon: renderMenuIcon(Barcode),
    children: [
      {
        label: "Base64",
        key: "/encodedecode/base64text",
        icon: renderMenuIcon(TextUnderline),
      },
      {
        label: "URL",
        key: "/encodedecode/url",
        icon: renderMenuIcon(Link),
      },
    ],
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
        label: "文本Hash",
        key: "/generator/hash",
        icon: renderMenuIcon(Hash),
      },
      {
        label: "文件校验",
        key: "/generator/checksum",
        icon: renderMenuIcon(File),
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
    icon: renderMenuIcon(TextItalic),
    children: [
      {
        label: "Markdown",
        key: "/text/markdown",
        icon: renderMenuIcon(Markdown),
      },
    ],
  },
  // {
  //   label: "网络",
  //   key: "/network",
  //   icon: renderIcon(NetworkPublic),
  //   children: [
  //     {
  //       label: "IP",
  //       key: "/network/ip",
  //       icon: renderIcon(NetworkPublic),
  //     }
  //   ]
  // },
  // {
  //   label: "图像",
  //   key: "/image",
  //   icon: renderIcon(Image),
  //   children: [
  //     {
  //       label: "Excalidraw",
  //       key: "/graphic/excalidraw",
  //       icon: renderIcon(NetworkPublic),
  //     }
  //   ]
  // },
  // {
  //   label: "其它",
  //   key: "/other",
  //   icon: renderIcon(ToolKit),
  //   children: [
  //     {
  //       label: "剪切板",
  //       key: "/other/clipboard",
  //       icon: renderIcon(Clipboard),
  //     },
  //     {
  //       label: "二维码",
  //       key: "/other/qrcode",
  //       icon: renderIcon(QrCode),
  //     }
  //   ]
  // },
  {
    label: "外链",
    key: "external",
    icon: renderMenuIcon(Link),
    children: [
      {
        label: "Excalidraw",
        key: "https://excalidraw.com/",
        icon: renderMenuIcon(Link),
        external: true,
      },
      {
        label: "正则可视化",
        key: "https://regex-vis.com/",
        icon: renderMenuIcon(Link),
        external: true,
      },
      {
        label: "正则测试",
        key: "https://regex101.com/",
        icon: renderMenuIcon(Link),
        external: true,
      },
    ],
  },
];

// 所有菜单，包含子菜单，外部链接，移除首页
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
  return (key, item) => {
    if (item.external) {
      window.open(key);
      return;
    }
    router.push(key);
  };
};
