<script setup lang="ts">
import { inject, computed, ref } from "vue";
import { useRouter, useRoute } from "vue-router";
import { menus, navigateToMenu } from "./menu";
import { NButton, NIcon, useThemeVars } from "naive-ui";
import { Moon, Sun, ToolBox, Search } from "@vicons/carbon";
import { pinyin as toPinyin } from "pinyin-pro";

const router = useRouter();
const route = useRoute();
const theme = inject("theme");
const toggleTheme = inject("toggleTheme");
const themeVars = useThemeVars();

const toMenu = navigateToMenu(router);

const collapsed = ref(false);

const search = ref("");

const flatTools = computed(() => {
  const list: { key: string; label: string; group: string }[] = [];
  for (const group of menus as any[]) {
    if (group.children) {
      for (const child of group.children) {
        list.push({ key: child.key, label: child.label, group: group.label });
      }
    } else {
      list.push({ key: group.key, label: group.label, group: "" });
    }
  }
  return list;
});

const searchOptions = computed(() => {
  const kw = search.value.trim().toLowerCase();
  if (!kw) return [];

  const index = flatTools.value.map((t) => {
    const p = toPinyin(t.label, { toneType: "none", type: "array" }).join("");
    const first = toPinyin(t.label, { pattern: "first", toneType: "none", type: "array" }).join("");
    return { ...t, pinyin: p.toLowerCase(), first: first.toLowerCase() };
  });

  return index
    .filter(
      (t) =>
        t.label.toLowerCase().includes(kw) ||
        t.key.toLowerCase().includes(kw) ||
        (t.group && t.group.toLowerCase().includes(kw)) ||
        t.pinyin.includes(kw) ||
        t.first.includes(kw)
    )
    .slice(0, 12)
    .map((t) => ({
      value: t.key,
      label: t.group ? `${t.label}（${t.group}）` : t.label,
    }));
});

const onSelectTool = (key: string | number) => {
  search.value = "";
  toMenu(String(key));
};

const siderBg = computed(() => themeVars.value.bodyColor);
const siderBorder = computed(() => themeVars.value.borderColor);

const siderWidth = computed(() => (collapsed.value ? 64 : 240));

const activeKey = computed(() => route.path);
</script>

<template>
  <n-layout has-sider position="absolute" :style="{ background: 'var(--tb-bg-app)' }">
    <n-layout-sider
      bordered
      collapse-mode="width"
      :collapsed-width="64"
      :width="siderWidth"
      :collapsed="collapsed"
      show-trigger="bar"
      :native-scrollbar="false"
      :style="{
        background: siderBg,
        borderColor: siderBorder,
        transition: 'width .2s cubic-bezier(.4,0,.2,1)',
      }"
    >
      <div class="tb-brand" :class="{ 'is-collapsed': collapsed }">
        <div class="tb-brand-left">
          <div class="tb-brand-logo">
            <n-icon size="22">
              <ToolBox />
            </n-icon>
          </div>
          <transition name="fade">
            <span v-if="!collapsed" class="tb-brand-name">ToolBox</span>
          </transition>
        </div>
        <n-tooltip trigger="hover" placement="right">
          <template #trigger>
            <n-button
              quaternary
              circle
              size="small"
              class="tb-theme-btn"
              @click="toggleTheme"
            >
              <template #icon>
                <n-icon>
                  <template v-if="theme === 'light'">
                    <Moon />
                  </template>
                  <template v-else>
                    <Sun />
                  </template>
                </n-icon>
              </template>
            </n-button>
          </template>
          {{ theme === 'light' ? '切换深色模式' : '切换浅色模式' }}
        </n-tooltip>
      </div>

      <n-menu
        :options="menus"
        :value="activeKey"
        :collapsed="collapsed"
        :collapsed-width="64"
        :indent="18"
        :root-indent="14"
        :style="{ background: 'transparent' }"
        @update:value="toMenu"
      />
    </n-layout-sider>

    <div class="tb-main">
      <div class="tb-topbar">
        <div class="tb-search">
          <n-icon class="tb-search-icon">
            <Search />
          </n-icon>
          <n-auto-complete
            v-model:value="search"
            :options="searchOptions"
            :input-props="{ placeholder: '搜索工具，快速切换…' }"
            clearable
            size="large"
            style="flex: 1; min-width: 0"
            @select="onSelectTool"
          />
        </div>
      </div>

      <div class="tb-scroll">
        <div class="tb-viewport">
          <router-view />
        </div>
      </div>
    </div>
  </n-layout>
</template>

<style scoped>
.tb-brand {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: var(--tb-header-h);
  padding: 0 18px;
  overflow: hidden;
}

.tb-brand-left {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
}

.tb-brand-logo {
  width: 32px;
  height: 32px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 9px;
  background: var(--tb-primary-weak);
  color: var(--tb-primary);
}

.tb-brand-name {
  font-size: 16px;
  font-weight: 650;
  letter-spacing: -0.01em;
  color: var(--tb-text);
  white-space: nowrap;
}

.tb-brand.is-collapsed {
  padding: 0 16px;
  justify-content: center;
}

.tb-brand.is-collapsed .tb-brand-left {
  display: none;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.15s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.tb-topbar {
  flex-shrink: 0;
  height: var(--tb-header-h);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0 20px;
  border-bottom: 1px solid var(--tb-border);
  background: var(--tb-bg-elevated);
}

.tb-search {
  position: relative;
  display: flex;
  align-items: center;
  width: 100%;
  max-width: 560px;
}

.tb-search-icon {
  position: absolute;
  left: 12px;
  z-index: 1;
  color: var(--tb-text-3);
  pointer-events: none;
  font-size: 16px;
}

.tb-search :deep(.n-input__input-el) {
  padding-left: 34px;
}

.tb-viewport {
  height: 100%;
  min-height: 100%;
}

.tb-main {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.tb-scroll {
  flex: 1;
  min-height: 0;
  width: 100%;
  overflow-y: auto;
  overflow-x: hidden;
}
</style>
