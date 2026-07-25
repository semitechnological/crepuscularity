import 'package:flutter/material.dart';

import '../dither/dither_paint.dart';
import '../dither/palette.dart';

/// Full-bleed ordered-dither gradient plane.
class DitherGradient extends StatelessWidget {
  const DitherGradient({
    this.color = DitherColor.blue,
    this.variant = DitherVariant.gradient,
    this.height = 80,
    this.horizontal = false,
    super.key,
  });

  final DitherColor color;
  final DitherVariant variant;
  final double height;
  final bool horizontal;

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      height: height,
      width: double.infinity,
      child: CustomPaint(
        painter: _GradientPainter(
          color: color,
          variant: variant,
          horizontal: horizontal,
        ),
      ),
    );
  }
}

class _GradientPainter extends CustomPainter {
  _GradientPainter({
    required this.color,
    required this.variant,
    required this.horizontal,
  });

  final DitherColor color;
  final DitherVariant variant;
  final bool horizontal;

  @override
  void paint(Canvas canvas, Size size) {
    if (size.width <= 0 || size.height <= 0) return;
    final backing = backingSize(size.width, size.height);
    final cols = backing.cols;
    final rows = backing.rows;
    final seed = seedOf(color);
    canvas.save();
    canvas.scale(size.width / cols, size.height / rows);
    if (horizontal) {
      for (var x = 0; x < cols; x++) {
        final t = (1 - x / (cols - 1)) * (rows - 1);
        paintDitherColumn(canvas, x, t.round(), rows - 1, seed, variant);
      }
    } else {
      for (var x = 0; x < cols; x++) {
        paintDitherColumn(canvas, x, 0, rows - 1, seed, variant);
      }
    }
    canvas.restore();
  }

  @override
  bool shouldRepaint(covariant _GradientPainter oldDelegate) {
    return oldDelegate.color != color ||
        oldDelegate.variant != variant ||
        oldDelegate.horizontal != horizontal;
  }
}
