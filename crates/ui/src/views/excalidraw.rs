use std::{cell::Cell, rc::Rc};

use gpui::{prelude::FluentBuilder as _, *};
use gpui_component::{
    WindowExt,
    button::*,
    input::{Input, InputState},
    *,
};
use serde::{Deserialize, Serialize};

use crate::config_store;

const DEFAULT_STROKE: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const DEFAULT_FILL: [f32; 4] = [0.0, 0.0, 1.0, 0.15];
const TEXT_FONT_SIZE: f32 = 16.0;

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
enum ShapeKind {
    Rectangle,
    Ellipse,
    Diamond,
    Line,
    Arrow,
    Text,
    Freedraw,
}

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
enum StrokeStyleKind {
    Solid,
    Dashed,
}

#[derive(Clone, Serialize, Deserialize)]
struct ExcalidrawElement {
    id: usize,
    kind: ShapeKind,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    text: String,
    #[serde(default)]
    points: Vec<(f32, f32)>,
    stroke: [f32; 4],
    fill: [f32; 4],
    stroke_width: f32,
    stroke_style: StrokeStyleKind,
}

#[derive(Clone, Copy, PartialEq)]
enum Tool {
    Select,
    Freedraw,
    Rectangle,
    Ellipse,
    Diamond,
    Line,
    Arrow,
    Text,
    Eraser,
}

impl Tool {
    fn label(&self) -> &'static str {
        match self {
            Tool::Select => "选择",
            Tool::Freedraw => "手绘",
            Tool::Rectangle => "矩形",
            Tool::Ellipse => "椭圆",
            Tool::Diamond => "菱形",
            Tool::Line => "直线",
            Tool::Arrow => "箭头",
            Tool::Text => "文本",
            Tool::Eraser => "橡皮",
        }
    }
}

enum Drag {
    Create {
        start_x: f32,
        start_y: f32,
    },
    Move {
        index: usize,
        offset_x: f32,
        offset_y: f32,
    },
    Freedraw {
        index: usize,
    },
    Pan {
        start_mouse_x: f32,
        start_mouse_y: f32,
        origin_vx: f32,
        origin_vy: f32,
    },
}

struct ColorPreset {
    label: &'static str,
    hsla: Hsla,
    arr: [f32; 4],
}

fn color_presets() -> Vec<ColorPreset> {
    vec![
        ColorPreset {
            label: "黑",
            hsla: hsla(0.0, 0.0, 0.0, 1.0),
            arr: [0.0, 0.0, 0.0, 1.0],
        },
        ColorPreset {
            label: "红",
            hsla: hsla(0.0, 0.84, 0.6, 1.0),
            arr: [0.0, 0.84, 0.6, 1.0],
        },
        ColorPreset {
            label: "橙",
            hsla: hsla(0.08, 0.84, 0.6, 1.0),
            arr: [0.08, 0.84, 0.6, 1.0],
        },
        ColorPreset {
            label: "黄",
            hsla: hsla(0.14, 0.84, 0.6, 1.0),
            arr: [0.14, 0.84, 0.6, 1.0],
        },
        ColorPreset {
            label: "绿",
            hsla: hsla(0.33, 0.72, 0.5, 1.0),
            arr: [0.33, 0.72, 0.5, 1.0],
        },
        ColorPreset {
            label: "蓝",
            hsla: hsla(0.6, 0.72, 0.55, 1.0),
            arr: [0.6, 0.72, 0.55, 1.0],
        },
        ColorPreset {
            label: "紫",
            hsla: hsla(0.78, 0.72, 0.55, 1.0),
            arr: [0.78, 0.72, 0.55, 1.0],
        },
        ColorPreset {
            label: "透明",
            hsla: hsla(0.0, 0.0, 0.0, 0.0),
            arr: [0.0, 0.0, 0.0, 0.0],
        },
    ]
}

pub struct ExcalidrawView {
    elements: Vec<ExcalidrawElement>,
    tool: Tool,
    next_id: usize,
    selection: Option<usize>,
    drag: Option<Drag>,
    viewport_x: f32,
    viewport_y: f32,
    zoom: f32,
    history: Vec<Vec<ExcalidrawElement>>,
    redo_stack: Vec<Vec<ExcalidrawElement>>,
    stroke_color: [f32; 4],
    fill_color: [f32; 4],
    stroke_width: f32,
    stroke_style: StrokeStyleKind,
    status: String,
    doc_name_state: Entity<InputState>,
    text_edit_state: Entity<InputState>,
    saved_docs: Vec<config_store::ExcalidrawDocRecord>,
    canvas_origin: Rc<Cell<Point<Pixels>>>,
}

