import 'package:crepuscularity_components/crepuscularity_components.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('built-in themes are registered', () {
    expect(ComponentThemes.byName.keys, containsAll([
      'dither-kit',
      'kumo',
      'night',
      'chalk',
      'aurora',
    ]));
  });

  test('dither-kit seeds match dither-kit palette', () {
    final theme = ComponentThemes.ditherKit;
    expect(theme.seed('blue').fill, (53, 143, 243));
    expect(theme.seed('green').fill, (40, 210, 110));
  });

  test('kumo blue matches Cloudflare categorical blue', () {
    expect(ComponentThemes.kumo.seed('blue').fill, (66, 144, 240));
  });

  test('ComponentThemes.of falls back to dither-kit', () {
    expect(ComponentThemes.of('missing').name, 'dither-kit');
  });
}
