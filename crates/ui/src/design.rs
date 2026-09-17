//! 统一设计系统：为所有 GPUI 工具页面提供一致的「页面外壳 / 卡片分区 / 操作条 / 说明文字」。
//!
//! 设计语言：轻盈的卡片式开发者工具。
//! - 页面 = 标题条（标题 + 副标题）+ 一组卡片分区；
//! - 卡片 = 柔和圆角 + 细边框 + 表面色背景，内部 `gap_3` 纵向节奏；
//! - 功能区用 `caption` 小号灰字标注，主操作按钮用 `primary`；
//! - 全程基于主题语义色（popover/border/muted_foreground/primary...），深浅主题自适应。

use gpui_kit::*;
use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder;

/// 页面根容器：铺满内容区宽度，纵向统一留白。
pub fn page() -> Div {
    div().w_full().flex_col().gap_4()
}

/// 页面标题条：标题（加粗）+ 副标题（灰色小字）。
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

/// 卡片容器：柔和圆角 + 细边框 + 表面色背景。
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

/// 卡片内小标题 / 功能区说明（灰字小号）。
pub fn caption(text: impl IntoElement, cx: &App) -> Div {
    div()
        .text_xs()
        .font_medium()
        .text_color(cx.theme().muted_foreground)
        .child(text)
}

/// 通用操作行：一行按钮水平排列。
pub fn toolbar() -> Div {
    div().flex().items_center().gap_2()
}

/// 居中的主操作行（如 编码/解码 按钮）。
pub fn action_row() -> Div {
    div().flex().items_center().justify_center().gap_2()
}

/// 输入/输出等代码类文本框统一高度。
pub const CODE_BOX_HEIGHT: Pixels = px(220.0);