impl ExcalidrawView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let canvas_origin = Rc::new(Cell::new(point(px(0.0), px(0.0))));
        Self {
            elements: Vec::new(),
            tool: Tool::Select,
            next_id: 1,
            selection: None,
            drag: None,
            viewport_x: 0.0,
            viewport_y: 0.0,
            zoom: 1.0,
            history: Vec::new(),
            redo_stack: Vec::new(),
            stroke_color: DEFAULT_STROKE,
            fill_color: DEFAULT_FILL,
            stroke_width: 2.0,
            stroke_style: StrokeStyleKind::Solid,
            status: "选择工具后在画布上拖拽创建图形。中键拖拽平移画布，Ctrl+滚轮缩放。".to_string(),
            doc_name_state: cx.new(|cx| InputState::new(window, cx).placeholder("文档名称")),
            text_edit_state: cx.new(|cx| InputState::new(window, cx).placeholder("文本内容")),
            saved_docs: Vec::new(),
            canvas_origin,
        }
    }

    fn set_tool(&mut self, tool: Tool, cx: &mut Context<Self>) {
        self.tool = tool;
        self.selection = None;
        self.status = format!("当前工具：{}", tool.label());
        cx.notify();
    }

    fn push_history(&mut self) {
        self.history.push(self.elements.clone());
        if self.history.len() > 50 {
            self.history.remove(0);
        }
        self.redo_stack.clear();
    }

    fn undo(&mut self, cx: &mut Context<Self>) {
        if let Some(prev) = self.history.pop() {
            self.redo_stack
                .push(std::mem::replace(&mut self.elements, prev));
            self.selection = None;
            self.status = "已撤销。".to_string();
            cx.notify();
        } else {
            self.status = "没有可撤销的操作。".to_string();
            cx.notify();
        }
    }

    fn redo(&mut self, cx: &mut Context<Self>) {
        if let Some(next) = self.redo_stack.pop() {
            self.history
                .push(std::mem::replace(&mut self.elements, next));
            self.selection = None;
            self.status = "已重做。".to_string();
            cx.notify();
        } else {
            self.status = "没有可重做的操作。".to_string();
            cx.notify();
        }
    }

    fn delete_selected(&mut self, cx: &mut Context<Self>) {
        if let Some(index) = self.selection {
            self.push_history();
            self.elements.remove(index);
            self.selection = None;
            self.status = "已删除选中元素。".to_string();
            cx.notify();
        }
    }

    fn clear(&mut self, cx: &mut Context<Self>) {
        if self.elements.is_empty() {
            return;
        }
        self.push_history();
        self.elements.clear();
        self.selection = None;
        self.status = "画布已清空。".to_string();
        cx.notify();
    }

    fn export_json(&mut self, cx: &mut Context<Self>) {
        let json = serde_json::to_string_pretty(&self.elements).unwrap_or_default();
        cx.write_to_clipboard(ClipboardItem::new_string(json));
        self.status = format!("已导出 {} 个元素的 JSON 到剪贴板。", self.elements.len());
        cx.notify();
    }

    fn save_doc(&mut self, cx: &mut Context<Self>) {
        let name = self.doc_name_state.read(cx).value().to_string();
        if name.trim().is_empty() {
            self.status = "请输入文档名称。".to_string();
            cx.notify();
            return;
        }
        let elements_json = serde_json::to_string(&self.elements).unwrap_or_default();
        let name_clone = name.clone();
        self.status = format!("正在保存文档 {name}...");
        cx.notify();

        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            let result = config_store::save_excalidraw_doc(&name_clone, &elements_json).await;
            let _ = this.update(cx, |this, cx| {
                this.status = match result {
                    Ok(_) => format!("文档 {name_clone} 已保存。"),
                    Err(err) => format!("保存文档失败：{err}"),
                };
                cx.notify();
            });
        })
        .detach();
    }

    fn load_docs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.status = "正在加载文档列表...".to_string();
        cx.notify();

        cx.spawn_in(window, async move |this: WeakEntity<Self>, cx| {
            let result = config_store::load_excalidraw_docs().await;
            let _ = this.update_in(cx, |this, window, cx| {
                match result {
                    Ok(docs) => {
                        this.status = format!("已加载 {} 个文档。", docs.len());
                        this.saved_docs = docs.clone();
                        this.show_load_dialog(docs, window, cx);
                    }
                    Err(err) => {
                        this.status = format!("加载文档列表失败：{err}");
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn show_load_dialog(
        &mut self,
        docs: Vec<config_store::ExcalidrawDocRecord>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let this = cx.entity().downgrade();
        window.open_dialog(cx, move |dialog, _, cx| {
            dialog
                .title(
                    div()
                        .text_lg()
                        .font_semibold()
                        .child("选择 Excalidraw 文档"),
                )
                .width(px(640.))
                .child(doc_load_dialog_content(docs.clone(), this.clone(), cx))
        });
    }

    fn apply_doc(
        &mut self,
        elements_json: String,
        name: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let elements: Vec<ExcalidrawElement> =
            serde_json::from_str(&elements_json).unwrap_or_default();
        if let Some(max_id) = elements.iter().map(|e| e.id).max() {
            self.next_id = max_id + 1;
        }
        self.push_history();
        self.elements = elements;
        self.selection = None;
        self.doc_name_state.update(cx, |state, cx| {
            state.set_value(name.clone(), window, cx);
        });
        self.status = format!("已加载文档 {name}，共 {} 个元素。", self.elements.len());
        cx.notify();
    }

    fn delete_doc(&mut self, name: String, cx: &mut Context<Self>) {
        self.status = format!("正在删除文档 {name}...");
        cx.notify();

        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            let result = config_store::delete_excalidraw_doc(name.clone()).await;
            let _ = this.update(cx, |this, cx| {
                this.status = match result {
                    Ok(true) => format!("文档 {name} 已删除。"),
                    Ok(false) => format!("未找到文档 {name}。"),
                    Err(err) => format!("删除文档失败：{err}"),
                };
                cx.notify();
            });
        })
        .detach();
    }

    fn set_stroke_color(&mut self, color: [f32; 4], cx: &mut Context<Self>) {
        self.stroke_color = color;
        if let Some(index) = self.selection {
            self.push_history();
            if let Some(elem) = self.elements.get_mut(index) {
                elem.stroke = color;
            }
        }
        cx.notify();
    }

    fn set_fill_color(&mut self, color: [f32; 4], cx: &mut Context<Self>) {
        self.fill_color = color;
        if let Some(index) = self.selection {
            self.push_history();
            if let Some(elem) = self.elements.get_mut(index) {
                elem.fill = color;
            }
        }
        cx.notify();
    }

    fn set_stroke_width(&mut self, width: f32, cx: &mut Context<Self>) {
        self.stroke_width = width;
        if let Some(index) = self.selection {
            self.push_history();
            if let Some(elem) = self.elements.get_mut(index) {
                elem.stroke_width = width;
            }
        }
        cx.notify();
    }

    fn set_stroke_style(&mut self, style: StrokeStyleKind, cx: &mut Context<Self>) {
        self.stroke_style = style;
        if let Some(index) = self.selection {
            self.push_history();
            if let Some(elem) = self.elements.get_mut(index) {
                elem.stroke_style = style;
            }
        }
        cx.notify();
    }

    fn update_selected_text(&mut self, cx: &mut Context<Self>) {
        let text = self.text_edit_state.read(cx).value().to_string();
        if let Some(index) = self.selection {
            if let Some(elem) = self.elements.get_mut(index) {
                if elem.kind == ShapeKind::Text {
                    elem.text = text;
                    cx.notify();
                }
            }
        }
    }

    fn canvas_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let origin = self.canvas_origin.get();
        let mouse_x = f32::from(event.position.x) - f32::from(origin.x);
        let mouse_y = f32::from(event.position.y) - f32::from(origin.y);
        let canvas_x = (mouse_x - self.viewport_x) / self.zoom;
        let canvas_y = (mouse_y - self.viewport_y) / self.zoom;

        // Middle button: pan
        if event.button == MouseButton::Middle {
            self.drag = Some(Drag::Pan {
                start_mouse_x: f32::from(event.position.x),
                start_mouse_y: f32::from(event.position.y),
                origin_vx: self.viewport_x,
                origin_vy: self.viewport_y,
            });
            cx.notify();
            return;
        }

        if event.button != MouseButton::Left {
            return;
        }

        match self.tool {
            Tool::Select => {
                if let Some(index) = hit_test(&self.elements, canvas_x, canvas_y) {
                    self.selection = Some(index);
                    let elem = &self.elements[index];
                    let offset_x = canvas_x - elem.x;
                    let offset_y = canvas_y - elem.y;
                    let is_text = elem.kind == ShapeKind::Text;
                    let text = elem.text.clone();
                    let elem_id = elem.id;
                    self.push_history();
                    self.drag = Some(Drag::Move {
                        index,
                        offset_x,
                        offset_y,
                    });
                    if is_text {
                        self.text_edit_state.update(cx, |state, cx| {
                            state.set_value(text, _window, cx);
                        });
                    }
                    self.status = format!("已选中元素 #{}。", elem_id);
                } else {
                    self.selection = None;
                    self.status = "已取消选择。".to_string();
                }
                cx.notify();
            }
            Tool::Eraser => {
                if let Some(index) = hit_test(&self.elements, canvas_x, canvas_y) {
                    self.push_history();
                    self.elements.remove(index);
                    self.selection = None;
                    self.status = "已擦除元素。".to_string();
                    cx.notify();
                }
            }
            Tool::Text => {
                self.push_history();
                let id = self.next_id;
                self.next_id += 1;
                let elem = ExcalidrawElement {
                    id,
                    kind: ShapeKind::Text,
                    x: canvas_x,
                    y: canvas_y,
                    width: 120.0,
                    height: TEXT_FONT_SIZE + 8.0,
                    text: "文本".to_string(),
                    points: Vec::new(),
                    stroke: self.stroke_color,
                    fill: self.fill_color,
                    stroke_width: self.stroke_width,
                    stroke_style: self.stroke_style,
                };
                self.elements.push(elem);
                let index = self.elements.len() - 1;
                self.selection = Some(index);
                self.text_edit_state.update(cx, |state, cx| {
                    state.set_value("文本".to_string(), _window, cx);
                });
                self.status = "已创建文本元素，在属性面板编辑内容。".to_string();
                cx.notify();
            }
            Tool::Freedraw => {
                self.push_history();
                let id = self.next_id;
                self.next_id += 1;
                let elem = ExcalidrawElement {
                    id,
                    kind: ShapeKind::Freedraw,
                    x: canvas_x,
                    y: canvas_y,
                    width: 0.0,
                    height: 0.0,
                    text: String::new(),
                    points: vec![(0.0, 0.0)],
                    stroke: self.stroke_color,
                    fill: [0.0, 0.0, 0.0, 0.0],
                    stroke_width: self.stroke_width,
                    stroke_style: self.stroke_style,
                };
                self.elements.push(elem);
                let index = self.elements.len() - 1;
                self.drag = Some(Drag::Freedraw { index });
                cx.notify();
            }
            Tool::Rectangle | Tool::Ellipse | Tool::Diamond | Tool::Line | Tool::Arrow => {
                self.push_history();
                self.drag = Some(Drag::Create {
                    start_x: canvas_x,
                    start_y: canvas_y,
                });
                cx.notify();
            }
        }
    }

    fn canvas_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let origin = self.canvas_origin.get();
        let mouse_x = f32::from(event.position.x) - f32::from(origin.x);
        let mouse_y = f32::from(event.position.y) - f32::from(origin.y);
        let canvas_x = (mouse_x - self.viewport_x) / self.zoom;
        let canvas_y = (mouse_y - self.viewport_y) / self.zoom;

        let Some(drag) = self.drag.take() else {
            return;
        };

        match drag {
            Drag::Create { start_x, start_y } => {
                let x = canvas_x.min(start_x);
                let y = canvas_y.min(start_y);
                let width = (canvas_x - start_x).abs();
                let height = (canvas_y - start_y).abs();

                // Remove the preview element if it exists (last element with matching tool)
                let kind = match self.tool {
                    Tool::Rectangle => ShapeKind::Rectangle,
                    Tool::Ellipse => ShapeKind::Ellipse,
                    Tool::Diamond => ShapeKind::Diamond,
                    Tool::Line => ShapeKind::Line,
                    Tool::Arrow => ShapeKind::Arrow,
                    _ => {
                        self.drag = Some(Drag::Create { start_x, start_y });
                        return;
                    }
                };

                // Update or create the preview element
                if let Some(last) = self.elements.last_mut() {
                    if last.kind == kind && last.id == self.next_id - 1 {
                        last.x = x;
                        last.y = y;
                        last.width = width;
                        last.height = height;
                    }
                } else {
                    let id = self.next_id;
                    self.next_id += 1;
                    let points = match kind {
                        ShapeKind::Line | ShapeKind::Arrow => {
                            vec![(0.0, 0.0), (width, height)]
                        }
                        _ => Vec::new(),
                    };
                    self.elements.push(ExcalidrawElement {
                        id,
                        kind,
                        x,
                        y,
                        width,
                        height,
                        text: String::new(),
                        points,
                        stroke: self.stroke_color,
                        fill: self.fill_color,
                        stroke_width: self.stroke_width,
                        stroke_style: self.stroke_style,
                    });
                }

                self.drag = Some(Drag::Create { start_x, start_y });
                cx.notify();
            }
            Drag::Move {
                index,
                offset_x,
                offset_y,
            } => {
                if let Some(elem) = self.elements.get_mut(index) {
                    elem.x = canvas_x - offset_x;
                    elem.y = canvas_y - offset_y;
                }
                self.drag = Some(Drag::Move {
                    index,
                    offset_x,
                    offset_y,
                });
                cx.notify();
            }
            Drag::Freedraw { index } => {
                if let Some(elem) = self.elements.get_mut(index) {
                    let rel_x = canvas_x - elem.x;
                    let rel_y = canvas_y - elem.y;
                    elem.points.push((rel_x, rel_y));
                    let min_x = elem
                        .points
                        .iter()
                        .map(|p| p.0)
                        .fold(f32::INFINITY, f32::min);
                    let min_y = elem
                        .points
                        .iter()
                        .map(|p| p.1)
                        .fold(f32::INFINITY, f32::min);
                    let max_x = elem
                        .points
                        .iter()
                        .map(|p| p.0)
                        .fold(f32::NEG_INFINITY, f32::max);
                    let max_y = elem
                        .points
                        .iter()
                        .map(|p| p.1)
                        .fold(f32::NEG_INFINITY, f32::max);
                    elem.x += min_x;
                    elem.y += min_y;
                    elem.width = max_x - min_x;
                    elem.height = max_y - min_y;
                    elem.points = elem
                        .points
                        .iter()
                        .map(|(px, py)| (*px - min_x, *py - min_y))
                        .collect();
                }
                self.drag = Some(Drag::Freedraw { index });
                cx.notify();
            }
            Drag::Pan {
                start_mouse_x,
                start_mouse_y,
                origin_vx,
                origin_vy,
            } => {
                let cur_mouse_x = f32::from(event.position.x);
                let cur_mouse_y = f32::from(event.position.y);
                self.viewport_x = origin_vx + (cur_mouse_x - start_mouse_x);
                self.viewport_y = origin_vy + (cur_mouse_y - start_mouse_y);
                self.drag = Some(Drag::Pan {
                    start_mouse_x,
                    start_mouse_y,
                    origin_vx,
                    origin_vy,
                });
                cx.notify();
            }
        }
    }

    fn canvas_mouse_up(
        &mut self,
        _event: &MouseUpEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.drag.is_some() {
            self.drag = None;
            cx.notify();
        }
    }

    fn canvas_scroll_wheel(
        &mut self,
        event: &ScrollWheelEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if event.modifiers.control || event.modifiers.platform {
            let delta = event.delta.pixel_delta(px(20.0));
            let zoom_delta = -f32::from(delta.y) / 200.0;
            let old_zoom = self.zoom;
            self.zoom = (self.zoom + zoom_delta).clamp(0.1, 30.0);
            if (self.zoom - old_zoom).abs() > 0.001 {
                // 以鼠标位置为中心缩放：调整 viewport 使鼠标下的画布坐标不变
                let mouse_x = f32::from(event.position.x - self.canvas_origin.get().x);
                let mouse_y = f32::from(event.position.y - self.canvas_origin.get().y);
                let ratio = self.zoom / old_zoom;
                self.viewport_x = mouse_x - ratio * (mouse_x - self.viewport_x);
                self.viewport_y = mouse_y - ratio * (mouse_y - self.viewport_y);
                self.status = format!("缩放：{:.0}%", self.zoom * 100.0);
                cx.notify();
            }
        }
    }

    fn export_svg(&mut self, cx: &mut Context<Self>) {
        let mut svg = String::from(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"800\" height=\"600\" viewBox=\"0 0 800 600\">\n",
        );
        svg.push_str("<rect width=\"800\" height=\"600\" fill=\"white\"/>\n");
        for elem in &self.elements {
            svg.push_str(&element_to_svg(elem));
        }
        svg.push_str("</svg>");
        cx.write_to_clipboard(ClipboardItem::new_string(svg));
        self.status = format!("已导出 {} 个元素的 SVG 到剪贴板。", self.elements.len());
        cx.notify();
    }
}

impl Render for ExcalidrawView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // 匹配 Tauri Excalidraw.vue：全屏画布 + 左侧垂直工具栏
        div()
            .size_full()
            .relative()
            // 画布填满全屏
            .child(canvas_container(self, cx))
            // 左侧：垂直工具栏 + 选中时下方的样式面板
            .child(
                div()
                    .absolute()
                    .top(px(8.0))
                    .left(px(8.0))
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(toolbar(self, cx))
                    .when(self.selection.is_some(), |el| {
                        el.child(style_panel(self, cx))
                    }),
            )
            // 顶部水平操作栏（撤销/重做/删除/清空/导出/文档管理）
            .child(top_toolbar(self, cx))
            // 底部状态栏：缩放百分比 + 状态信息（对齐 Excalidraw 官方左下角 zoom/坐标）
            .child(
                div()
                    .absolute()
                    .bottom(px(8.0))
                    .left(px(8.0))
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_2()
                    .py_1()
                    .rounded_md()
                    .bg(cx.theme().background)
                    .shadow_sm()
                    .border_1()
                    .border_color(cx.theme().border)
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!("缩放: {:.0}%", self.zoom * 100.0))
                    .when(!self.status.is_empty(), |el| {
                        el.child(format!("| {}", self.status))
                    }),
            )
    }
}

