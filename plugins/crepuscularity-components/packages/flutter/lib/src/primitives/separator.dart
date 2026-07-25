import 'package:flutter/material.dart';

/// Thin horizontal or vertical divider.
class DitherSeparator extends StatelessWidget {
  const DitherSeparator({
    this.orientation = Axis.horizontal,
    this.color,
    this.thickness = 1,
    super.key,
  });

  final Axis orientation;
  final Color? color;
  final double thickness;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final c = color ?? theme.dividerColor.withValues(alpha: 0.55);
    if (orientation == Axis.vertical) {
      return SizedBox(
        width: thickness,
        child: ColoredBox(color: c),
      );
    }
    return SizedBox(
      height: thickness,
      width: double.infinity,
      child: ColoredBox(color: c),
    );
  }
}
