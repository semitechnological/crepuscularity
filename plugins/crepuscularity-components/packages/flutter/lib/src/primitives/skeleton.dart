import 'package:flutter/material.dart';

/// Lightweight placeholder block (static dither-tint fill; no shimmer yet).
class DitherSkeleton extends StatelessWidget {
  const DitherSkeleton({
    this.width = double.infinity,
    this.height = 16,
    this.rounded = true,
    this.color,
    super.key,
  });

  final double width;
  final double height;
  final bool rounded;
  final Color? color;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final fill = color ?? theme.colorScheme.onSurface.withValues(alpha: 0.12);
    return SizedBox(
      width: width,
      height: height,
      child: DecoratedBox(
        decoration: BoxDecoration(
          color: fill,
          borderRadius: rounded ? BorderRadius.circular(4) : BorderRadius.zero,
        ),
      ),
    );
  }
}