fn top_toolbar(this: &ExcalidrawView, cx: &mut Context<ExcalidrawView>) -> Div {
    div()
        .absolute()
        .top(px(8.0))
        .left(px(64.0))
        .right(px(8.0))
        .flex()
        .items_center()
        .gap_1()
        .p_1()
        .rounded_md()
        .bg(cx.theme().background)
        .shadow_sm()
        .border_1()
        .border_color(cx.theme().border)
        .child(
            Button::new("undo")
                .small()
                .ghost()
                .icon(Icon::new(IconName::ArrowLeft))
                .tooltip("撤销")
                .on_click(cx.listener(|this, _, _, cx| {
                    this.undo(cx);
                })),
        )
        .child(
            Button::new("redo")
                .small()
                .ghost()
                .icon(Icon::new(IconName::ArrowRight))
                .tooltip("重做")
                .on_click(cx.listener(|this, _, _, cx| {
                    this.redo(cx);
                })),
        )
        .child(div().w(px(8.0)))
        .child(
            Button::new("delete-selected")
                .small()
                .ghost()
                .icon(Icon::new(IconName::Delete))
                .tooltip("删除选中")
                .on_click(cx.listener(|this, _, _, cx| {
                    this.delete_selected(cx);
                })),
        )
        .child(
            Button::new("clear-all")
                .small()
                .ghost()
                .icon(Icon::new(IconName::Close))
                .tooltip("清空画布")
                .on_click(cx.listener(|this, _, _, cx| {
                    this.clear(cx);
                })),
        )
        .child(div().w(px(8.0)))
        .child(
            Button::new("export-json")
                .small()
                .ghost()
                .icon(Icon::new(IconName::Copy))
                .tooltip("导出 JSON 到剪贴板")
                .on_click(cx.listener(|this, _, _, cx| {
                    this.export_json(cx);
                })),
        )
        .child(
            Button::new("export-svg")
                .small()
                .ghost()
                .icon(Icon::new(IconName::ExternalLink))
                .tooltip("导出 SVG 到剪贴板")
                .on_click(cx.listener(|this, _, _, cx| {
                    this.export_svg(cx);
                })),
        )
        .child(div().flex_1())
        .child(
            div()
                .w(px(160.0))
                .child(Input::new(&this.doc_name_state)),
        )
        .child(
            Button::new("save-doc")
                .small()
                .ghost()
                .icon(Icon::new(IconName::File))
                .tooltip("保存文档")
                .on_click(cx.listener(|this, _, _, cx| {
                    this.save_doc(cx);
                })),
        )
        .child(
            Button::new("load-doc")
                .small()
                .ghost()
                .icon(Icon::new(IconName::Search))
                .tooltip("加载文档")
                .on_click(cx.listener(|this, _, window, cx| {
                    this.load_docs(window, cx);
                })),
        )
}

