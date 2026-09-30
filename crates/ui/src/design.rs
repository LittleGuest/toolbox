
use gpui_kit::*;
use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder;

pub fn page() -> Div {
    div().w_full().flex_col().gap_4()
}

pub fn page_header(title: impl IntoElement, subtitle: impl Into<SharedString>, cx: &App) -> Div {
    let subtitle = subtitle.into();
    div()
        .flex_col()
        .gap_0p5()
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(div().h(px(20.0)).w(px(3.0)).rounded(px(2.0)).bg(cx.theme().primary))
                .child(div().text_lg().font_semibold().child(title)),
        )
        .when(!subtitle.is_empty(), |this| {
            this.child(
                div()
                    .pl_5()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(subtitle.clone()),
            )
        })
}

pub fn card(cx: &App) -> Div {
    div()
        .w_full()
        .flex_col()
        .gap_3()
        .rounded(px(12.0))
        .border_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().popover)
        .p_4()
}

pub fn caption(text: impl IntoElement, cx: &App) -> Div {
    div()
        .text_xs()
        .font_medium()
        .text_color(cx.theme().muted_foreground)
        .child(text)
}

pub fn toolbar() -> Div {
    div().flex().items_center().gap_2()
}

pub fn action_row() -> Div {
    div().flex().items_center().justify_center().gap_2()
}

pub const CODE_BOX_HEIGHT: Pixels = px(220.0);

// 与 Tauri/Vue 版 global.scss 对齐的语义色
pub const ERROR_RED: u32 = 0xd03050;
pub const OK_GREEN: u32 = 0x18a058;
pub const WARN_AMBER: u32 = 0xf0a020;
pub const ACCENT: u32 = 0x7c5cff;

/// 半透明色（模拟 rgba(hex, alpha) 叠加在浅色背景上的效果）
pub fn tint(hex: u32, alpha: f32) -> Hsla {
    Hsla::from(rgb(hex)).opacity(alpha)
}

/// 卡片头：左侧 图标盒 + 标题，右侧 副标题（对应 tb-card-header）
pub fn card_header(
    icon: IconName,
    title: impl Into<SharedString>,
    subtitle: impl Into<SharedString>,
    cx: &App,
) -> Div {
    let title = title.into();
    let subtitle = subtitle.into();
    div()
        .flex()
        .items_center()
        .justify_between()
        .gap_4()
        .pb(px(14.0))
        .mb_4()
        .border_b_1()
        .border_color(cx.theme().border)
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .w(px(30.0))
                        .h(px(30.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded(px(9.0))
                        .bg(cx.theme().primary.opacity(0.1))
                        .text_color(cx.theme().primary)
                        .child(Icon::new(icon).size(px(17.0))),
                )
                .child(
                    div()
                        .text_size(px(15.0))
                        .font_semibold()
                        .text_color(cx.theme().foreground)
                        .child(title),
                ),
        )
        .child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(subtitle),
        )
}

/// 小节标签（对应 tb-editor-label：12px 加粗 灰色）
pub fn editor_label(text: impl Into<SharedString>, cx: &App) -> Div {
    div()
        .text_size(px(12.0))
        .font_semibold()
        .text_color(cx.theme().muted_foreground)
        .child(text.into())
}

/// 提示文本（对应 tb-hint：12.5px 次要色）
pub fn hint(text: impl Into<SharedString>, cx: &App) -> Div {
    div()
        .text_size(px(12.5))
        .text_color(cx.theme().muted_foreground)
        .child(text.into())
}

/// 计数胶囊（对应 pfp-count：主色底 圆角药丸）
pub fn pill(text: impl Into<SharedString>, cx: &App) -> Div {
    div()
        .px_2()
        .py(px(2.0))
        .rounded_full()
        .bg(cx.theme().primary.opacity(0.1))
        .text_size(px(12.0))
        .font_semibold()
        .text_color(cx.theme().primary)
        .child(text.into())
}

/// 迷你标签（对应 n-tag size=tiny）
pub fn mini_tag(text: impl Into<SharedString>, bg: Hsla, fg: Hsla) -> Div {
    div()
        .text_size(px(11.0))
        .px_1p5()
        .py(px(1.0))
        .rounded(px(4.0))
        .bg(bg)
        .text_color(fg)
        .child(text.into())
}

/// 统计药丸（对应 ed-stat：加粗数字 + 说明）
pub fn stat_pill(
    count: impl Into<SharedString>,
    label: impl Into<SharedString>,
    count_color: Hsla,
    cx: &App,
) -> Div {
    div()
        .flex()
        .items_center()
        .gap(px(2.0))
        .px(px(10.0))
        .py(px(4.0))
        .rounded_full()
        .border_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().background)
        .text_size(px(12.0))
        .text_color(cx.theme().muted_foreground)
        .child(
            div()
                .font_semibold()
                .text_color(count_color)
                .child(count.into()),
        )
        .child(label.into())
}

/// 提交区（对应 tb-submit-row：贴底 + 上边距）
pub fn submit_row() -> Div {
    div().mt_auto().pt(px(18.0))
}

