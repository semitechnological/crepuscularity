//! Type definitions for Unthought's canonical deploy-time `SiteBuilderData` contract.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteBuilderData {
    pub business_name: String,
    #[serde(default)]
    pub domain: Option<String>,
    pub seo: SiteSeo,
    pub theme: SiteTheme,
    pub elements: Vec<SiteElement>,
    #[serde(default)]
    pub analytics: Option<AnalyticsConfig>,
    #[serde(default)]
    pub turnstile: Option<TurnstileConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteSeo {
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub og_image: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteTheme {
    pub accent: String,
    pub accent_soft: String,
    pub surface: String,
    pub text: String,
    pub muted: String,
    pub border: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsConfig {
    #[serde(default)]
    pub cloudflare_web_analytics_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TurnstileConfig {
    #[serde(default)]
    pub site_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SiteElement {
    Hero(HeroElement),
    Stats(StatsElement),
    Services(ServicesElement),
    FeatureGrid(FeatureGridElement),
    Story(StoryElement),
    Process(ProcessElement),
    Testimonials(TestimonialsElement),
    Faq(FaqElement),
    Cta(CtaElement),
    ContactForm(ContactFormElement),
    ShopifyCollection(ShopifyCollectionElement),
    ShopifyProduct(ShopifyProductElement),
}

impl SiteElement {
    pub fn id(&self) -> &str {
        match self {
            SiteElement::Hero(element) => &element.id,
            SiteElement::Stats(element) => &element.id,
            SiteElement::Services(element) => &element.id,
            SiteElement::FeatureGrid(element) => &element.id,
            SiteElement::Story(element) => &element.id,
            SiteElement::Process(element) => &element.id,
            SiteElement::Testimonials(element) => &element.id,
            SiteElement::Faq(element) => &element.id,
            SiteElement::Cta(element) => &element.id,
            SiteElement::ContactForm(element) => &element.id,
            SiteElement::ShopifyCollection(element) => &element.id,
            SiteElement::ShopifyProduct(element) => &element.id,
        }
    }

    pub fn element_type(&self) -> &'static str {
        match self {
            SiteElement::Hero(_) => "hero",
            SiteElement::Stats(_) => "stats",
            SiteElement::Services(_) => "services",
            SiteElement::FeatureGrid(_) => "feature_grid",
            SiteElement::Story(_) => "story",
            SiteElement::Process(_) => "process",
            SiteElement::Testimonials(_) => "testimonials",
            SiteElement::Faq(_) => "faq",
            SiteElement::Cta(_) => "cta",
            SiteElement::ContactForm(_) => "contact_form",
            SiteElement::ShopifyCollection(_) => "shopify_collection",
            SiteElement::ShopifyProduct(_) => "shopify_product",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeroElement {
    pub id: String,
    pub props: HeroProps,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeroProps {
    #[serde(default)]
    pub eyebrow: Option<String>,
    pub headline: String,
    pub subheadline: String,
    pub primary: SiteButton,
    #[serde(default)]
    pub secondary: Option<SiteButton>,
    #[serde(default)]
    pub media: Option<SiteMedia>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsElement {
    pub id: String,
    pub props: StatsProps,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsProps {
    #[serde(default)]
    pub eyebrow: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    pub items: Vec<SiteStat>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicesElement {
    pub id: String,
    pub props: ServicesProps,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServicesProps {
    #[serde(default)]
    pub eyebrow: Option<String>,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    pub items: Vec<SiteService>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureGridElement {
    pub id: String,
    pub props: FeatureGridProps,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatureGridProps {
    #[serde(default)]
    pub eyebrow: Option<String>,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    pub items: Vec<SiteFeature>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryElement {
    pub id: String,
    pub props: StoryProps,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryProps {
    #[serde(default)]
    pub eyebrow: Option<String>,
    pub title: String,
    pub body: String,
    #[serde(default)]
    pub pull_quote: Option<String>,
    #[serde(default)]
    pub media: Option<SiteMedia>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessElement {
    pub id: String,
    pub props: ProcessProps,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessProps {
    #[serde(default)]
    pub eyebrow: Option<String>,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    pub steps: Vec<SiteStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestimonialsElement {
    pub id: String,
    pub props: TestimonialsProps,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestimonialsProps {
    #[serde(default)]
    pub eyebrow: Option<String>,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    pub items: Vec<SiteTestimonial>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaqElement {
    pub id: String,
    pub props: FaqProps,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FaqProps {
    #[serde(default)]
    pub eyebrow: Option<String>,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    pub items: Vec<SiteFaqItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtaElement {
    pub id: String,
    pub props: CtaProps,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CtaProps {
    #[serde(default)]
    pub eyebrow: Option<String>,
    pub title: String,
    pub body: String,
    pub primary: SiteButton,
    #[serde(default)]
    pub secondary: Option<SiteButton>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactFormElement {
    pub id: String,
    pub props: ContactFormProps,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContactFormProps {
    #[serde(default)]
    pub eyebrow: Option<String>,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    pub form_id: String,
    pub fields: Vec<SiteField>,
    pub submission: ContactSubmission,
    #[serde(default)]
    pub submit_label: Option<String>,
    #[serde(default)]
    pub success_message: Option<String>,
    #[serde(default)]
    pub hidden_fields: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopifyCollectionElement {
    pub id: String,
    pub props: ShopifyCollectionProps,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShopifyCollectionProps {
    #[serde(default)]
    pub eyebrow: Option<String>,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    pub collection: ShopifyCollection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopifyProductElement {
    pub id: String,
    pub props: ShopifyProductProps,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShopifyProductProps {
    #[serde(default)]
    pub eyebrow: Option<String>,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    pub product: ShopifyProduct,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteButton {
    pub label: String,
    pub href: String,
    #[serde(default)]
    pub external: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteMedia {
    pub src: String,
    pub alt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteStat {
    pub label: String,
    pub value: String,
    #[serde(default)]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteService {
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub href: Option<String>,
    #[serde(default)]
    pub badge: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteFeature {
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub badge: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteStep {
    pub title: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteTestimonial {
    pub quote: String,
    pub author: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteFaqItem {
    pub question: String,
    pub answer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteField {
    pub name: String,
    pub label: String,
    #[serde(default)]
    pub placeholder: Option<String>,
    #[serde(default)]
    pub field_type: Option<String>,
    #[serde(default)]
    pub required: Option<bool>,
    #[serde(default)]
    pub options: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum ContactSubmission {
    Mailto {
        target: String,
    },
    Url {
        target: String,
        #[serde(default)]
        method: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopifyProduct {
    pub name: String,
    pub description: String,
    pub price: String,
    #[serde(default)]
    pub compare_at_price: Option<String>,
    #[serde(default)]
    pub image: Option<SiteMedia>,
    pub href: String,
    #[serde(default)]
    pub badge: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopifyCollection {
    pub handle: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub href: Option<String>,
    pub products: Vec<ShopifyProduct>,
}

#[derive(Debug, Clone)]
pub struct BuildResult {
    pub html: String,
    pub warnings: Vec<String>,
}