fn toolbar(this: &ExcalidrawView, cx: &mut Context<ExcalidrawView>) -> Div {
    // 工具顺序对齐 Excalidraw 官方：Select→Rectangle→Diamond→Ellipse→Arrow→Line→Freedraw→Text→Eraser
    // 用现有图标近似映射
    let tools: [(Tool, Option<IconName>, &'static str); 9] = [
        (Tool::Select, Some(IconName::Map), "选择(V)"),
        (Tool::Rectangle, Some(IconName::Frame), "矩形(R)"),
        (Tool::Diamond, Some(IconName::Star), "菱形(D)"),
        (Tool::Ellipse, Some(IconName::CircleCheck), "椭圆(O)"),
        (Tool::Arrow, Some(IconName::ArrowUp), "箭头(A)"),
        (Tool::Line, Some(IconName::Minus), "直线(L)"),
        (Tool::Freedraw, Some(IconName::Dash), "手绘(P)"),
        (Tool::Text, Some(IconName::ALargeSmall), "文本(T)"),
        (Tool::Eraser, Some(IconName::Close), "橡皮(E)"),
    ];

    let mut group = div()
        .flex()
        .flex_col()
        .items_center()
        .gap_1()
        .p_1()
        .rounded_md()
        .bg(cx.theme().background)
        .shadow_lg()
        .border_1()
        .border_color(cx.theme().border);

    for (i, (tool, icon_opt, tooltip)) in tools.iter().enumerate() {
        let tool = *tool;
        let is_active = this.tool == tool;
        let mut btn = Button::new(("tool", i))
            .tooltip(*tooltip)
            .when(is_active, |btn| btn.selected(true))
            .on_click(cx.listener(move |this, _, _, cx| {
                this.set_tool(tool, cx);
            }));
        if let Some(icon) = icon_opt.as_ref() {
            btn = btn.icon(Icon::new(icon.clone()));
        } else {
            btn = btn.label(tool.label());
        }
        group = group.child(btn);
    }

    group
}

fn style_panel(this: &ExcalidrawView, cx: &mut Context<ExcalidrawView>) -> Div {
    // 垂直布局，对齐 Excalidraw 官方左侧属性面板风格
    let mut panel = div()
        .flex()
        .flex_col()
        .gap_2()
        .p_2()
        .w(px(180.0))
        .bg(cx.theme().background)
        .border_1()
        .border_color(cx.theme().border)
        .rounded_md()
        .shadow_sm();

    // Stroke color presets（4 列网格）
    panel = panel.child(div().text_xs().font_semibold().child("描边"));
    let stroke_color = this.stroke_color;
    let stroke_grid = color_presets()
        .iter()
        .enumerate()
        .fold(div().flex().flex_wrap().gap_1(), |acc, (i, preset)| {
            let arr = preset.arr;
            let is_active = stroke_color == arr;
            acc.child(
                div()
                    .id(("stroke-color", i))
                    .w(px(24.0))
                    .h(px(24.0))
                    .rounded_md()
                    .bg(preset.hsla)
                    .border_1()
                    .border_color(if is_active {
                        cx.theme().primary
                    } else {
                        cx.theme().border
                    })
                    .when(preset.label == "透明", |d| {
                        d.child(div().text_xs().text_center().child("∅"))
                    })
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            this.set_stroke_color(arr, cx);
                        }),
                    ),
            )
        });
    panel = panel.child(stroke_grid);

    // Fill color presets（4 列网格）
    panel = panel.child(div().text_xs().font_semibold().child("填充"));
    let fill_color = this.fill_color;
    let fill_grid = color_presets()
        .iter()
        .enumerate()
        .fold(div().flex().flex_wrap().gap_1(), |acc, (i, preset)| {
            let arr = preset.arr;
            let is_active = fill_color == arr;
            acc.child(
                div()
                    .id(("fill-color", i))
                    .w(px(24.0))
                    .h(px(24.0))
                    .rounded_md()
                    .bg(preset.hsla)
                    .border_1()
                    .border_color(if is_active {
                        cx.theme().primary
                    } else {
                        cx.theme().border
                    })
                    .when(preset.label == "透明", |d| {
                        d.child(div().text_xs().text_center().child("∅"))
                    })
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            this.set_fill_color(arr, cx);
                        }),
                    ),
            )
        });
    panel = panel.child(fill_grid);

    // Stroke width（水平 3 个按钮）
    panel = panel.child(div().text_xs().font_semibold().child("线宽"));
    let width_row = [1.0f32, 2.0, 4.0]
        .iter()
        .enumerate()
        .fold(div().flex().gap_1(), |acc, (i, width)| {
            let is_active = (this.stroke_width - width).abs() < 0.01;
            let w = *width;
            acc.child(
                Button::new(("stroke-width", i))
                    .small()
                    .label(format!("{w}"))
                    .tooltip(format!("线宽 {w}"))
                    .when(is_active, |btn| btn.primary())
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.set_stroke_width(w, cx);
                    })),
            )
        });
    panel = panel.child(width_row);

    // Stroke style（水平 2 个按钮）
    panel = panel.child(div().text_xs().font_semibold().child("样式"));
    panel = panel.child(
        div()
            .flex()
            .gap_1()
            .child(
                Button::new("style-solid")
                    .small()
                    .label("实线")
                    .when(this.stroke_style == StrokeStyleKind::Solid, |btn| {
                        btn.primary()
                    })
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.set_stroke_style(StrokeStyleKind::Solid, cx);
                    })),
            )
            .child(
                Button::new("style-dashed")
                    .small()
                    .label("虚线")
                    .when(this.stroke_style == StrokeStyleKind::Dashed, |btn| {
                        btn.primary()
                    })
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.set_stroke_style(StrokeStyleKind::Dashed, cx);
                    })),
            ),
    );

    // Text editor for selected text element
    if let Some(index) = this.selection {
        if let Some(elem) = this.elements.get(index) {
            if elem.kind == ShapeKind::Text {
                panel = panel.child(div().text_xs().font_semibold().child("文本"));
                panel = panel.child(Input::new(&this.text_edit_state));
                panel = panel.child(
                    Button::new("update-text")
                        .small()
                        .primary()
                        .label("更新文本")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.update_selected_text(cx);
                        })),
                );
            }
        }
    }

    panel
}

