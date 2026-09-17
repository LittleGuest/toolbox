use anyhow::{Error, Result};
use lopdf::{
    content::{Content, Operation},
    dictionary, Document, Object, ObjectId, Stream,
};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

type Id = (u32, u16);

/// 图片 -> PDF：每张图片生成一个 A4 宽（595pt）自适应高度页面。
pub fn images_to_pdf(inputs: &[String], output_path: &str) -> Result<()> {
    if inputs.is_empty() {
        return Err(Error::msg("请至少选择一张图片"));
    }
    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();
    let mut kids: Vec<Object> = Vec::new();

    for input in inputs {
        let img = image::open(input)
            .map_err(|e| Error::msg(format!("无法读取 {}: {e}", image_desc(input))))?;
        let rgb = img.to_rgb8();
        let (w, h) = rgb.dimensions();
        let (w, h) = (w.max(1) as f32, h.max(1) as f32);
        let page_w = 595.0;
        let page_h = page_w * (h / w);

        let img_stream = Stream::new(
            dictionary! {
                "Type" => "XObject",
                "Subtype" => "Image",
                "Width" => w as u32,
                "Height" => h as u32,
                "ColorSpace" => "DeviceRGB",
                "BitsPerComponent" => 8,
            },
            rgb.into_raw(),
        );

        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "MediaBox" => vec![0.0.into(), 0.0.into(), page_w.into(), page_h.into()],
        });
        doc.insert_image(page_id, img_stream, (0.0, 0.0), (page_w, page_h))?;
        kids.push(Object::Reference(page_id));
    }

    doc.objects.insert(
        pages_id,
        Object::Dictionary(dictionary! {
            "Type" => "Pages",
            "Kids" => kids,
            "Count" => inputs.len() as u32,
        }),
    );
    let catalog_id = doc.add_object(dictionary! {"Type" => "Catalog", "Pages" => pages_id});
    doc.trailer.set("Root", catalog_id);
    doc.save(output_path)?;
    Ok(())
}

/// 合并多个 PDF，页面按输入顺序排列。
pub fn pdf_merge(inputs: &[String], output_path: &str) -> Result<()> {
    if inputs.is_empty() {
        return Err(Error::msg("请至少选择一个 PDF"));
    }
    let mut specs = Vec::with_capacity(inputs.len());
    for p in inputs {
        let doc = Document::load(p).map_err(|e| Error::msg(format!("读取 {p} 失败: {e}")))?;
        let pages: Vec<u32> = (1..=doc.get_pages().len() as u32).collect();
        specs.push((doc, pages));
    }
    let mut merged = build_combined(specs)?;
    merged.save(output_path)?;
    Ok(())
}

/// PDF 页面编辑：删除（delete）、旋转（rotate: (页码, 角度)）、重排（order）。
/// order 为空表示按原页序；非空则为最终页序（1 基页码）。
pub fn pdf_edit(
    input: &str,
    output_path: &str,
    delete: &[u32],
    rotate: &[(u32, f32)],
    order: &[u32],
) -> Result<()> {
    let mut doc = Document::load(input).map_err(|e| Error::msg(format!("读取 {input} 失败: {e}")))?;
    let page_map = doc.get_pages();
    let page_count = page_map.len() as u32;

    // 应用旋转
    for (page_num, deg) in rotate {
        if let Some(page_id) = page_map.get(page_num) {
            let degrees = ((*deg as i64 % 360) + 360) % 360;
            if degrees != 0 {
                if let Ok(dict) = doc.get_object_mut(*page_id).and_then(Object::as_dict_mut) {
                    let cur = dict.get(b"Rotate").and_then(Object::as_i64).unwrap_or(0);
                    dict.set("Rotate", (cur + degrees) % 360);
                }
            }
        }
    }

    // 最终页序：order 提供则使用之，否则用原序；再剔除 delete
    let delete: std::collections::HashSet<u32> = delete.iter().copied().collect();
    let base: Vec<u32> = if order.is_empty() {
        (1..=page_count).collect()
    } else {
        order.to_vec()
    };
    let mut keep: Vec<u32> = base.into_iter().filter(|p| !delete.contains(p)).collect();
    // 去重并限制范围
    let mut seen = std::collections::HashSet::new();
    keep.retain(|p| *p >= 1 && *p <= page_count && seen.insert(*p));
    if keep.is_empty() {
        return Err(Error::msg("编辑后没有剩余页面"));
    }

    let mut merged = build_combined(vec![(doc, keep)])?;
    merged.save(output_path)?;
    Ok(())
}

