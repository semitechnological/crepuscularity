const XML: &str = include_str!(concat!(env!("OUT_DIR"), "/stm32_dashboard.xml"));

#[test]
fn build_script_generates_stm32_lvgl_xml() {
    assert!(XML.contains(r#"<screen name="Stm32Dashboard">"#));
    assert!(XML.contains(r#"<lv_obj id="panel""#));
    assert!(XML.contains(
        r##"<lv_label text_color="#ffffff" text_font="montserrat_16" text="STM32F411 ILI9341"/>"##
    ));
    assert!(XML.contains(r#"<lv_bar id="cpu" value="72"/>"#));
    assert!(XML.contains(r##"<lv_label text_color="#22c55e" text="nominal"/>"##));
}
