import 'dart:ui';

import '../dither/palette.dart';

/// Named surface + seed theme for the component registry.
class ComponentTheme {
  const ComponentTheme({
    required this.name,
    required this.background,
    required this.foreground,
    required this.muted,
    required this.accent,
    required this.seeds,
  });

  final String name;
  final Color background;
  final Color foreground;
  final Color muted;
  final Color accent;
  final Map<String, DitherSeed> seeds;

  DitherSeed seed(String name) =>
      seeds[name] ?? seeds['blue'] ?? seedOf(DitherColor.blue);
}

Color _hex(String hex) {
  var h = hex.replaceFirst('#', '');
  if (h.length == 6) h = 'FF$h';
  return Color(int.parse(h, radix: 16));
}

DitherSeed _seed(Rgb fill, Rgb line, Rgb star) =>
    DitherSeed(fill: fill, line: line, star: star);

/// Built-in themes mirroring `catalog/themes/*.json`.
abstract final class ComponentThemes {
  static final ditherKit = ComponentTheme(
    name: 'dither-kit',
    background: _hex('#0a0a0c'),
    foreground: _hex('#f2f2f5'),
    muted: _hex('#5c5c64'),
    accent: _hex('#358ff3'),
    seeds: {
      for (final e in ditherPalette.entries) e.key.name: e.value,
    },
  );

  static final kumo = ComponentTheme(
    name: 'kumo',
    background: _hex('#ffffff'),
    foreground: _hex('#1a1a1a'),
    muted: _hex('#6b7280'),
    accent: _hex('#4290f0'),
    seeds: {
      'blue': _seed((66, 144, 240), (142, 188, 246), (200, 222, 251)),
      'yellow': _seed((245, 182, 71), (250, 210, 130), (255, 230, 180)),
      'pink': _seed((232, 100, 157), (245, 160, 195), (255, 205, 225)),
      'purple': _seed((141, 88, 238), (180, 145, 245), (210, 190, 255)),
      'teal': _seed((80, 195, 182), (140, 220, 210), (190, 235, 228)),
      'orange': _seed((211, 117, 54), (235, 165, 110), (250, 200, 160)),
      'grey': _seed((203, 203, 203), (160, 160, 160), (180, 180, 180)),
    },
  );

  static final night = ComponentTheme(
    name: 'night',
    background: _hex('#0c0e14'),
    foreground: _hex('#e8eaf0'),
    muted: _hex('#6a7080'),
    accent: _hex('#7aa2ff'),
    seeds: ditherKit.seeds,
  );

  static final chalk = ComponentTheme(
    name: 'chalk',
    background: _hex('#f7f5f0'),
    foreground: _hex('#1c1b19'),
    muted: _hex('#7a756c'),
    accent: _hex('#2f6fed'),
    seeds: {
      'green': _seed((40, 210, 110), (30, 160, 85), (20, 120, 65)),
      'blue': _seed((53, 143, 243), (40, 110, 200), (30, 80, 160)),
      'purple': _seed((150, 110, 255), (110, 80, 200), (80, 55, 160)),
      'pink': _seed((240, 90, 190), (190, 60, 150), (150, 40, 120)),
      'orange': _seed((255, 150, 50), (210, 110, 30), (170, 80, 20)),
      'red': _seed((240, 70, 70), (190, 45, 45), (150, 30, 30)),
      'grey': _seed((120, 118, 112), (90, 88, 82), (70, 68, 62)),
    },
  );

  static final aurora = ComponentTheme(
    name: 'aurora',
    background: _hex('#061018'),
    foreground: _hex('#e6fff8'),
    muted: _hex('#4a7a72'),
    accent: _hex('#28e0b0'),
    seeds: {
      'green': _seed((40, 220, 140), (120, 255, 200), (180, 255, 230)),
      'blue': _seed((40, 160, 255), (120, 200, 255), (180, 225, 255)),
      'purple': _seed((160, 100, 255), (200, 160, 255), (230, 200, 255)),
      'pink': _seed((255, 80, 180), (255, 150, 210), (255, 200, 230)),
      'orange': _seed((255, 140, 60), (255, 190, 120), (255, 220, 170)),
      'red': _seed((255, 70, 90), (255, 140, 150), (255, 190, 195)),
      'grey': _seed((80, 100, 110), (120, 140, 150), (160, 175, 185)),
    },
  );

  static final Map<String, ComponentTheme> byName = {
    ditherKit.name: ditherKit,
    kumo.name: kumo,
    night.name: night,
    chalk.name: chalk,
    aurora.name: aurora,
  };

  static ComponentTheme of(String name) => byName[name] ?? ditherKit;
}