fn canvas_container(this: &ExcalidrawView, cx: &mut Context<ExcalidrawView>) -> Div {
    let elements = this.elements.clone();
    let viewport_x = this.viewport_x;
    let viewport_y = this.viewport_y;
    let zoom = this.zoom;
    let selection = this.selection;
    let canvas_origin = this.canvas_origin.clone();
    let border_color = cx.theme().border;
    let bg_color = gpui::white();

    let text_overlays: Vec<(f32, f32, f32, f32, String, [f32; 4])> = elements
        .iter()
        .filter(|e| e.kind == ShapeKind::Text)
        .map(|e| {
            (
                (e.x * zoom) + viewport_x,
                (e.y * zoom) + viewport_y,
                e.width * zoom,
                e.height * zoom,
                e.text.clone(),
                e.stroke,
            )
        })
        .collect();

    let selection_bounds: Option<(f32, f32, f32, f32)> = selection.and_then(|index| {
        elements.get(index).map(|e| {
            (
                (e.x * zoom) + viewport_x,
                (e.y * zoom) + viewport_y,
                e.width * zoom,
                e.height * zoom,
            )
        })
    });

    div()
        .relative()
        .size_full()
        .border_1()
        .border_color(border_color)
        .rounded_lg()
        .overflow_hidden()
        .child(
            canvas(
                move |_, _, _| elements.clone(),
                move |bounds, elems, window, _| {
                    canvas_origin.set(bounds.origin);
                    window.paint_quad(fill(bounds, bg_color));
                    // 点阵网格（Excalidraw 风格）
                    let dot_color = hsla(0.0, 0.0, 0.85, 1.0);
                    let dot_size = 1.5_f32;
                    let step = 20.0_f32 * zoom;
                    let start_x = viewport_x.rem_euclid(step);
                    let start_y = viewport_y.rem_euclid(step);
                    let mut x = start_x;
                    while x < f32::from(bounds.size.width) {
                        let mut y = start_y;
                        while y < f32::from(bounds.size.height) {
                            let dot_bounds = Bounds::new(
                                bounds.origin + point(px(x - dot_size / 2.0), px(y - dot_size / 2.0)),
                                size(px(dot_size), px(dot_size)),
                            );
                            window.paint_quad(fill(dot_bounds, dot_color));
                            y += step;
                        }
                        x += step;
                    }

                    for elem in &elems {
                        paint_element(bounds.origin, elem, viewport_x, viewport_y, zoom, window);
                    }

                    // Selection highlight
                    if let Some((sx, sy, sw, sh)) = selection_bounds {
                        let sel_bounds = Bounds::new(
                            bounds.origin + point(px(sx - 2.0), px(sy - 2.0)),
                            size(px(sw + 4.0), px(sh + 4.0)),
                        );
                        window.paint_quad(
                            fill(sel_bounds, hsla(0.0, 0.0, 0.0, 0.0))
                                .border_widths(1.0)
                                .border_color(hsla(0.6, 0.8, 0.55, 1.0)),
                        );
                    }
                },
            )
            .size_full(),
        )
        .children(
            text_overlays
                .into_iter()
                .enumerate()
                .map(|(_i, (x, y, _w, _h, text, color))| {
                    let hsla_color = hsla(color[0], color[1], color[2], color[3]);
                    div()
                        .absolute()
                        .left(px(x + 4.0))
                        .top(px(y + 4.0))
                        .text_sm()
                        .text_color(hsla_color)
                        .child(text)
                        // 不设置 .id()，避免成为交互元素拦截画布鼠标事件
                        // 也不绑定任何 on_mouse_down，让事件穿透到画布
                }),
        )
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(|this, event, window, cx| {
                this.canvas_mouse_down(event, window, cx);
            }),
        )
        .on_mouse_down(
            MouseButton::Middle,
            cx.listener(|this, event, window, cx| {
                this.canvas_mouse_down(event, window, cx);
            }),
        )
        .on_mouse_move(cx.listener(|this, event, window, cx| {
            this.canvas_mouse_move(event, window, cx);
        }))
        .on_mouse_up(
            MouseButton::Left,
            cx.listener(|this, event, window, cx| {
                this.canvas_mouse_up(event, window, cx);
            }),
        )
        .on_mouse_up(
            MouseButton::Middle,
            cx.listener(|this, event, window, cx| {
                this.canvas_mouse_up(event, window, cx);
            }),
        )
        .on_scroll_wheel(cx.listener(|this, event, window, cx| {
            this.canvas_scroll_wheel(event, window, cx);
        }))
        .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
            let m = &event.keystroke.modifiers;
            let key = event.keystroke.key.as_str();
            match key {
                // Ctrl+Z 撤销
                "z" if m.control && !m.shift => {
                    this.undo(cx);
                }
                // Ctrl+Shift+Z 重做
                "z" if m.control && m.shift => {
                    this.redo(cx);
                }
                // Ctrl+Y 重做
                "y" if m.control => {
                    this.redo(cx);
                }
                // Delete / Backspace 删除选中
                "delete" | "backspace" => {
                    this.delete_selected(cx);
                }
                // Escape 取消选择
                "escape" => {
                    this.selection = None;
                    cx.notify();
                }
                // 工具快捷键（仅在无修饰键时生效）
                _ if !m.control && !m.alt && !m.platform => {
                    match key {
                        "v" => this.set_tool(Tool::Select, cx),
                        "r" => this.set_tool(Tool::Rectangle, cx),
                        "o" => this.set_tool(Tool::Ellipse, cx),
                        "d" => this.set_tool(Tool::Diamond, cx),
                        "l" => this.set_tool(Tool::Line, cx),
                        "a" => this.set_tool(Tool::Arrow, cx),
                        "t" => this.set_tool(Tool::Text, cx),
                        "p" => this.set_tool(Tool::Freedraw, cx),
                        "e" => this.set_tool(Tool::Eraser, cx),
                        _ => {}
                    }
                }
                _ => {}
            }
        }))
}

