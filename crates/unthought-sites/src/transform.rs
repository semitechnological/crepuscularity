//! Transform Unthought `SiteBuilderData` into a Crepuscularity `TemplateContext`.

use crate::types::*;
use crepuscularity_core::TemplateContext;
use crepuscularity_web::render_from_files;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BuildError {
    #[error("Failed to parse payload: {0}")]
    ParseError(String),

    #[error("Template render error: {0}")]
    RenderError(String),
}

pub fn build_site(payload: &SiteBuilderData) -> Result<BuildResult, BuildError> {
    let files = load_templates();
    let ctx = payload_to_context(payload);

    let html =
        render_from_files(&files, "site.crepus#Page", &ctx).map_err(BuildError::RenderError)?;

    Ok(BuildResult {
        html,
        warnings: vec![],
    })
}

pub fn build_site_from_json(json: &str) -> Result<BuildResult, BuildError> {
    let payload: SiteBuilderData =
        serde_json::from_str(json).map_err(|e| BuildError::ParseError(e.to_string()))?;
    build_site(&payload)
}

fn load_templates() -> HashMap<String, String> {
    let mut files = HashMap::new();

    files.insert(
        "site.crepus".to_string(),
        include_str!("../../../templates/site/site.crepus").to_string(),
    );

    files.insert(
        "elements/hero.crepus".to_string(),
        include_str!("../../../templates/site/elements/hero.crepus").to_string(),
    );
    files.insert(
        "elements/services.crepus".to_string(),
        include_str!("../../../templates/site/elements/services.crepus").to_string(),
    );
    files.insert(
        "elements/stats.crepus".to_string(),
        include_str!("../../../templates/site/elements/stats.crepus").to_string(),
    );
    files.insert(
        "elements/feature-grid.crepus".to_string(),
        include_str!("../../../templates/site/elements/feature-grid.crepus").to_string(),
    );
    files.insert(
        "elements/story.crepus".to_string(),
        include_str!("../../../templates/site/elements/story.crepus").to_string(),
    );
    files.insert(
        "elements/process.crepus".to_string(),
        include_str!("../../../templates/site/elements/process.crepus").to_string(),
    );
    files.insert(
        "elements/testimonials.crepus".to_string(),
        include_str!("../../../templates/site/elements/testimonials.crepus").to_string(),
    );
    files.insert(
        "elements/faq.crepus".to_string(),
        include_str!("../../../templates/site/elements/faq.crepus").to_string(),
    );
    files.insert(
        "elements/cta.crepus".to_string(),
        include_str!("../../../templates/site/elements/cta.crepus").to_string(),
    );
    files.insert(
        "elements/contact-form.crepus".to_string(),
        include_str!("../../../templates/site/elements/contact-form.crepus").to_string(),
    );
    files.insert(
        "elements/shopify-collection.crepus".to_string(),
        include_str!("../../../templates/site/elements/shopify-collection.crepus").to_string(),
    );
    files.insert(
        "elements/shopify-product.crepus".to_string(),
        include_str!("../../../templates/site/elements/shopify-product.crepus").to_string(),
    );

    files
}

fn payload_to_context(payload: &SiteBuilderData) -> TemplateContext {
    let mut ctx = TemplateContext::new();
    ctx.set("businessName", payload.business_name.clone());
    ctx.set("seoTitle", payload.seo.title.clone());
    ctx.set("seoDescription", payload.seo.description.clone());
    ctx.set(
        "seoOgImage",
        payload.seo.og_image.clone().unwrap_or_default(),
    );

    ctx.set("themeAccent", payload.theme.accent.clone());
    ctx.set("themeAccentSoft", payload.theme.accent_soft.clone());
    ctx.set("themeSurface", payload.theme.surface.clone());
    ctx.set("themeText", payload.theme.text.clone());
    ctx.set("themeMuted", payload.theme.muted.clone());
    ctx.set("themeBorder", payload.theme.border.clone());

    let elements = payload
        .elements
        .iter()
        .map(element_to_context)
        .collect::<Vec<_>>();
    ctx.set("elements", elements);
    ctx.virtual_files = load_templates();
    ctx
}

