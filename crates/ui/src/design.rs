
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