fn paint_element(
    origin: Point<Pixels>,
    elem: &ExcalidrawElement,
    viewport_x: f32,
    viewport_y: f32,
    zoom: f32,
    window: &mut Window,
) {
    let screen_x = elem.x * zoom + viewport_x;
    let screen_y = elem.y * zoom + viewport_y;
    let abs_origin = origin + point(px(screen_x), px(screen_y));
    let stroke = hsla(
        elem.stroke[0],
        elem.stroke[1],
        elem.stroke[2],
        elem.stroke[3],
    );
    let fill = hsla(elem.fill[0], elem.fill[1], elem.fill[2], elem.fill[3]);
    let width = px(elem.stroke_width * zoom);

    let dash = match elem.stroke_style {
        StrokeStyleKind::Dashed => Some([px(6.0 * zoom), px(4.0 * zoom)]),
        StrokeStyleKind::Solid => None,
    };

    let zoomed_width = elem.width * zoom;
    let zoomed_height = elem.height * zoom;

    match elem.kind {
        ShapeKind::Rectangle => {
            let bounds = Bounds::new(abs_origin, size(px(zoomed_width), px(zoomed_height)));
            window.paint_quad(
                fill_quad(bounds, fill)
                    .border_widths(elem.stroke_width * zoom)
                    .border_color(stroke),
            );
        }
        ShapeKind::Ellipse => {
            let bounds = Bounds::new(abs_origin, size(px(zoomed_width), px(zoomed_height)));
            window.paint_quad(
                fill_quad(bounds, fill)
                    .corner_radii(999.0)
                    .border_widths(elem.stroke_width * zoom)
                    .border_color(stroke),
            );
        }
        ShapeKind::Diamond => {
            let cx = screen_x + zoomed_width / 2.0;
            let cy = screen_y + zoomed_height / 2.0;
            let p1 = origin + point(px(cx), px(screen_y));
            let p2 = origin + point(px(screen_x + zoomed_width), px(cy));
            let p3 = origin + point(px(cx), px(screen_y + zoomed_height));
            let p4 = origin + point(px(screen_x), px(cy));
            if elem.fill[3] > 0.0 {
                let mut path = PathBuilder::fill();
                path.move_to(p1);
                path.line_to(p2);
                path.line_to(p3);
                path.line_to(p4);
                path.line_to(p1);
                if let Ok(p) = path.build() {
                    window.paint_path(p, fill);
                }
            }
            let mut path = PathBuilder::stroke(width);
            if let Some(d) = dash {
                path = path.dash_array(&d);
            }
            path.move_to(p1);
            path.line_to(p2);
            path.line_to(p3);
            path.line_to(p4);
            path.line_to(p1);
            if let Ok(p) = path.build() {
                window.paint_path(p, stroke);
            }
        }
        ShapeKind::Line | ShapeKind::Arrow => {
            let start = abs_origin;
            let end = origin + point(px(screen_x + zoomed_width), px(screen_y + zoomed_height));
            let mut path = PathBuilder::stroke(width);
            if let Some(d) = dash {
                path = path.dash_array(&d);
            }
            path.move_to(start);
            path.line_to(end);
            if elem.kind == ShapeKind::Arrow {
                let dx = f32::from(end.x - start.x);
                let dy = f32::from(end.y - start.y);
                let len = (dx * dx + dy * dy).sqrt().max(0.001);
                let ux = dx / len;
                let uy = dy / len;
                let arrow_len = 12.0_f32 * zoom;
                let arrow_angle = 0.5f32;
                let cos_a = arrow_angle.cos();
                let sin_a = arrow_angle.sin();
                let end_x = f32::from(end.x);
                let end_y = f32::from(end.y);
                let ax1 = end_x - arrow_len * (ux * cos_a + uy * sin_a);
                let ay1 = end_y - arrow_len * (uy * cos_a - ux * sin_a);
                let ax2 = end_x - arrow_len * (ux * cos_a - uy * sin_a);
                let ay2 = end_y - arrow_len * (uy * cos_a + ux * sin_a);
                path.move_to(end);
                path.line_to(point(px(ax1), px(ay1)));
                path.move_to(end);
                path.line_to(point(px(ax2), px(ay2)));
            }
            if let Ok(p) = path.build() {
                window.paint_path(p, stroke);
            }
        }
        ShapeKind::Text => {
            // Background quad for text
            let bounds = Bounds::new(abs_origin, size(px(zoomed_width), px(zoomed_height)));
            if elem.fill[3] > 0.0 {
                window.paint_quad(fill_quad(bounds, fill));
            }
            // Text is rendered as overlay div
        }
        ShapeKind::Freedraw => {
            if elem.points.len() < 2 {
                return;
            }
            let mut path = PathBuilder::stroke(width);
            if let Some(d) = dash {
                path = path.dash_array(&d);
            }
            let first = elem.points[0];
            path.move_to(abs_origin + point(px(first.0 * zoom), px(first.1 * zoom)));
            for pt in &elem.points[1..] {
                path.line_to(abs_origin + point(px(pt.0 * zoom), px(pt.1 * zoom)));
            }
            if let Ok(p) = path.build() {
                window.paint_path(p, stroke);
            }
        }
    }
}

