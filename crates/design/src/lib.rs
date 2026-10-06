use gpui_kit::{component::*, *};

pub fn page() -> Div {
    div()
        .w_full()
        .flex_col()
        .gap_4()
        .pt(px(16.0))
        .px(px(20.0))
        .pb(px(24.0))
}

pub fn card(cx: &App) -> Div {
    div()
        .w_full()
        .flex_col()
        .gap_4()
        .rounded(px(14.0))
        .border_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().popover)
        .p(px(20.0))
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

pub const ERROR_RED: u32 = 0xd03050;
pub const OK_GREEN: u32 = 0x18a058;
pub const WARN_AMBER: u32 = 0xf0a020;
pub const ACCENT: u32 = 0x7c5cff;

pub fn tint(hex: u32, alpha: f32) -> Hsla {
    Hsla::from(rgb(hex)).opacity(alpha)
}

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

pub fn editor_label(text: impl Into<SharedString>, cx: &App) -> Div {
    div()
        .text_size(px(12.0))
        .font_semibold()
        .text_color(cx.theme().muted_foreground)
        .child(text.into())
}

pub fn hint(text: impl Into<SharedString>, cx: &App) -> Div {
    div()
        .text_size(px(12.5))
        .text_color(cx.theme().muted_foreground)
        .child(text.into())
}

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

pub fn submit_row() -> Div {
    div().mt_auto().pt(px(18.0))
}
