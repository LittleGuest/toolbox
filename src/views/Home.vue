<script setup>
import { ref } from "vue";
import { useRouter } from "vue-router";
import { menuAll, navigateToMenu } from "@/menu.js";

const router = useRouter();
// 定义一个变量来跟踪当前悬停的卡片
const hoveredCard = ref(null);
const toMenu = navigateToMenu(router);
</script>

<template>
  <div class="home-container">
    <!-- 菜单卡片区域 -->
    <div class="menu-grid">
      <n-card
        v-for="menu in menuAll"
        :key="menu.key"
        :class="['menu-card', hoveredCard === menu.key ? 'hovered' : '']"
        :bordered="false"
        :shadow="hoveredCard === menu.key ? 'deep' : 'hover'"
        @mouseenter="hoveredCard = menu.key"
        @mouseleave="hoveredCard = null"
        @click="toMenu(menu.key, menu)"
        style="width: 240px"
      >
        <div class="card-content">
          <span class="menu-title">{{ menu.label }}</span>
        </div>
      </n-card>
    </div>

    <!-- 版权信息 -->
    <footer class="footer">
      <span class="copyright">© 2025 ToolBox 工具箱 | by 蒲建全</span>
    </footer>
  </div>
</template>

<style lang="scss" scoped>
// 定义变量
$primary-color: #ffffff; // 改为纯白色
$title-gradient: linear-gradient(90deg, #3a1c71 0%, #d76d77 50%, #ffaf7b 100%);
$icon-gradient: linear-gradient(135deg, #f5f7fa 0%, #e4eaf5 100%);
$icon-hover-gradient: linear-gradient(135deg, #f7ba1e 0%, #f15b2a 100%);
$shadow-color: rgba(247, 186, 30, 0.3);
$text-color: #333;
$secondary-text-color: #666;
$border-color: rgba(0, 0, 0, 0.1); // 调整边框颜色以适应白色背景
$footer-border-color: rgba(0, 0, 0, 0.1);
$card-bg: #f8f8f8; // 乳白色
$external-badge-bg: #f15b2a;

.home-container {
  margin: 0 auto;
  height: 100%;
  background: $primary-color; // 使用纯白色
  overflow-y: auto;

  &::-webkit-scrollbar {
    display: none;
  }

  .menu-grid {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: 10px;

    .menu-card {
      height: 100%;
      width: 240px; /* 固定宽度 */
      min-width: 240px; /* 最小宽度确保不会缩小 */
      cursor: pointer;
      transition: all 0.4s cubic-bezier(0.175, 0.885, 0.32, 1.275);
      background: $card-bg; // 使用乳白色
      border-radius: 12px;
      overflow: hidden;
      position: relative;
      margin: 16px;
      box-shadow: 0 4px 12px rgba(0, 0, 0, 0.05); // 添加轻微阴影以增强层次感

      &.hovered {
        transform: translateY(-15px) scale(1.03);
        z-index: 10;
      }

      .card-content {
        padding: 24px;
        display: flex;
        flex-direction: column;
        align-items: center;
        text-align: center;

        .icon-container {
          margin-bottom: 16px;
          position: relative;

          .menu-icon {
            width: 56px;
            height: 56px;
            display: flex;
            align-items: center;
            justify-content: center;
            border-radius: 50%;
            background: $icon-gradient;
            box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
            transition: all 0.3s ease;
          }
        }

        .menu-title {
          margin-bottom: 8px;
          color: $text-color;
          font-weight: 600;
        }
      }

      // 炫酷的背景动画效果
      &::before {
        content: "";
        position: absolute;
        top: 0;
        left: -100%;
        width: 100%;
        height: 100%;
        background: linear-gradient(
          90deg,
          transparent,
          rgba(255, 255, 255, 0.2),
          transparent
        );
        transition: left 0.6s ease;
        z-index: 0;
      }

      // 调整卡片悬停效果以适应新颜色
      &:hover {
        .menu-icon {
          background: $icon-hover-gradient;
          transform: scale(1.1);
        }

        &::before {
          left: 100%;
        }
      }

      // 为外部链接添加特殊标识
      &:has(.external-link)::after {
        content: "外部链接";
        position: absolute;
        top: 10px;
        right: 10px;
        background: $external-badge-bg;
        color: white;
        font-size: 12px;
        padding: 2px 8px;
        border-radius: 12px;
      }
    }
  }

  .footer {
    margin-top: 60px;
    padding-top: 20px;
    border-top: 1px solid $footer-border-color;
    text-align: center;

    .copyright {
      color: $secondary-text-color;
      font-size: 14px;
    }
  }
}
</style>
