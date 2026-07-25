import 'package:flutter/material.dart';

import '../dither/palette.dart';

enum DitherBadgeTone { solid, soft }

class DitherBadge extends StatelessWidget {
  const DitherBadge({
    required this.label,
    this.color = DitherColor.blue,
    this.tone = DitherBadgeTone.soft,
    super.key,
  });

  final String label;
  final DitherColor color;
  final DitherBadgeTone tone;

  @override
  Widget build(BuildContext context) {
    final fill = ditherRgb(seedOf(color).fill);
    final bg = tone == DitherBadgeTone.solid
        ? fill
        : fill.withValues(alpha: 0.18);
    final fg = tone == DitherBadgeTone.solid ? Colors.white : fill;
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
      decoration: BoxDecoration(
        color: bg,
        borderRadius: BorderRadius.circular(4),
      ),
      child: Text(
        label,
        style: TextStyle(
          color: fg,
          fontSize: 12,
          fontWeight: FontWeight.w600,
        ),
      ),
    );
  }
}
