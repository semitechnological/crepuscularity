//! JSON-serializable view intermediate representation (platform-neutral).

#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Bumped when the JSON schema gains incompatible fields; shells should check `version`.
pub const IR_VERSION: u32 = 2;

/// Root document from parsing + lowering (see `crepuscularity_native::render_template_to_ir`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ViewIr {
    pub version: u32,
    pub root: Vec<ViewNode>,
}

/// Portable layout/theming hints mapped from Tailwind-like classes (see [`crate::style`]).
///
/// **Sizing sentinel values** — `width`, `height`, `max_width`, `max_height`:
/// - `> 0.0`  → absolute points
/// - `-1.0`   → fill parent (`maxWidth/maxHeight: .infinity` in SwiftUI)
/// - `-2.0`   → fit content (`fixedSize()` in SwiftUI)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ViewStyle {
    // ── Padding ──────────────────────────────────────────────────────────
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

    // ── Margin ───────────────────────────────────────────────────────────
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

    // ── Sizing ───────────────────────────────────────────────────────────
    /// Absolute pts, -1.0 = fill, -2.0 = fit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_width: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_height: Option<f32>,
    /// Absolute pts or -1.0 for fill
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_width: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_height: Option<f32>,
    /// Width as fraction of parent (0.0–1.0). Used for w-1/2, w-1/3, etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width_fraction: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height_fraction: Option<f32>,
    /// aspect-square → 1.0, aspect-video → 16/9
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<f32>,

    // ── Typography ───────────────────────────────────────────────────────
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_weight: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_align: Option<String>,
    /// Line-height multiplier (1.0 = none, 1.5 = normal, 2.0 = loose)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_height: Option<f32>,
    /// Letter spacing in points
    #[serde(skip_serializing_if = "Option::is_none")]
    pub letter_spacing: Option<f32>,
    /// "uppercase" | "lowercase" | "capitalize" | "normal"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_transform: Option<String>,
    /// "sans" | "serif" | "mono"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub italic: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub underline: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strikethrough: Option<bool>,

    // ── Color ────────────────────────────────────────────────────────────
    #[serde(skip_serializing_if = "Option::is_none")]
    pub foreground_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,

    // ── Border ───────────────────────────────────────────────────────────
    #[serde(skip_serializing_if = "Option::is_none")]
    pub corner_radius: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_width: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_color: Option<String>,

    // ── Opacity / visibility ─────────────────────────────────────────────
    /// 0.0–1.0
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hidden: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overflow_hidden: Option<bool>,

    // ── Flex ─────────────────────────────────────────────────────────────
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flex_grow: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flex_shrink: Option<f32>,
    /// "start" | "end" | "center" | "stretch" | "baseline"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub align_self: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flex_wrap: Option<bool>,
    /// "row" | "column" — explicit flex direction
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flex_direction: Option<String>,

    // ── Position & Layering ─────────────────────────────────────────────
    /// "static" | "relative" | "absolute" | "fixed"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub z_index: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub right: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bottom: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub left: Option<f32>,

    // ── Transform ────────────────────────────────────────────────────────
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translate_x: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translate_y: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale_x: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale_y: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotate: Option<f32>,

    // ── Shadow ────────────────────────────────────────────────────────
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shadow_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shadow_radius: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shadow_offset_x: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shadow_offset_y: Option<f32>,

    // ── Text Layout ─────────────────────────────────────────────────
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_overflow: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub white_space: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_clamp: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_select: Option<String>,
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
            && self.width.is_none()
            && self.height.is_none()
            && self.min_width.is_none()
            && self.min_height.is_none()
            && self.max_width.is_none()
            && self.max_height.is_none()
            && self.width_fraction.is_none()
            && self.height_fraction.is_none()
            && self.aspect_ratio.is_none()
            && self.font_size.is_none()
            && self.font_weight.is_none()
            && self.text_align.is_none()
            && self.line_height.is_none()
            && self.letter_spacing.is_none()
            && self.text_transform.is_none()
            && self.font_family.is_none()
            && self.italic.is_none()
            && self.underline.is_none()
            && self.strikethrough.is_none()
            && self.foreground_color.is_none()
            && self.background_color.is_none()
            && self.corner_radius.is_none()
            && self.border_width.is_none()
            && self.border_color.is_none()
            && self.opacity.is_none()
            && self.hidden.is_none()
            && self.overflow_hidden.is_none()
            && self.flex_grow.is_none()
            && self.flex_shrink.is_none()
            && self.align_self.is_none()
            && self.flex_wrap.is_none()
            && self.flex_direction.is_none()
            && self.position.is_none()
            && self.z_index.is_none()
            && self.top.is_none()
            && self.right.is_none()
            && self.bottom.is_none()
            && self.left.is_none()
            && self.translate_x.is_none()
            && self.translate_y.is_none()
            && self.scale_x.is_none()
            && self.scale_y.is_none()
            && self.rotate.is_none()
            && self.shadow_color.is_none()
            && self.shadow_radius.is_none()
            && self.shadow_offset_x.is_none()
            && self.shadow_offset_y.is_none()
            && self.text_overflow.is_none()
            && self.white_space.is_none()
            && self.line_clamp.is_none()
            && self.cursor.is_none()
            && self.user_select.is_none()
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
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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
    /// Single-line or multi-line text entry; maps to SwiftUI `TextField` / `TextEditor`.
    #[serde(rename = "input")]
    Input {
        placeholder: String,
        /// Context field name for the bound string (Swift codegen uses `context.<bind>`).
        bind: String,
        #[serde(default)]
        multiline: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        style: Option<ViewStyle>,
    },
    /// Segmented / picker control bound to a context string field.
    #[serde(rename = "picker")]
    Picker {
        /// Context field holding the selected option value.
        bind: String,
        options: Vec<PickerOption>,
        #[serde(skip_serializing_if = "Option::is_none")]
        style: Option<ViewStyle>,
    },
}

/// One selectable option in a [`ViewNode::Picker`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct PickerOption {
    pub value: String,
    pub label: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum StackAxis {
    Row,
    Column,
}
