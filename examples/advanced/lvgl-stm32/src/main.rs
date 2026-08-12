const STM32_DASHBOARD_XML: &str = include_str!(concat!(env!("OUT_DIR"), "/stm32_dashboard.xml"));

fn main() {
    println!("{STM32_DASHBOARD_XML}");
}
