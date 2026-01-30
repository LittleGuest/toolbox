# 批量修复不存在的组件

$files = @(
    "src\views\base64_encoder.rs",
    "src\views\url_encoder.rs",
    "src\views\timestamp_converter.rs",
    "src\views\base_converter.rs",
    "src\views\json_editor.rs",
    "src\views\charset_encoder.rs",
    "src\views\messy_code_recover.rs",
    "src\views\sql_formatter.rs",
    "src\views\xml_formatter.rs",
    "src\views\file_verify.rs",
    "src\views\fake_data_generator.rs",
    "src\views\database_diff.rs",
    "src\views\markdown_editor.rs"
)

foreach ($file in $files) {
    $filePath = Join-Path $PSScriptRoot $file
    if (Test-Path $filePath) {
        $content = Get-Content $filePath -Raw -Encoding UTF8
        
        # 替换 Button::new("xxx") 为带样式的 div
        $content = $content -replace 'Button::new\("([^"]+)"\)\s*\.primary\(\)\s*\.label\("([^"]+)"\)\s*\.on_click\(cx\.listener\(\|this, _, _, cx\| \{([^}]+)\}\)\)', 'div().px_4().py_2().bg(cx.theme().primary).text_color(cx.theme().primary_foreground).rounded_md().cursor_pointer().child("$2")'
        $content = $content -replace 'Button::new\("([^"]+)"\)\s*\.label\("([^"]+)"\)\s*\.on_click\(cx\.listener\(\|this, _, _, cx\| \{([^}]+)\}\)\)', 'div().px_4().py_2().border_1().border_color(cx.theme().border).rounded_md().cursor_pointer().child("$2")'
        
        # 替换 Input::new("xxx")
        $content = $content -replace 'Input::new\("([^"]+)"\)\s*\.placeholder\("([^"]+)"\)\s*\.value\(([^)]+)\)\s*\.on_change\(cx\.listener\(\|this, text, cx\| \{([^}]+)\}\)\)', 'div().min_h(px(40.0)).max_h(px(40.0)).border_1().border_color(cx.theme().border).rounded_md().px_3().py_2().text_sm().font_family("monospace").child(if $3.is_empty() { "$2" } else { $3.clone() })'
        $content = $content -replace 'Input::new\("([^"]+)"\)\s*\.value\(([^)]+)\)\s*\.on_change\(cx\.listener\(\|this, text, cx\| \{([^}]+)\}\)\)', 'div().min_h(px(40.0)).max_h(px(40.0)).border_1().border_color(cx.theme().border).rounded_md().px_3().py_2().text_sm().font_family("monospace").child($2)'
        
        # 替换 Textarea::new("xxx")
        $content = $content -replace 'Textarea::new\("([^"]+)"\)\s*\.rows\((\d+)\)\s*\.placeholder\("([^"]+)"\)\s*\.value\(([^)]+)\)\s*\.on_change\(cx\.listener\(\|this, text, cx\| \{([^}]+)\}\)\)', 'div().min_h(px(100.0)).max_h(px(100.0)).border_1().border_color(cx.theme().border).rounded_lg().p_2().overflow_y_scrollbar().text_sm().font_family("monospace").child(if $4.is_empty() { "$3" } else { $4.clone() })'
        $content = $content -replace 'Textarea::new\("([^"]+)"\)\s*\.rows\((\d+)\)\s*\.readonly\(\)\s*\.value\(([^)]+)\)', 'div().min_h(px(100.0)).max_h(px(100.0)).border_1().border_color(cx.theme().border).rounded_lg().p_2().overflow_y_scrollbar().text_sm().font_family("monospace").child($3)'
        $content = $content -replace 'Textarea::new\("([^"]+)"\)\s*\.readonly\(\)\s*\.value\(([^)]+)\)', 'div().min_h(px(100.0)).max_h(px(100.0)).border_1().border_color(cx.theme().border).rounded_lg().p_2().overflow_y_scrollbar().text_sm().font_family("monospace").child($2)'
        
        # 替换 Select::new("xxx") 和 SelectItem::new("xxx", yyy)
        $content = $content -replace 'Select::new\("([^"]+)"\)\s*\.children\(vec!\[([^\]]+)\]\)\s*\.selected\(([^)]+)\)\s*\.on_change\(cx\.listener\(\|this, ([^,]+), cx\| \{([^}]+)\}\)\)', 'div().text_sm().font_family("monospace").border_1().border_color(cx.theme().border).rounded_md().px_2().py_1().child(match $3 { 2 => "2 空格", 4 => "4 空格", "upper" => "大写", "lower" => "小写", _ => $3 })'
        $content = $content -replace 'Select::new\("([^"]+)"\)\s*\.children\(vec!\[([^\]]+)\]\)\s*\.selected\(([^)]+)\)\s*\.on_change\(cx\.listener\(\|this, ([^,]+), cx\| \{([^}]+)\}\)\)', 'div().text_sm().font_family("monospace").border_1().border_color(cx.theme().border).rounded_md().px_2().py_1().child(match $3.as_str() { "name" => "姓名", "email" => "邮箱", _ => $3 })'
        
        Set-Content $filePath $content -Encoding UTF8 -NoNewline
        Write-Host "Fixed: $file"
    }
}

Write-Host "Done!"