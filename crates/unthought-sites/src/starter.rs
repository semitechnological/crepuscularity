//! Starter [`SiteBuilderData`](crate::SiteBuilderData) for scaffolding and CLI examples.

use crate::types::*;

/// Minimal valid one-page marketing payload for new site projects or smoke tests.
pub fn starter_site_payload(business_name: &str) -> SiteBuilderData {
    SiteBuilderData {
        business_name: business_name.to_string(),
        domain: None,
        seo: SiteSeo {
            title: business_name.to_string(),
            description: "Generated marketing page. Edit site.json and run `crepus web build`."
                .to_string(),
            og_image: None,
        },
        theme: SiteTheme {
            accent: "#3b82f6".to_string(),
            accent_soft: "#60a5fa".to_string(),
            surface: "#09090b".to_string(),
            text: "#fafafa".to_string(),
            muted: "#a1a1aa".to_string(),
            border: "#27272a".to_string(),
        },
        elements: vec![SiteElement::Hero(HeroElement {
            id: "hero-1".to_string(),
            props: HeroProps {
                eyebrow: Some("Welcome".to_string()),
                headline: format!("Meet {business_name}"),
                subheadline: "Ship a polished landing page from structured data.".to_string(),
                primary: SiteButton {
                    label: "Get started".to_string(),
                    href: "#contact".to_string(),
                    external: None,
                },
                secondary: None,
                media: None,
            },
        })],
        analytics: None,
        turnstile: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build_site;

    #[test]
    fn starter_renders_html() {
        let html = build_site(&starter_site_payload("Labs Inc"))
            .expect("starter payload should render")
            .html;
        assert!(html.contains("Labs Inc"));
        assert!(html.contains("Meet Labs Inc"));
        assert!(html.contains("<html"));
    }

    #[test]
    fn starter_round_trips_json() {
        let p = starter_site_payload("JSON Co");
        let json = serde_json::to_string(&p).expect("serialize");
        let back: SiteBuilderData = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.business_name, "JSON Co");
    }
}