fn element_to_context(element: &SiteElement) -> TemplateContext {
    let mut ctx = TemplateContext::new();
    ctx.set("id", element.id().to_string());
    ctx.set("type", element.element_type());

    match element {
        SiteElement::Hero(element) => {
            ctx.set("eyebrow", element.props.eyebrow.clone().unwrap_or_default());
            ctx.set("headline", element.props.headline.clone());
            ctx.set("subheadline", element.props.subheadline.clone());
            ctx.set("primaryCtaLabel", element.props.primary.label.clone());
            ctx.set("primaryCtaHref", element.props.primary.href.clone());

            if let Some(secondary) = &element.props.secondary {
                ctx.set("secondaryCtaLabel", secondary.label.clone());
                ctx.set("secondaryCtaHref", secondary.href.clone());
            }

            if let Some(media) = &element.props.media {
                ctx.set("imageUrl", media.src.clone());
                ctx.set("imageAlt", media.alt.clone());
            }
        }
        SiteElement::Stats(element) => {
            ctx.set("eyebrow", element.props.eyebrow.clone().unwrap_or_default());
            ctx.set("title", element.props.title.clone().unwrap_or_default());
            ctx.set(
                "description",
                element.props.description.clone().unwrap_or_default(),
            );
            let stats = element
                .props
                .items
                .iter()
                .map(|stat| {
                    let mut item = TemplateContext::new();
                    item.set("label", stat.label.clone());
                    item.set("value", stat.value.clone());
                    item.set("detail", stat.detail.clone().unwrap_or_default());
                    item
                })
                .collect::<Vec<_>>();
            ctx.set("stats", stats);
        }
        SiteElement::Services(element) => {
            ctx.set("eyebrow", element.props.eyebrow.clone().unwrap_or_default());
            ctx.set("title", element.props.title.clone());
            ctx.set(
                "description",
                element.props.description.clone().unwrap_or_default(),
            );
            let services = element
                .props
                .items
                .iter()
                .map(|service| {
                    let mut item = TemplateContext::new();
                    item.set("title", service.title.clone());
                    item.set("description", service.description.clone());
                    item.set("icon", service.badge.clone().unwrap_or_default());
                    item
                })
                .collect::<Vec<_>>();
            ctx.set("services", services);
        }
        SiteElement::FeatureGrid(element) => {
            ctx.set("eyebrow", element.props.eyebrow.clone().unwrap_or_default());
            ctx.set("title", element.props.title.clone());
            ctx.set(
                "description",
                element.props.description.clone().unwrap_or_default(),
            );
            let features = element
                .props
                .items
                .iter()
                .map(|feature| {
                    let mut item = TemplateContext::new();
                    item.set("title", feature.title.clone());
                    item.set("description", feature.description.clone());
                    item.set("badge", feature.badge.clone().unwrap_or_default());
                    item
                })
                .collect::<Vec<_>>();
            ctx.set("features", features);
            ctx.set("columns", 3_i64);
        }
        SiteElement::Story(element) => {
            ctx.set("eyebrow", element.props.eyebrow.clone().unwrap_or_default());
            ctx.set("title", element.props.title.clone());
            ctx.set("body", element.props.body.clone());
            ctx.set(
                "signature",
                element.props.pull_quote.clone().unwrap_or_default(),
            );
        }
        SiteElement::Process(element) => {
            ctx.set("eyebrow", element.props.eyebrow.clone().unwrap_or_default());
            ctx.set("title", element.props.title.clone());
            ctx.set(
                "description",
                element.props.description.clone().unwrap_or_default(),
            );
            let steps = element
                .props
                .steps
                .iter()
                .enumerate()
                .map(|(index, step)| {
                    let mut item = TemplateContext::new();
                    item.set("number", (index + 1) as i64);
                    item.set("title", step.title.clone());
                    item.set("description", step.description.clone());
                    item
                })
                .collect::<Vec<_>>();
            ctx.set("steps", steps);
        }
        SiteElement::Testimonials(element) => {
            ctx.set("eyebrow", element.props.eyebrow.clone().unwrap_or_default());
            ctx.set("title", element.props.title.clone());
            ctx.set(
                "description",
                element.props.description.clone().unwrap_or_default(),
            );
            let testimonials = element
                .props
                .items
                .iter()
                .map(|testimonial| {
                    let mut item = TemplateContext::new();
                    item.set("quote", testimonial.quote.clone());
                    item.set("author", testimonial.author.clone());
                    item.set("role", testimonial.role.clone());
                    item.set(
                        "initial",
                        testimonial.author.chars().next().unwrap_or('?').to_string(),
                    );
                    item
                })
                .collect::<Vec<_>>();
            ctx.set("testimonials", testimonials);
        }
        SiteElement::Faq(element) => {
            ctx.set("eyebrow", element.props.eyebrow.clone().unwrap_or_default());
            ctx.set("title", element.props.title.clone());
            ctx.set(
                "description",
                element.props.description.clone().unwrap_or_default(),
            );
            let items = element
                .props
                .items
                .iter()
                .map(|faq| {
                    let mut item = TemplateContext::new();
                    item.set("question", faq.question.clone());
                    item.set("answer", faq.answer.clone());
                    item
                })
                .collect::<Vec<_>>();
            ctx.set("items", items);
        }
        SiteElement::Cta(element) => {
            ctx.set("eyebrow", element.props.eyebrow.clone().unwrap_or_default());
            ctx.set("title", element.props.title.clone());
            ctx.set("body", element.props.body.clone());
            ctx.set("primaryLabel", element.props.primary.label.clone());
            ctx.set("primaryHref", element.props.primary.href.clone());
            if let Some(secondary) = &element.props.secondary {
                ctx.set("secondaryLabel", secondary.label.clone());
                ctx.set("secondaryHref", secondary.href.clone());
            }
        }
        SiteElement::ContactForm(element) => {
            ctx.set("eyebrow", element.props.eyebrow.clone().unwrap_or_default());
            ctx.set("title", element.props.title.clone());
            ctx.set(
                "description",
                element.props.description.clone().unwrap_or_default(),
            );
            ctx.set("formId", element.props.form_id.clone());
            ctx.set(
                "submitLabel",
                element
                    .props
                    .submit_label
                    .clone()
                    .unwrap_or_else(|| "Send Message".to_string()),
            );

            match &element.props.submission {
                ContactSubmission::Mailto { target } => {
                    ctx.set("endpoint", format!("mailto:{target}"));
                    ctx.set("method", "POST");
                }
                ContactSubmission::Url { target, method } => {
                    ctx.set("endpoint", target.clone());
                    ctx.set(
                        "method",
                        method
                            .clone()
                            .unwrap_or_else(|| "post".to_string())
                            .to_uppercase(),
                    );
                }
            }

            ctx.set(
                "successMessage",
                element.props.success_message.clone().unwrap_or_default(),
            );

            let fields = element
                .props
                .fields
                .iter()
                .map(|field| {
                    let mut item = TemplateContext::new();
                    item.set("name", field.name.clone());
                    item.set("label", field.label.clone());
                    item.set(
                        "type",
                        field
                            .field_type
                            .clone()
                            .unwrap_or_else(|| "text".to_string()),
                    );
                    item.set("required", field.required.unwrap_or(false));
                    item.set("placeholder", field.placeholder.clone().unwrap_or_default());

                    if let Some(options) = &field.options {
                        let select_options = options
                            .iter()
                            .map(|option| {
                                let mut value = TemplateContext::new();
                                value.set("value", option.clone());
                                value.set("label", option.clone());
                                value
                            })
                            .collect::<Vec<_>>();
                        item.set("options", select_options);
                    }

                    item
                })
                .collect::<Vec<_>>();
            ctx.set("fields", fields);
        }
        SiteElement::ShopifyCollection(element) => {
            ctx.set("eyebrow", element.props.eyebrow.clone().unwrap_or_default());
            ctx.set("title", element.props.title.clone());
            ctx.set(
                "description",
                element.props.description.clone().unwrap_or_default(),
            );
            ctx.set("collectionHandle", element.props.collection.handle.clone());

            let products = element
                .props
                .collection
                .products
                .iter()
                .map(shopify_product_to_context)
                .collect::<Vec<_>>();
            ctx.set("products", products);
        }
        SiteElement::ShopifyProduct(element) => {
            ctx.set("eyebrow", element.props.eyebrow.clone().unwrap_or_default());
            ctx.set("productHandle", element.props.product.href.clone());
            ctx.set("productTitle", element.props.product.name.clone());
            ctx.set(
                "productDescription",
                element.props.product.description.clone(),
            );
            ctx.set("productPrice", element.props.product.price.clone());
            ctx.set(
                "productCompareAtPrice",
                element
                    .props
                    .product
                    .compare_at_price
                    .clone()
                    .unwrap_or_default(),
            );

            if let Some(image) = &element.props.product.image {
                ctx.set("productImage", image.src.clone());
                ctx.set("productImageAlt", image.alt.clone());
                let mut image_ctx = TemplateContext::new();
                image_ctx.set("url", image.src.clone());
                image_ctx.set("alt", image.alt.clone());
                ctx.set("productImages", vec![image_ctx]);
            } else {
                ctx.set("productImage", "");
                ctx.set("productImageAlt", "");
                ctx.set("productImages", Vec::<TemplateContext>::new());
            }

            ctx.set("productVariants", Vec::<TemplateContext>::new());
        }
    }

    ctx
}