fn element_to_svg(elem: &ExcalidrawElement) -> String {
    let stroke = format!(
        "rgba({},{},{},{})",
        (elem.stroke[0] * 255.0) as u8,
        (elem.stroke[1] * 255.0) as u8,
        (elem.stroke[2] * 255.0) as u8,
        elem.stroke[3]
    );
    let fill = if elem.fill[3] > 0.0 {
        format!(
            "rgba({},{},{},{})",
            (elem.fill[0] * 255.0) as u8,
            (elem.fill[1] * 255.0) as u8,
            (elem.fill[2] * 255.0) as u8,
            elem.fill[3]
        )
    } else {
        "none".to_string()
    };
    let dash_attr = match elem.stroke_style {
        StrokeStyleKind::Dashed => format!(" stroke-dasharray=\"6,4\""),
        StrokeStyleKind::Solid => String::new(),
    };
    match elem.kind {
        ShapeKind::Rectangle => {
            format!(
                "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" stroke=\"{}\" fill=\"{}\" stroke-width=\"{}\"{dash_attr}/>\n",
                elem.x, elem.y, elem.width, elem.height, stroke, fill, elem.stroke_width
            )
        }
        ShapeKind::Ellipse => {
            let cx = elem.x + elem.width / 2.0;
            let cy = elem.y + elem.height / 2.0;
            format!(
                "<ellipse cx=\"{}\" cy=\"{}\" rx=\"{}\" ry=\"{}\" stroke=\"{}\" fill=\"{}\" stroke-width=\"{}\"{dash_attr}/>\n",
                cx,
                cy,
                elem.width / 2.0,
                elem.height / 2.0,
                stroke,
                fill,
                elem.stroke_width
            )
        }
        ShapeKind::Diamond => {
            let cx = elem.x + elem.width / 2.0;
            let cy = elem.y + elem.height / 2.0;
            format!(
                "<polygon points=\"{},{} {},{} {},{} {},{}\" stroke=\"{}\" fill=\"{}\" stroke-width=\"{}\"{dash_attr}/>\n",
                cx,
                elem.y,
                elem.x + elem.width,
                cy,
                cx,
                elem.y + elem.height,
                elem.x,
                cy,
                stroke,
                fill,
                elem.stroke_width
            )
        }
        ShapeKind::Line => {
            format!(
                "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"{}\"{dash_attr}/>\n",
                elem.x,
                elem.y,
                elem.x + elem.width,
                elem.y + elem.height,
                stroke,
                elem.stroke_width
            )
        }
        ShapeKind::Arrow => {
            let x2 = elem.x + elem.width;
            let y2 = elem.y + elem.height;
            let dx = elem.width;
            let dy = elem.height;
            let len = (dx * dx + dy * dy).sqrt().max(0.001);
            let ux = dx / len;
            let uy = dy / len;
            let arrow_len = 12.0_f32;
            let arrow_angle = 0.5f32;
            let cos_a = arrow_angle.cos();
            let sin_a = arrow_angle.sin();
            let ax1 = x2 - arrow_len * (ux * cos_a + uy * sin_a);
            let ay1 = y2 - arrow_len * (uy * cos_a - ux * sin_a);
            let ax2 = x2 - arrow_len * (ux * cos_a - uy * sin_a);
            let ay2 = y2 - arrow_len * (uy * cos_a + ux * sin_a);
            format!(
                "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"{}\"{dash_attr}/>\n<polyline points=\"{},{} {},{} {},{}\" stroke=\"{}\" stroke-width=\"{}\" fill=\"none\"/>\n",
                elem.x,
                elem.y,
                x2,
                y2,
                stroke,
                elem.stroke_width,
                ax1,
                ay1,
                x2,
                y2,
                ax2,
                ay2,
                stroke,
                elem.stroke_width
            )
        }
        ShapeKind::Text => {
            format!(
                "<text x=\"{}\" y=\"{}\" fill=\"{}\" font-size=\"{TEXT_FONT_SIZE}\">{}</text>\n",
                elem.x,
                elem.y + TEXT_FONT_SIZE,
                stroke,
                elem.text
            )
        }
        ShapeKind::Freedraw => {
            if elem.points.len() < 2 {
                return String::new();
            }
            let mut d = format!("M{} {}", elem.points[0].0, elem.points[0].1);
            for pt in &elem.points[1..] {
                d.push_str(&format!(" L{} {}", pt.0, pt.1));
            }
            format!(
                "<path d=\"{d}\" stroke=\"{}\" fill=\"none\" stroke-width=\"{}\"{dash_attr}/>\n",
                stroke, elem.stroke_width
            )
        }
    }
}

