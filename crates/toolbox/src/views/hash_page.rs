//! 「Hash 计算」容器页 —— 严格对齐 Tauri `src/views/generator/Hash.vue`：
//! 单页 2 个 Tab（文本 / 文件校验），文件校验之前被拆成独立侧栏项，现合并回同一页面。

use gpui_kit::{
    component::tab::{Tab, TabBar},
    *,
};

use crate::views::{FileVerify, HashCalculator};

const TABS: [&str; 2] = ["文本", "文件校验"];

pub struct HashPage {
    active_tab: usize,
    text_hash: Option<Entity<HashCalculator>>,
    file_verify: Option<Entity<FileVerify>>,
}

impl HashPage {
    pub fn new(_window: &mut Window, _cx: &mut Context<Self>) -> Self {
        Self {
            active_tab: 0,
            text_hash: None,
            file_verify: None,
        }
    }
}

impl Render for HashPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        match self.active_tab {
            0 if self.text_hash.is_none() => {
                self.text_hash = Some(cx.new(|cx| HashCalculator::new(window, cx)));
            }
            1 if self.file_verify.is_none() => {
                self.file_verify = Some(cx.new(|cx| FileVerify::new(window, cx)));
            }
            _ => {}
        }

        let weak = cx.entity().downgrade();
        let tab_bar = TabBar::new("hash-page-tabs")
            .underline()
            .selected_index(self.active_tab)
            .on_click(move |index: &usize, _, cx| {
                if let Some(this) = weak.upgrade() {
                    this.update(cx, |this, cx| {
                        this.active_tab = *index;
                        cx.notify();
                    });
                }
            })
            .children(TABS.iter().map(|label| Tab::new().label(*label)));

        let content: Div = match self.active_tab {
            0 => div().children(self.text_hash.clone()),
            _ => div().children(self.file_verify.clone()),
        };

        // 对齐 Tauri Hash.vue：tb-page > tb-card > n-tabs，Tab 在 card 内
        design::page().child(
            design::card(cx)
                .child(div().mb_1().child(tab_bar))
                .child(content),
        )
    }
}
