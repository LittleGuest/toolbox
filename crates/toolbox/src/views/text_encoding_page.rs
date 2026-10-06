//! 「文本编码」容器页 —— 严格对齐 Tauri `src/views/encodedecode/TextEncoding.vue`：
//! 单页 4 个 Tab（字符编码 / 乱码恢复 / 转义工具 / Unicode 转义），
//! 之前被拆成 4 个独立侧栏项，现合并回同一页面。

use gpui_kit::{
    component::tab::{Tab, TabBar},
    *,
};

use crate::views::{CharsetEncoder, EscapeTools, MessyCodeRecover, Utf8UnicodeConverter};

const TABS: [&str; 4] = ["字符编码", "乱码恢复", "转义工具", "Unicode 转义"];

pub struct TextEncodingPage {
    active_tab: usize,
    charset: Option<Entity<CharsetEncoder>>,
    messy: Option<Entity<MessyCodeRecover>>,
    escape: Option<Entity<EscapeTools>>,
    unicode: Option<Entity<Utf8UnicodeConverter>>,
}

impl TextEncodingPage {
    pub fn new(_window: &mut Window, _cx: &mut Context<Self>) -> Self {
        Self {
            active_tab: 0,
            charset: None,
            messy: None,
            escape: None,
            unicode: None,
        }
    }
}

impl Render for TextEncodingPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // 懒创建当前 Tab 的子视图（与 main.rs 的 render_xxx_view 缓存模式一致）
        match self.active_tab {
            0 if self.charset.is_none() => {
                self.charset = Some(cx.new(|cx| CharsetEncoder::new(window, cx)));
            }
            1 if self.messy.is_none() => {
                self.messy = Some(cx.new(|cx| MessyCodeRecover::new(window, cx)));
            }
            2 if self.escape.is_none() => {
                self.escape = Some(cx.new(|cx| EscapeTools::new(window, cx)));
            }
            3 if self.unicode.is_none() => {
                self.unicode = Some(cx.new(|cx| Utf8UnicodeConverter::new(window, cx)));
            }
            _ => {}
        }

        let weak = cx.entity().downgrade();
        let tab_bar = TabBar::new("text-encoding-tabs")
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
            0 => div().children(self.charset.clone()),
            1 => div().children(self.messy.clone()),
            2 => div().children(self.escape.clone()),
            _ => div().children(self.unicode.clone()),
        };

        // 对齐 Tauri TextEncoding.vue：tb-page > tb-card > n-tabs，Tab 在 card 内
        design::page().child(
            design::card(cx)
                .child(div().mb_1().child(tab_bar))
                .child(content),
        )
    }
}
