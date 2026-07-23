/// Renders the constrained [ViewIr] model to Flutter widgets.
///
/// This package is deliberately GENERIC and action-agnostic: `on_click` /
/// `on_change` / `on_long_press` strings are surfaced verbatim to the host via
/// [CrepusView.onAction]. The package never interprets, maps, or `eval`s an
/// action — the host owns that policy. Unknown/disallowed node kinds render
/// nothing.
library;

import 'package:flutter/material.dart';

import 'crepus_parser.dart';
import 'view_ir.dart';

/// Host callback for a surfaced action string. The package does not interpret
/// the string in any way; the host decides what (if anything) it means.
typedef CrepusActionCallback = void Function(String action);

/// Theming knobs. Defaults are derived from the ambient [ThemeData] so the
/// renderer looks reasonable with zero configuration; a host (e.g. omi's hub)
/// can override to match its palette.
class CrepusTheme {
  const CrepusTheme({
    this.textColor,
    this.mutedColor,
    this.accentColor,
    this.surfaceColor,
    this.borderColor,
    this.baseFontSize = 14,
  });

  final Color? textColor;
  final Color? mutedColor;
  final Color? accentColor;
  final Color? surfaceColor;
  final Color? borderColor;
  final double baseFontSize;

  CrepusTheme _resolved(BuildContext context) {
    final theme = Theme.of(context);
    final onSurface = theme.colorScheme.onSurface;
    return CrepusTheme(
      textColor: textColor ?? onSurface,
      mutedColor: mutedColor ?? onSurface.withValues(alpha: 0.6),
      accentColor: accentColor ?? theme.colorScheme.primary,
      surfaceColor: surfaceColor ?? theme.colorScheme.surface,
      borderColor: borderColor ?? theme.dividerColor,
      baseFontSize: baseFontSize,
    );
  }
}

/// A widget that renders Crepuscularity View IR or `.crepus` source.
class CrepusView extends StatelessWidget {
  /// Render from decoded View IR JSON (`{ version, root: [...] }`).
  CrepusView.fromIr(
    Map<Object?, Object?> ir, {
    super.key,
    this.onAction,
    this.theme = const CrepusTheme(),
    this.data = const {},
    CrepusLimits limits = CrepusLimits.defaults,
  }) : _ir = ViewIr.fromJson(ir, limits: limits);

  /// Render from `.crepus` source (parsed in pure Dart — no Rust round-trip).
  CrepusView.fromSource(
    String source, {
    super.key,
    this.onAction,
    this.theme = const CrepusTheme(),
    this.data = const {},
    CrepusLimits limits = CrepusLimits.defaults,
  }) : _ir = viewIrFromSource(source, limits: limits);

  final ViewIr _ir;
  final CrepusActionCallback? onAction;
  final CrepusTheme theme;

  /// Runtime scope for `{...}` interpolation and `if`/`forEach` bindings.
  final Map<String, Object?> data;

  /// The decoded document (exposed for tests / introspection).
  ViewIr get ir => _ir;

  @override
  Widget build(BuildContext context) {
    final resolved = theme._resolved(context);
    final renderer = _Renderer(resolved, onAction);
    final children = renderer.renderList(_ir.root, data);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: children,
    );
  }
}

class _Renderer {
  _Renderer(this.theme, this.onAction);

  final CrepusTheme theme;
  final CrepusActionCallback? onAction;

  List<Widget> renderList(List<ViewNode> nodes, Map<String, Object?> scope) {
    final widgets = <Widget>[];
    for (final node in nodes) {
      final rendered = render(node, scope);
      if (rendered != null) widgets.add(rendered);
    }
    return widgets;
  }

  Widget? render(ViewNode node, Map<String, Object?> scope) {
    final widget = switch (node) {
      TextNode() => _text(node, scope),
      StackNode() => _stack(node, scope),
      ScrollNode() => _scroll(node, scope),
      ButtonNode() => _button(node),
      ToggleNode() => _toggle(node),
      CheckboxNode() => _checkbox(node),
      ProgressNode() => _progress(node),
      MeterNode() => _meter(node),
      BadgeNode() => _badge(node),
      DividerNode() => _divider(node),
      SpacerNode() => _spacer(node),
      ImageNode() => _image(node),
      ListNode() => _list(node, scope),
      ListItemNode() => _listItem(node, scope),
      IfNode() => _ifNode(node, scope),
      ForEachNode() => _forEach(node, scope),
      // Unknown/disallowed kinds render nothing (a faint chip in debug only).
      UnsupportedNode() => _unsupported(node),
    };
    if (widget == null) return null;
    return _applyStyle(widget, node.style);
  }