fn shopify_product_to_context(product: &ShopifyProduct) -> TemplateContext {
    let mut ctx = TemplateContext::new();
    ctx.set("title", product.name.clone());
    ctx.set(
        "handle",
        product.href.rsplit('/').next().unwrap_or("").to_string(),
    );
    ctx.set("description", product.description.clone());
    ctx.set("price", product.price.clone());
    ctx.set(
        "compareAtPrice",
        product.compare_at_price.clone().unwrap_or_default(),
    );
    ctx.set("href", product.href.clone());
    ctx.set("badge", product.badge.clone().unwrap_or_default());

    if let Some(image) = &product.image {
        ctx.set("image", image.src.clone());
        ctx.set("imageAlt", image.alt.clone());
        let mut image_ctx = TemplateContext::new();
        image_ctx.set("src", image.src.clone());
        image_ctx.set("alt", image.alt.clone());
        ctx.set("images", vec![image_ctx]);
    }

    ctx.set("variants", Vec::<TemplateContext>::new());
    ctx
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_runtime_payload() {
        let payload = SiteBuilderData {
            business_name: "Test Business".to_string(),
            domain: Some("example.com".to_string()),
            seo: SiteSeo {
                title: "Test Business".to_string(),
                description: "A test business site".to_string(),
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
                    eyebrow: None,
                    headline: "Welcome to Test Business".to_string(),
                    subheadline: "We make great things".to_string(),
                    primary: SiteButton {
                        label: "Get Started".to_string(),
                        href: "#contact".to_string(),
                        external: None,
                    },
                    secondary: None,
                    media: None,
                },
            })],
            analytics: None,
            turnstile: None,
        };

        let result = build_site(&payload).expect("runtime payload should render");
        assert!(result.html.contains("Welcome to Test Business"));
    }
}
