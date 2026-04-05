use std::collections::HashMap;

use crepuscularity_core::TemplateContext;
use crepuscularity_web::render_from_files;

const SITE: &str = include_str!("../../../templates/site/site.crepus");
const SERVICES: &str = include_str!("../../../templates/site/elements/services.crepus");
const HERO: &str = include_str!("../../../templates/site/elements/hero.crepus");
const CONTACT: &str = include_str!("../../../templates/site/elements/contact-form.crepus");

#[test]
fn site_page_renders_service_item_title() {
    let mut files = HashMap::new();
    // Same virtual paths as unthought-sites `load_templates`.
    files.insert("site.crepus".to_string(), SITE.to_string());
    files.insert("elements/services.crepus".to_string(), SERVICES.to_string());
    files.insert("elements/hero.crepus".to_string(), HERO.to_string());
    files.insert(
        "elements/contact-form.crepus".to_string(),
        CONTACT.to_string(),
    );

    let mut hero = TemplateContext::new();
    hero.set("id", "h1");
    hero.set("type", "hero");
    hero.set("eyebrow", "Launch faster");
    hero.set("headline", "Welcome");
    hero.set("subheadline", "We make things");
    hero.set("primaryCtaLabel", "Go");
    hero.set("primaryCtaHref", "#");

    let mut svc1 = TemplateContext::new();
    svc1.set("title", "Launch setup");
    svc1.set("description", "Brand setup");
    svc1.set("icon", "Popular");
    let mut svc2 = TemplateContext::new();
    svc2.set("title", "Ongoing support");
    svc2.set("description", "Monthly");
    svc2.set("icon", "");

    let mut services_el = TemplateContext::new();
    services_el.set("id", "s1");
    services_el.set("type", "services");
    services_el.set("eyebrow", "What we do");
    services_el.set("title", "Services");
    services_el.set("description", "Choose a package that fits.");
    services_el.set("services", vec![svc1, svc2]);

    let mut root = TemplateContext::new();
    root.set("businessName", "TestCo");
    root.set("seoTitle", "T");
    root.set("seoDescription", "D");
    root.set("seoOgImage", "");
    root.set("themeAccent", "#3b82f6");
    root.set("themeAccentSoft", "#60a5fa");
    root.set("themeSurface", "#09090b");
    root.set("themeText", "#fafafa");
    root.set("themeMuted", "#a1a1aa");
    root.set("themeBorder", "#27272a");
    root.set("elements", vec![hero, services_el]);

    let html = render_from_files(&files, "site.crepus#Page", &root).expect("render");
    assert!(
        html.contains("Launch setup"),
        "missing service title; snippet={}",
        html.find("What we do")
            .map(|i| html[i..].chars().take(1500).collect::<String>())
            .unwrap_or_else(|| html.chars().take(1500).collect())
    );
}