/// 按页码范围拆分 PDF。ranges 形如 "1-3,5,7-9"，每个范围生成一个 PDF。
pub fn pdf_split(input: &str, output_dir: &str, ranges: &str) -> Result<Vec<String>> {
    let parsed = parse_ranges(ranges)?;
    let doc = Document::load(input).map_err(|e| Error::msg(format!("读取 {input} 失败: {e}")))?;
    let page_count = doc.get_pages().len() as u32;
    let dir = PathBuf::from(output_dir);
    std::fs::create_dir_all(&dir)?;
    let stem = Path::new(input)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("split");

    let mut outs = Vec::new();
    for (i, pages) in parsed.iter().enumerate() {
        if pages.is_empty() {
            continue;
        }
        if let Some(&max_p) = pages.iter().max() {
            if max_p > page_count {
                return Err(Error::msg(format!("页码 {max_p} 超出文档总页数 {page_count}")));
            }
        }
        let mut merged = build_combined(vec![(doc.clone(), pages.clone())])?;
        let out_path = dir.join(format!("{stem}_part{}.pdf", i + 1));
        merged.save(&out_path)?;
        outs.push(out_path.to_string_lossy().into_owned());
    }
    if outs.is_empty() {
        return Err(Error::msg("未产生任何输出文件"));
    }
    Ok(outs)
}

/// 为 PDF 添加页码。position: bottom|top + left|center|right。
/// format_pattern 中的 {n} 会被替换为实际页码；无 {n} 则在末尾追加页码。
pub fn pdf_add_page_numbers(
    input: &str,
    output_path: &str,
    position: &str,
    font_size: f32,
    format_pattern: &str,
    start_at: i32,
) -> Result<()> {
    let mut doc = Document::load(input).map_err(|e| Error::msg(format!("读取 {input} 失败: {e}")))?;
    let margin = font_size;

    let font_id = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Helvetica",
        "Encoding" => "WinAnsiEncoding",
    });
    let font_res = dictionary! {"Font" => dictionary! {"F1" => font_id}};

    let page_map = doc.get_pages();
    let total = page_map.len();
    for (i, page_id) in page_map.values().enumerate() {
        let page_no = start_at + i as i32;
        let text = build_number_text(format_pattern, page_no, (i + 1) as i32, total as i32);
        if text.is_empty() {
            continue;
        }
        let (page_w, page_h) = page_size(&doc, *page_id)?;
        let (x, y) = number_position(position, &text, font_size, margin, page_w, page_h);
        let fs = font_size;
        let content = Content {
            operations: vec![
                Operation::new("BT", vec![]),
                Operation::new("Tf", vec!["F1".into(), fs.into()]),
                Operation::new("Td", vec![x.into(), y.into()]),
                Operation::new("Tj", vec![Object::string_literal(text.as_str())]),
                Operation::new("ET", vec![]),
            ],
        };
        let mut form = lopdf::xobject::form(
            vec![0.0, 0.0, page_w, page_h],
            vec![1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            content.encode()?,
        );
        form.dict.set("Resources", font_res.clone());
        doc.insert_form_object(*page_id, form)?;
    }

    doc.save(output_path)?;
    Ok(())
}

// ---------- 内部工具 ----------

fn image_desc(input: &str) -> String {
    Path::new(input)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(input)
        .to_string()
}

