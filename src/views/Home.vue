<script setup lang="ts">
import { ref, computed } from "vue";
import { useRouter } from "vue-router";
import { menus, navigateToMenu } from "@/menu";

const router = useRouter();
const toMenu = navigateToMenu(router);
const hoveredCard = ref(null);

// 工具简述（用于首页卡片副标题）
const toolDesc: Record<string, string> = {
  "/transform/filetype": "文件类型识别与转换",
  "/transform/time": "时间戳与日期互转",
  "/transform/baseconversion": "进制及字符串进制互转",
  "/transform/cron": "Cron 表达式解析与生成",
  "/encodedecode/base": "Base64 / Base32 / Base58 编码解码",
  "/encodedecode/url": "URL 编码与解码",
  "/encodedecode/jwt": "JWT 解析与验证",
  "/encodedecode/textencode": "字符集 / 乱码 / 转义 / Unicode 编码转换",
  "/crypto": "AES / DES / RC4 等对称加密",
  "/formatter/jsoneditor": "JSON 编辑与格式化",
  "/formatter/sql": "SQL 语句格式化",
  "/formatter/xml": "XML 格式化与压缩",
  "/generator/uuid": "批量生成 UUID",
  "/generator/hash": "文本与文件 Hash 计算",
  "/database/datafaker": "可视化构造假数据",
  "/database/diff": "数据库结构与数据对比",
  "/text/markdown": "Markdown 编辑与预览",
  "/text/tools": "大小写 / 清理 / 统计等文本处理",
  "/text/diff": "文本 / JSON 差异对比",
  "/random/string": "随机字符串生成",
  "/random/number": "随机数字生成",
  "/random/data": "结构化随机数据",
  "/network/ip": "IP 地址数值转换",
  "/network/ipmac": "随机 IP / MAC / 时间",
  "/number/bitwise": "按位运算计算器",
  "/color": "颜色格式转换",
  "/graphic/convert": "图片格式转换",
  "/graphic/excalidraw": "手绘风格画板",
  "/pdf/images-to-pdf": "多张图片合成 PDF",
  "/pdf/merge": "合并多个 PDF",
  "/pdf/edit": "编辑 PDF 页面",
  "/pdf/page-number": "为 PDF 添加页码",
  "/pdf/split": "拆分 PDF 页面",
  "/other/qrcode": "生成二维码",
  "/other/clipboard": "剪贴板历史管理",
  "/regex": "正则表达式可视化",
  "/systemMonitor": "系统资源实时监控",
  "/codeSnippet": "代码片段管理",
  "/todo": "待办事项清单",
  "/setting": "应用设置",
};

// 组装首页工具列表（含父级分组信息）
const toolCards = computed(() => {
  const cards: any[] = [];
  for (const group of menus as any[]) {
    if (group.key === "/home") continue;
    if (group.children) {
      for (const child of group.children) {
        cards.push({
          key: child.key,
          label: child.label,
          desc: toolDesc[child.key] || "",
          icon: child.icon,
          group: group.label,
        });
      }
    } else {
      cards.push({
        key: group.key,
        label: group.label,
        desc: toolDesc[group.key] || "",
        icon: group.icon,
        group: "",
      });
    }
  }
  return cards;
});

const groupedCards = computed(() => {
  const map: Record<string, any[]> = {};
  for (const card of toolCards.value) {
    const g = card.group || "常用";
    if (!map[g]) map[g] = [];
    map[g].push(card);
  }
  return Object.entries(map);
});
</script>

<template>
  <div class="home">
    <section v-for="[group, cards] in groupedCards" :key="group" class="home-group">
      <h2 class="home-group-title">{{ group }}</h2>
      <div class="home-grid">
        <div
          v-for="card in cards"
          :key="card.key"
          class="home-card"
          :class="hoveredCard === card.key ? 'hovered' : ''"
          @mouseenter="hoveredCard = card.key"
          @mouseleave="hoveredCard = null"
          @click="toMenu(card.key)"
        >
          <div class="home-card-icon">
            <component :is="card.icon" />
          </div>
          <div class="home-card-text">
            <div class="home-card-title">{{ card.label }}</div>
            <div class="home-card-desc">{{ card.desc }}</div>
          </div>
        </div>
      </div>
    </section>

    <footer class="home-footer">
      <span>ToolBox · 开发工具集合</span>
    </footer>
  </div>
</template>

<style scoped>
.home {
  max-width: 1280px;
  margin: 0 auto;
  padding: 8px 24px 40px;
}

.home-group {
  margin-top: 24px;
}

.home-group-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--tb-text-3);
  letter-spacing: 0.06em;
  margin: 0 0 12px;
  padding-left: 2px;
}

.home-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(248px, 1fr));
  gap: 14px;
}

.home-card {
  display: flex;
  align-items: flex-start;
  gap: 14px;
  background: var(--tb-bg-elevated);
  border: 1px solid var(--tb-border);
  border-radius: var(--tb-radius-l);
  box-shadow: var(--tb-shadow-card);
  padding: 18px;
  cursor: pointer;
  transition: border-color 0.2s ease, transform 0.2s ease, box-shadow 0.2s ease;
}

.home-card:hover,
.home-card.hovered {
  border-color: var(--tb-primary);
  box-shadow: 0 4px 20px rgba(79, 110, 247, 0.12);
  transform: translateY(-2px);
}

.home-card-icon {
  width: 40px;
  height: 40px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 11px;
  background: var(--tb-primary-weak);
  color: var(--tb-primary);
  font-size: 20px;
}

.home-card-text {
  min-width: 0;
}

.home-card-title {
  font-size: 14.5px;
  font-weight: 600;
  color: var(--tb-text);
  margin-bottom: 4px;
}

.home-card-desc {
  font-size: 12.5px;
  color: var(--tb-text-3);
  line-height: 1.5;
}

.home-footer {
  margin-top: 40px;
  padding-top: 20px;
  border-top: 1px solid var(--tb-border);
  text-align: center;
  color: var(--tb-text-3);
  font-size: 12.5px;
}

@media (max-width: 760px) {
  .home {
    padding: 8px 16px 32px;
  }
  .home-grid {
    grid-template-columns: 1fr;
  }
}
</style>