fn hit_test(elements: &[ExcalidrawElement], x: f32, y: f32) -> Option<usize> {
    // Iterate in reverse to hit top-most element first
    for (i, elem) in elements.iter().enumerate().rev() {
        if x >= elem.x && x <= elem.x + elem.width && y >= elem.y && y <= elem.y + elem.height {
            return Some(i);
        }
    }
    None
}

fn fill_quad(bounds: Bounds<Pixels>, color: Hsla) -> PaintQuad {
    fill(bounds, color)
}

fn doc_load_dialog_content(
    docs: Vec<config_store::ExcalidrawDocRecord>,
    target: WeakEntity<ExcalidrawView>,
    cx: &mut App,
) -> Div {
    let mut content = div().flex().flex_col().gap_3();

    if docs.is_empty() {
        return content.child(div().text_sm().child("暂无已保存文档，请先保存文档。"));
    }

    for (index, doc) in docs.into_iter().enumerate() {
        let name = doc.name.clone();
        let elements_json = doc.elements_json.clone();
        let updated_at = doc.updated_at;
        let target_apply = target.clone();
        let target_delete = target.clone();
        let name_apply = name.clone();
        let name_delete = name.clone();
        content = content.child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .gap_3()
                .border_1()
                .border_color(cx.theme().border)
                .rounded_md()
                .p_2()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(div().text_sm().font_semibold().child(name.clone()))
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(format!("更新时间：{updated_at}")),
                        ),
                )
                .child(
                    ButtonGroup::new(("doc-actions", index))
                        .child(
                            Button::new(("apply-doc", index))
                                .icon(Icon::new(IconName::ArrowRight))
                                .tooltip("加载文档")
                                .on_click(move |_, window, cx| {
                                    let target = target_apply.clone();
                                    let elements_json = elements_json.clone();
                                    let name = name_apply.clone();
                                    let window_handle = window.window_handle();
                                    let _ = window_handle.update(cx, move |_, window, cx| {
                                        if let Some(target) = target.upgrade() {
                                            let _ = target.update(cx, |this, cx| {
                                                this.apply_doc(
                                                    elements_json.clone(),
                                                    name.clone(),
                                                    window,
                                                    cx,
                                                );
                                            });
                                        }
                                    });
                                    window.close_dialog(cx);
                                }),
                        )
                        .child(
                            Button::new(("delete-doc", index))
                                .icon(Icon::new(IconName::Delete))
                                .tooltip("删除文档")
                                .on_click(move |_, window, cx| {
                                    let target = target_delete.clone();
                                    let name = name_delete.clone();
                                    let window_handle = window.window_handle();
                                    let _ = window_handle.update(cx, move |_, _window, cx| {
                                        if let Some(target) = target.upgrade() {
                                            let _ = target.update(cx, |this, cx| {
                                                this.delete_doc(name.clone(), cx);
                                            });
                                        }
                                    });
                                    window.close_dialog(cx);
                                }),
                        ),
                ),
        );
    }

    content
}