/// 由若干 (Document, 需保留的页序) 构建合并/切片文档。
fn build_combined(specs: Vec<(Document, Vec<u32>)>) -> Result<Document> {
    let mut out = Document::with_version("1.5");
    let mut page_objects: BTreeMap<Id, Object> = BTreeMap::new();
    let mut other_objects: BTreeMap<Id, Object> = BTreeMap::new();
    let mut catalog: Option<(Id, Object)> = None;
    let mut keep_order: Vec<Id> = Vec::new();

    let mut max_id = 1u32;
    for (mut doc, keep) in specs {
        doc.renumber_objects_with(max_id);
        max_id = doc.max_id + 1;
        let page_map = doc.get_pages();
        for num in &keep {
            let page_id = *page_map
                .get(num)
                .ok_or_else(|| Error::msg(format!("页码 {num} 不存在")))?;
            keep_order.push(page_id);
        }
        for (oid, obj) in doc.objects.into_iter() {
            match obj.type_name().unwrap_or(b"") {
                b"Catalog" => {
                    if catalog.is_none() {
                        catalog = Some((oid, obj));
                    }
                }
                b"Pages" | b"Outlines" | b"Outline" => {}
                b"Page" => {
                    if keep_order.contains(&oid) {
                        page_objects.insert(oid, obj);
                    }
                }
                _ => {
                    other_objects.insert(oid, obj);
                }
            }
        }
    }

    // 新建 Pages 根节点
    let pages_id = out.new_object_id();
    let kids: Vec<Object> = keep_order.iter().map(|&id| Object::Reference(id)).collect();
    out.objects.insert(
        pages_id,
        Object::Dictionary(dictionary! {
            "Type" => "Pages",
            "Kids" => kids,
            "Count" => keep_order.len() as u32,
        }),
    );

    // 页面对象，更新 Parent
    for (page_id, obj) in page_objects {
        match obj.as_dict().cloned() {
            Ok(dict) => {
                let mut dict = dict;
                dict.set("Parent", pages_id);
                out.objects.insert(page_id, Object::Dictionary(dict));
            }
            Err(_) => {
                out.objects.insert(page_id, obj);
            }
        }
    }
    out.objects.extend(other_objects);

    // Catalog
    let root = match catalog {
        Some((cid, obj)) => {
            if let Ok(dict) = obj.as_dict().cloned() {
                let mut dict = dict;
                dict.set("Pages", pages_id);
                dict.remove(b"Outlines");
                out.objects.insert(cid, Object::Dictionary(dict));
                cid
            } else {
                new_catalog(&mut out, pages_id)
            }
        }
        None => new_catalog(&mut out, pages_id),
    };
    out.trailer.set("Root", root);

    out.max_id = out.objects.len() as u32;
    out.renumber_objects();
    Ok(out)
}

fn new_catalog(out: &mut Document, pages_id: Id) -> Id {
    out.add_object(dictionary! {"Type" => "Catalog", "Pages" => pages_id})
}

/// 读取页面尺寸（pt）。
fn page_size(doc: &Document, page_id: Id) -> Result<(f32, f32)> {
    let page = doc.get_dictionary(page_id)?;
    if let Ok(media_box) = page.get(b"MediaBox").and_then(Object::as_array) {
        if media_box.len() >= 4 {
            let nums: Vec<f32> = media_box
                .iter()
                .take(4)
                .map(|o| o.as_float().unwrap_or(0.0))
                .collect();
            let w = nums[2] - nums[0];
            let h = nums[3] - nums[1];
            if w > 0.0 && h > 0.0 {
                return Ok((w, h));
            }
        }
    }
    Ok((595.0, 842.0))
}

fn build_number_text(pattern: &str, page_no: i32, cur: i32, total: i32) -> String {
    if pattern.contains("{n}") {
        pattern
            .replace("{n}", &page_no.to_string())
            .replace("{cur}", &cur.to_string())
            .replace("{total}", &total.to_string())
    } else {
        format!("{pattern}{page_no}")
    }
}

/// 估算 Helvetica 文本宽度（用于居中对齐 / 右对齐）。
fn text_width(text: &str, font_size: f32) -> f32 {
    let units: f32 = text
        .chars()
        .map(|c| match c {
            ' ' | '.' | ',' | '-' | '/' | ':' => 0.28,
            '0'..='9' => 0.55,
            c if c.is_ascii_uppercase() => 0.67,
            c if c.is_ascii_lowercase() => 0.56,
            _ => 0.6,
        })
        .sum();
    font_size * units
}

fn number_position(position: &str, text: &str, fs: f32, margin: f32, w: f32, h: f32) -> (f32, f32) {
    let tw = text_width(text, fs);
    let vertical = if position.contains("top") {
        h - margin - fs
    } else {
        margin + fs * 0.2
    };
    let x = if position.contains("center") {
        (w - tw) * 0.5
    } else if position.contains("right") {
        w - margin - tw
    } else {
        margin
    };
    (x.max(0.0), vertical.max(fs * 0.5))
}

/// 解析 "1-3,5,7-9" 形式的范围。
fn parse_ranges(s: &str) -> Result<Vec<Vec<u32>>> {
    let mut ranges = Vec::new();
    for token in s.split([',', '，']).map(str::trim) {
        if token.is_empty() {
            continue;
        }
        let pages = if let Some(idx) = token.find('-') {
            let (a, b) = (&token[..idx], &token[idx + 1..]);
            if a.is_empty() || b.is_empty() {
                return Err(Error::msg(format!("无效的页码范围: {token}")));
            }
            let (a, b) = (a.trim().parse::<u32>()?, b.trim().parse::<u32>()?);
            let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
            (lo..=hi).collect::<Vec<u32>>()
        } else {
            vec![token.trim().parse::<u32>()?]
        };
        ranges.push(pages);
    }
    if ranges.is_empty() {
        return Err(Error::msg("请至少输入一个页码或范围"));
    }
    Ok(ranges)
}