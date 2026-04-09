//! JSON-serializable view intermediate representation (platform-neutral).

use serde::{Deserialize, Serialize};

/// Bumped when the JSON schema gains incompatible fields; shells should check `version`.
pub const IR_VERSION: u32 = 2;

/// Root document from parsing + lowering (see `crepuscularity_native::render_template_to_ir`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewIr {
    pub version: u32,
    pub root: Vec<ViewNode>,
}

/// Portable layout/theming hints mapped from Tailwind-like classes (see [`crate::style`]).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ViewStyle {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding_horizontal: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding_vertical: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding_top: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding_bottom: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding_left: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding_right: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_horizontal: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_vertical: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_top: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_bottom: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_left: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_right: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_weight: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_align: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub foreground_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub corner_radius: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub italic: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub underline: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strikethrough: Option<bool>,
}

impl ViewStyle {
    fn is_effectively_empty(&self) -> bool {
        self.padding.is_none()
            && self.padding_horizontal.is_none()
            && self.padding_vertical.is_none()
            && self.padding_top.is_none()
            && self.padding_bottom.is_none()
            && self.padding_left.is_none()
            && self.padding_right.is_none()
            && self.margin.is_none()
            && self.margin_horizontal.is_none()
            && self.margin_vertical.is_none()
            && self.margin_top.is_none()
            && self.margin_bottom.is_none()
            && self.margin_left.is_none()
            && self.margin_right.is_none()
            && self.font_size.is_none()
            && self.font_weight.is_none()
            && self.text_align.is_none()
            && self.foreground_color.is_none()
            && self.background_color.is_none()
            && self.corner_radius.is_none()
            && self.italic.is_none()
            && self.underline.is_none()
            && self.strikethrough.is_none()
    }

    pub(crate) fn opt(self) -> Option<Self> {
        if self.is_effectively_empty() {
            None
        } else {
            Some(self)
        }
    }
}

/// A node in the platform-neutral tree. Serialized with `kind` for Swift/Kotlin.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ViewNode {
    #[serde(rename = "text")]
    Text {
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        style: Option<ViewStyle>,
    },
    #[serde(rename = "stack")]
    Stack {
        axis: StackAxis,
        #[serde(skip_serializing_if = "Option::is_none")]
        spacing: Option<f32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        align_items: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        justify_content: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        style: Option<ViewStyle>,
        children: Vec<ViewNode>,
    },
    #[serde(rename = "button")]
    Button {
        label: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        on_click: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        style: Option<ViewStyle>,
    },
    #[serde(rename = "image")]
    Image {
        src: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        alt: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        style: Option<ViewStyle>,
    },
    #[serde(rename = "scroll")]
    Scroll {
        axis: StackAxis,
        #[serde(skip_serializing_if = "Option::is_none")]
        style: Option<ViewStyle>,
        children: Vec<ViewNode>,
    },
    #[serde(rename = "slotRotate")]
    SlotRotate {
        phrases: Vec<String>,
        interval_ms: u64,
        #[serde(skip_serializing_if = "Option::is_none")]
        style: Option<ViewStyle>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StackAxis {
    Row,
    Column,
}