  // ── Leaf + container renderers ────────────────────────────────────────────

  Widget _text(TextNode node, Map<String, Object?> scope) {
    final style = node.style;
    final content = _interpolate(node.content, scope);
    return Text(
      content,
      style: TextStyle(
        color: _color(style?.foregroundColor) ?? theme.textColor,
        fontSize: style?.fontSize ?? theme.baseFontSize,
        fontWeight: _weight(style?.fontWeight),
        fontStyle: style?.italic == true ? FontStyle.italic : null,
        decoration: _decoration(style),
        height: 1.35,
      ),
      textAlign: _textAlign(style?.textAlign),
    );
  }

  Widget _stack(StackNode node, Map<String, Object?> scope) {
    final children = renderList(node.children, scope);
    final spaced = _withSpacing(children, node.spacing, node.axis);
    final widget = node.axis == StackAxis.row
        ? Row(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: _crossAxis(node.alignItems),
            mainAxisAlignment: _mainAxis(node.justifyContent),
            children: spaced,
          )
        : Column(
            crossAxisAlignment: _crossAxisCol(node.alignItems),
            mainAxisSize: MainAxisSize.min,
            mainAxisAlignment: _mainAxis(node.justifyContent),
            children: spaced,
          );
    return _withLongPress(widget, node.onLongPress);
  }

