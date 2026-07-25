import 'package:flutter/material.dart';

import '../dither/palette.dart';

enum DitherButtonVariant { solid, outline, ghost }

class DitherButton extends StatelessWidget {
  const DitherButton({
    required this.label,
    this.onPressed,
    this.color = DitherColor.blue,
    this.variant = DitherButtonVariant.solid,
    this.disabled = false,
    super.key,
  });

  final String label;
  final VoidCallback? onPressed;
  final DitherColor color;
  final DitherButtonVariant variant;
  final bool disabled;

  @override
  Widget build(BuildContext context) {
    final fill = ditherRgb(seedOf(color).fill);
    final enabled = !disabled && onPressed != null;
    return TextButton(
      onPressed: enabled ? onPressed : null,
      style: TextButton.styleFrom(
        backgroundColor: switch (variant) {
          DitherButtonVariant.solid => fill,
          DitherButtonVariant.outline || DitherButtonVariant.ghost =>
            Colors.transparent,
        },
        foregroundColor: switch (variant) {
          DitherButtonVariant.solid => Colors.white,
          _ => fill,
        },
        side: variant == DitherButtonVariant.outline
            ? BorderSide(color: fill)
            : BorderSide.none,
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 10),
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(6)),
      ),
      child: Text(label),
    );
  }
}