  Widget _scroll(ScrollNode node, Map<String, Object?> scope) {
    final children = renderList(node.children, scope);
    return SingleChildScrollView(
      scrollDirection: node.axis == StackAxis.row
          ? Axis.horizontal
          : Axis.vertical,
      child: node.axis == StackAxis.row
          ? Row(mainAxisSize: MainAxisSize.min, children: children)
          : Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              mainAxisSize: MainAxisSize.min,
              children: children,
            ),
    );
  }

  Widget _button(ButtonNode node) {
    final child = Text(node.label);
    final button = TextButton(
      onPressed: node.onClick == null ? null : () => _dispatch(node.onClick),
      style: TextButton.styleFrom(
        foregroundColor: theme.accentColor,
        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      ),
      child: child,
    );
    return _withLongPress(button, node.onLongPress);
  }

  Widget _toggle(ToggleNode node) {
    final control = Switch(
      value: node.checked,
      onChanged: node.onChange == null ? null : (_) => _dispatch(node.onChange),
    );
    final row = Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        if (node.label.isNotEmpty)
          Flexible(child: Text(node.label, style: _bodyStyle())),
        control,
      ],
    );
    return _withLongPress(row, node.onLongPress);
  }

  Widget _checkbox(CheckboxNode node) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Checkbox(
          value: node.checked,
          onChanged: node.onChange == null
              ? null
              : (_) => _dispatch(node.onChange),
        ),
        if (node.label.isNotEmpty)
          Flexible(child: Text(node.label, style: _bodyStyle())),
      ],
    );
  }

  Widget _progress(ProgressNode node) {
    final fraction = node.max <= 0 ? 0.0 : (node.value / node.max).clamp(0, 1);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: [
        if (node.label != null && node.label!.isNotEmpty)
          Padding(
            padding: const EdgeInsets.only(bottom: 4),
            child: Text(node.label!, style: _mutedStyle()),
          ),
        ClipRRect(
          borderRadius: BorderRadius.circular(4),
          child: LinearProgressIndicator(
            value: fraction.toDouble(),
            minHeight: 6,
            backgroundColor: theme.borderColor,
            valueColor: AlwaysStoppedAnimation(theme.accentColor!),
          ),
        ),
      ],
    );
  }

  Widget _meter(MeterNode node) {
    final span = node.max - node.min;
    final fraction = span <= 0
        ? 0.0
        : ((node.value - node.min) / span).clamp(0, 1);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: [
        if (node.label != null && node.label!.isNotEmpty)
          Padding(
            padding: const EdgeInsets.only(bottom: 4),
            child: Text(node.label!, style: _mutedStyle()),
          ),
        ClipRRect(
          borderRadius: BorderRadius.circular(4),
          child: LinearProgressIndicator(
            value: fraction.toDouble(),
            minHeight: 6,
            backgroundColor: theme.borderColor,
            valueColor: AlwaysStoppedAnimation(theme.accentColor!),
          ),
        ),
      ],
    );
  }

  Widget _badge(BadgeNode node) {
    final tone = _toneColor(node.tone);
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
      decoration: BoxDecoration(
        color: tone.withValues(alpha: 0.14),
        borderRadius: BorderRadius.circular(999),
      ),
      child: Text(
        node.label,
        style: TextStyle(
          fontSize: 11,
          fontWeight: FontWeight.w600,
          color: tone,
        ),
      ),
    );
  }

  Widget _divider(DividerNode node) => node.axis == StackAxis.row
      ? SizedBox(
          height: 16,
          child: VerticalDivider(width: 1, color: theme.borderColor),
        )
      : Divider(height: 1, color: theme.borderColor);

  Widget _spacer(SpacerNode node) {
    final raw = node.size ?? 8;
    final size = raw.isFinite && raw > 0 ? raw : 0.0;
    return SizedBox(width: size, height: size);
  }

  Widget _image(ImageNode node) {
    final uri = Uri.tryParse(node.src);
    final isNetwork =
        uri != null && (uri.scheme == 'http' || uri.scheme == 'https');
    final placeholder = _imagePlaceholder(node);
    final image = isNetwork
        ? Image.network(
            node.src,
            fit: BoxFit.cover,
            errorBuilder: (_, _, _) => placeholder,
          )
        : placeholder;
    return _withLongPress(image, node.onLongPress);
  }

  Widget _imagePlaceholder(ImageNode node) => Container(
    padding: const EdgeInsets.all(8),
    color: theme.surfaceColor,
    child: Text(node.alt ?? '', style: _mutedStyle()),
  );

  Widget _list(ListNode node, Map<String, Object?> scope) {
    final items = <Widget>[];
    var index = 1;
    for (final child in node.children) {
      final marker = node.ordered ? '$index.' : '•';
      final rendered = render(child, scope);
      if (rendered == null) continue;
      items.add(
        Padding(
          padding: const EdgeInsets.symmetric(vertical: 2),
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Padding(
                padding: const EdgeInsets.only(right: 6),
                child: Text(marker, style: _mutedStyle()),
              ),
              Flexible(child: rendered),
            ],
          ),
        ),
      );
      index += 1;
    }
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: items,
    );
  }

  Widget _listItem(ListItemNode node, Map<String, Object?> scope) {
    final children = renderList(node.children, scope);
    final widget = Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: children,
    );
    return _withLongPress(widget, node.onLongPress);
  }

  Widget? _ifNode(IfNode node, Map<String, Object?> scope) {
    final branch = _evalCondition(node.condition, scope)
        ? node.thenChildren
        : node.elseChildren;
    if (branch == null || branch.isEmpty) return null;
    final children = renderList(branch, scope);
    if (children.isEmpty) return null;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: children,
    );
  }

  Widget? _forEach(ForEachNode node, Map<String, Object?> scope) {
    final source = scope[node.bind];
    if (source is! List) return null;
    final children = <Widget>[];
    for (final item in source) {
      final itemScope = {...scope, node.itemName: item};
      children.addAll(renderList(node.itemBody, itemScope));
    }
    if (children.isEmpty) return null;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: children,
    );
  }

  Widget? _unsupported(UnsupportedNode node) {
    // In debug builds only, surface a faint chip so authors notice a disallowed
    // kind; release builds render nothing at all.
    var debug = false;
    assert(() {
      debug = true;
      return true;
    }());
    if (!debug) return null;
    return Opacity(
      opacity: 0.4,
      child: Text('unsupported: ${node.kind}', style: _mutedStyle()),
    );
  }

  // ── The single dispatch seam to the host ──────────────────────────────────

  void _dispatch(String? action) {
    if (action == null || action.isEmpty) return;
    onAction?.call(action);
  }

  Widget _withLongPress(Widget child, String? action) {
    if (action == null || action.isEmpty) return child;
    return GestureDetector(onLongPress: () => _dispatch(action), child: child);
  }

  // ── Style + helpers ───────────────────────────────────────────────────────

  Widget _applyStyle(Widget child, ViewStyle? style) {
    if (style == null) return child;
    var result = child;
    final padding = _padding(style);
    if (padding != null) result = Padding(padding: padding, child: result);
    final bg = _color(style.backgroundColor);
    final borderColor = _color(style.borderColor);
    final radius = style.cornerRadius;
    final safeRadius = radius != null && radius.isFinite && radius > 0
        ? radius
        : null;
    if (bg != null || safeRadius != null || borderColor != null) {
      final borderWidth = style.borderWidth;
      result = DecoratedBox(
        decoration: BoxDecoration(
          color: bg,
          borderRadius: safeRadius == null
              ? null
              : BorderRadius.circular(safeRadius),
          border: borderColor == null
              ? null
              : Border.all(
                  color: borderColor,
                  width:
                      borderWidth != null &&
                          borderWidth.isFinite &&
                          borderWidth >= 0
                      ? borderWidth
                      : 1,
                ),
        ),
        child: result,
      );
    }
    if (style.opacity != null) {
      result = Opacity(opacity: style.opacity!.clamp(0, 1), child: result);
    }
    return result;
  }

  EdgeInsets? _padding(ViewStyle s) {
    if (s.padding == null &&
        s.paddingHorizontal == null &&
        s.paddingVertical == null &&
        s.paddingTop == null &&
        s.paddingBottom == null &&
        s.paddingLeft == null &&
        s.paddingRight == null) {
      return null;
    }
    return EdgeInsets.only(
      top: s.paddingTop ?? s.paddingVertical ?? s.padding ?? 0,
      bottom: s.paddingBottom ?? s.paddingVertical ?? s.padding ?? 0,
      left: s.paddingLeft ?? s.paddingHorizontal ?? s.padding ?? 0,
      right: s.paddingRight ?? s.paddingHorizontal ?? s.padding ?? 0,
    );
  }

  List<Widget> _withSpacing(
    List<Widget> children,
    double? spacing,
    StackAxis axis,
  ) {
    if (spacing == null || spacing <= 0 || children.length < 2) return children;
    final gap = axis == StackAxis.row
        ? SizedBox(width: spacing)
        : SizedBox(height: spacing);
    final out = <Widget>[];
    for (var i = 0; i < children.length; i++) {
      if (i > 0) out.add(gap);
      out.add(children[i]);
    }
    return out;
  }

  TextStyle _bodyStyle() =>
      TextStyle(color: theme.textColor, fontSize: theme.baseFontSize);

  TextStyle _mutedStyle() =>
      TextStyle(color: theme.mutedColor, fontSize: theme.baseFontSize - 2);

  Color _toneColor(String? tone) {
    switch (tone) {
      case 'positive':
      case 'success':
        return const Color(0xff22c55e);
      case 'warning':
        return const Color(0xfff59e0b);
      case 'negative':
      case 'danger':
        return const Color(0xffef4444);
      case 'info':
        return theme.accentColor!;
    }
    return theme.mutedColor!;
  }

  FontWeight _weight(int? weight) {
    if (weight == null) return FontWeight.w400;
    if (weight >= 800) return FontWeight.w800;
    if (weight >= 700) return FontWeight.w700;
    if (weight >= 600) return FontWeight.w600;
    if (weight >= 500) return FontWeight.w500;
    if (weight >= 400) return FontWeight.w400;
    return FontWeight.w300;
  }

  TextDecoration? _decoration(ViewStyle? style) {
    final decorations = <TextDecoration>[
      if (style?.underline == true) TextDecoration.underline,
      if (style?.strikethrough == true) TextDecoration.lineThrough,
    ];
    if (decorations.isEmpty) return null;
    return TextDecoration.combine(decorations);
  }

  TextAlign? _textAlign(String? value) => switch (value) {
    'center' => TextAlign.center,
    'right' || 'trailing' => TextAlign.right,
    'left' || 'leading' => TextAlign.left,
    _ => null,
  };

  /// Row cross-axis. `stretch` is deliberately NOT honored here: a stretched
  /// [Row] demands a bounded height, and the renderer's own outer [Column] is
  /// `mainAxisSize.min`, so `stack row items-stretch` would throw an unbounded
  /// -constraint assertion on otherwise valid model-authored input.
  CrossAxisAlignment _crossAxis(String? align) => switch (align) {
    'center' => CrossAxisAlignment.center,
    'end' => CrossAxisAlignment.end,
    _ => CrossAxisAlignment.start,
  };

  CrossAxisAlignment _crossAxisCol(String? align) => switch (align) {
    'center' => CrossAxisAlignment.center,
    'end' => CrossAxisAlignment.end,
    'stretch' => CrossAxisAlignment.stretch,
    _ => CrossAxisAlignment.start,
  };

  MainAxisAlignment _mainAxis(String? justify) => switch (justify) {
    'center' => MainAxisAlignment.center,
    'end' => MainAxisAlignment.end,
    'between' => MainAxisAlignment.spaceBetween,
    'around' => MainAxisAlignment.spaceAround,
    _ => MainAxisAlignment.start,
  };

  Color? _color(String? hex) {
    if (hex == null) return null;
    var t = hex.trim();
    if (t.startsWith('#')) t = t.substring(1);
    if (t.length == 6) t = 'ff$t';
    if (t.length == 8) {
      // Hex is RRGGBBAA in some pipelines and AARRGGBB here; ir.rs emits
      // #RRGGBB(AA). Normalize #RRGGBBAA → 0xAARRGGBB.
      final value = int.tryParse(t, radix: 16);
      if (value == null) return null;
      // If the source had 8 digits (RRGGBBAA), rotate alpha to the front.
      if (hex.replaceAll('#', '').length == 8) {
        final rgb = value >> 8;
        final a = value & 0xff;
        return Color((a << 24) | rgb);
      }
      return Color(value);
    }
    return null;
  }

  /// Substitute `{path}` tokens from [scope] (dotted paths into maps). Missing
  /// keys resolve to empty string. NOT an expression evaluator.
  String _interpolate(String content, Map<String, Object?> scope) {
    if (!content.contains('{')) return content;
    return content.replaceAllMapped(RegExp(r'\{([^}]*)\}'), (match) {
      final path = match.group(1)!.trim();
      final value = _lookup(path, scope);
      return value == null ? '' : '$value';
    });
  }

  Object? _lookup(String path, Map<String, Object?> scope) {
    final parts = path.split('.');
    Object? current = scope[parts.first];
    for (final part in parts.skip(1)) {
      if (current is Map) {
        current = current[part];
      } else {
        return null;
      }
    }
    return current;
  }

  /// Evaluate a constrained `if` condition WITHOUT eval. Supports: a bare
  /// variable (truthy), `!var`, and `left == "value"` / `left != "value"`.
  bool _evalCondition(String condition, Map<String, Object?> scope) {
    var expr = condition.trim();
    if (expr.isEmpty) return false;
    for (final op in const ['==', '!=']) {
      final idx = expr.indexOf(op);
      if (idx > 0) {
        final left = _operand(expr.substring(0, idx).trim(), scope);
        final right = _operand(expr.substring(idx + op.length).trim(), scope);
        final equal = '$left' == '$right';
        return op == '==' ? equal : !equal;
      }
    }
    if (expr.startsWith('!')) {
      return !_truthy(_lookup(expr.substring(1).trim(), scope));
    }
    return _truthy(_lookup(expr, scope));
  }

  Object? _operand(String token, Map<String, Object?> scope) {
    if (token.startsWith('"') && token.endsWith('"') && token.length >= 2) {
      return token.substring(1, token.length - 1);
    }
    if (token == 'true') return true;
    if (token == 'false') return false;
    final number = num.tryParse(token);
    if (number != null) return number;
    return _lookup(token, scope);
  }

  bool _truthy(Object? value) {
    if (value == null) return false;
    if (value is bool) return value;
    if (value is num) return value != 0;
    if (value is String) return value.isNotEmpty;
    if (value is Iterable) return value.isNotEmpty;
    if (value is Map) return value.isNotEmpty;
    return true;
  }
}
