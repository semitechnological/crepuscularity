import 'dart:convert';
import 'dart:math';

import 'package:crepuscularity_flutter/crepuscularity_flutter.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

/// Property-based coverage. Line coverage is saturated; what is *not* covered
/// is the combinatorial space of documents a model can emit. Every case below
/// asserts the same two invariants the package promises its host:
///
///   1. decoding never throws — no input produces an exception, only inert
///      nodes;
///   2. rendering never throws — every decoded document builds and lays out to
///      a widget tree, however contradictory the properties are.
///
/// The generator is driven by a seeded [Random] so any failure is reproducible
/// from the seed printed in the failure reason.
Widget _host(Widget child) => MaterialApp(home: Scaffold(body: child));

/// A render overflow is not a renderer defect: the package deliberately lays a
/// document out at its natural size and leaves scrolling to the host, so any
/// document taller than the viewport overflows by design. Everything else — a
/// failed assertion, an unbounded constraint, a null dereference — is a defect,
/// so only overflow is filtered out of the "never throws" invariant.
Object? _fatal(Object? error) {
  if (error is FlutterError && error.message.contains('overflowed'))
    return null;
  return error;
}

/// Values that have historically broken renderers: non-finite numbers, numbers
/// smuggled in as strings, wrong JSON types, and extremes.
const _hostileNumbers = <Object?>[
  0,
  -0.0,
  1,
  -1,
  0.5,
  -1e9,
  1e9,
  1e308,
  -1e308,
  double.maxFinite,
  double.minPositive,
  'nan',
  'NaN',
  'Infinity',
  '-Infinity',
  '1e400',
  '',
  ' 12 ',
  'twelve',
  null,
  true,
  false,
  <Object?>[],
  <String, Object?>{},
];

final _hostileStrings = <Object?>[
  '',
  ' ',
  'x',
  'a' * 300,
  '{unclosed',
  '{a.b.c}',
  '{}',
  '{{nested}}',
  '\n\t\r',
  '😀 ünïcø∂é 日本語',
  r'$literal',
  '"quoted"',
  '</script>',
  null,
  42,
  true,
  <Object?>[],
];

final _colorish = <Object?>[
  '#ff0000',
  '#ff0000aa',
  'ff0000',
  '#fff',
  '#',
  '#gggggg',
  '#zzzzzzzz',
  'red',
  '',
  '   #00ff00   ',
  '#' * 40,
  null,
  0xff0000,
  true,
];

const _aligns = <Object?>[
  'start',
  'center',
  'end',
  'stretch',
  'between',
  'around',
  'baseline',
  'STRETCH',
  '',
  null,
  7,
];

const _styleKeys = <String>[
  'padding',
  'paddingHorizontal',
  'paddingVertical',
  'paddingTop',
  'paddingBottom',
  'paddingLeft',
  'paddingRight',
  'fontSize',
  'fontWeight',
  'cornerRadius',
  'borderWidth',
  'opacity',
];

const _allowedKindList = <String>[
  'text',
  'stack',
  'scroll',
  'button',
  'toggle',
  'checkbox',
  'progress',
  'meter',
  'badge',
  'divider',
  'spacer',
  'image',
  'if',
  'forEach',
  'list',
  'listItem',
];

const _forbiddenKindList = <String>[
  'webView',
  'input',
  'picker',
  'slider',
  'tabs',
  'slot',
  'embed',
  'dropzone',
  '',
  'TEXT',
];

class _Gen {
  _Gen(this.seed) : rng = Random(seed);

  final int seed;
  final Random rng;
  int budget = 0;

  T pick<T>(List<T> options) => options[rng.nextInt(options.length)];

  bool chance(double p) => rng.nextDouble() < p;

  Object? hostileNumber() => pick(_hostileNumbers);

  Object? hostileString() => pick(_hostileStrings);

  Map<String, Object?>? style() {
    if (chance(0.35)) return null;
    final style = <String, Object?>{};
    for (final key in _styleKeys) {
      if (chance(0.25)) style[key] = hostileNumber();
    }
    if (chance(0.3)) style['foregroundColor'] = pick(_colorish);
    if (chance(0.3)) style['backgroundColor'] = pick(_colorish);
    if (chance(0.3)) style['borderColor'] = pick(_colorish);
    if (chance(0.2)) {
      style['textAlign'] = pick(const <Object?>[
        'left',
        'right',
        'center',
        'leading',
        'trailing',
        'justify',
        '',
        null,
        3,
      ]);
    }
    for (final flag in const ['italic', 'underline', 'strikethrough']) {
      if (chance(0.2)) {
        style[flag] = pick(const <Object?>[true, false, 'true', 1, null]);
      }
    }
    // Occasionally hand the decoder something that is not a map at all.
    if (chance(0.05)) return null;
    return style;
  }

  Object? maybeStyle() => chance(0.05)
      ? pick(const <Object?>['not-a-map', 7, <Object?>[], true])
      : style();

  List<Object?> children(int depth) {
    if (depth <= 0 || budget <= 0) return const [];
    final n = rng.nextInt(3);
    return List.generate(n, (_) => node(depth - 1));
  }

  Object? node(int depth) {
    budget -= 1;
    if (chance(0.04)) {
      // Not an object at all.
      return pick(const <Object?>[null, 'text', 3, true, <Object?>[]]);
    }
    final kind = chance(0.12)
        ? pick(_forbiddenKindList)
        : pick(_allowedKindList);
    final map = <String, Object?>{'kind': kind};
    if (chance(0.85)) map['style'] = maybeStyle();
    switch (kind) {
      case 'text':
        map['content'] = hostileString();
        map['bind'] = hostileString();
      case 'stack':
        map['axis'] = pick(const <Object?>[
          'row',
          'column',
          'diagonal',
          null,
          1,
        ]);
        map['spacing'] = hostileNumber();
        map['alignItems'] = pick(_aligns);
        map['justifyContent'] = pick(_aligns);
        map['onLongPress'] = hostileString();
        map['children'] = children(depth);
      case 'scroll':
        map['axis'] = pick(const <Object?>['row', 'column', null]);
        map['children'] = children(depth);
      case 'button':
        map['label'] = hostileString();
        map['onClick'] = hostileString();
        map['onLongPress'] = hostileString();
      case 'toggle':
        map['label'] = hostileString();
        map['checked'] = pick(const <Object?>[true, false, 'true', 1, null]);
        map['onChange'] = hostileString();
        map['onLongPress'] = hostileString();
      case 'checkbox':
        map['label'] = hostileString();
        map['checked'] = pick(const <Object?>[true, false, 'yes', 0, null]);
        map['onChange'] = hostileString();
      case 'progress':
        map['label'] = hostileString();
        map['value'] = hostileNumber();
        map['max'] = hostileNumber();
      case 'meter':
        map['label'] = hostileString();
        map['value'] = hostileNumber();
        map['min'] = hostileNumber();
        map['max'] = hostileNumber();
      case 'badge':
        map['label'] = hostileString();
        map['tone'] = pick(const <Object?>[
          'positive',
          'success',
          'warning',
          'negative',
          'danger',
          'info',
          'chartreuse',
          '',
          null,
          9,
        ]);
      case 'divider':
        map['axis'] = pick(const <Object?>['row', 'column', null]);
      case 'spacer':
        map['size'] = hostileNumber();
      case 'image':
        map['src'] = pick(const <Object?>[
          '',
          'https://example.invalid/a.png',
          'http://example.invalid/a.png',
          'file:///etc/passwd',
          'data:image/png;base64,zzz',
          'asset://a',
          'not a uri :: ///',
          null,
          5,
        ]);
        map['alt'] = hostileString();
        map['onLongPress'] = hostileString();
      case 'if':
        map['condition'] = pick(const <Object?>[
          'flag',
          '!flag',
          'missing',
          'a == "b"',
          'a != b',
          'true',
          'false',
          '',
          '==',
          '!',
          'a == b == c',
          'items',
          'user.role == "admin"',
          null,
          3,
        ]);
        map['thenChildren'] = children(depth);
        map['elseChildren'] = chance(0.5) ? children(depth) : null;
      case 'forEach':
        map['bind'] = pick(const <Object?>[
          'items',
          'empty',
          'notAList',
          'missing',
          '',
          null,
        ]);
        map['itemName'] = pick(const <Object?>['item', 'x', '', null]);
        map['itemBody'] = children(depth);
      case 'list':
        map['ordered'] = pick(const <Object?>[true, false, 'true', null]);
        map['children'] = children(depth);
      case 'listItem':
        map['onLongPress'] = hostileString();
        map['children'] = children(depth);
      default:
        map['children'] = children(depth);
    }
    return map;
  }

  Map<Object?, Object?> document() {
    budget = 40;
    final root = List.generate(
      1 + rng.nextInt(4),
      (_) => node(3 + rng.nextInt(3)),
    );
    return {
      'version': chance(0.1) ? pick(const <Object?>[null, 'v5', -1, 9999]) : 5,
      'root': chance(0.03)
          ? pick(const <Object?>[null, 'root', 7, <String, Object?>{}])
          : root,
    };
  }

  Map<String, Object?> scope() => {
    'flag': pick(const <Object?>[true, false, 0, 1, '', 'x', null]),
    'a': pick(const <Object?>['b', 1, true, null]),
    'b': pick(const <Object?>['b', 2, false, null]),
    'items': List.generate(
      rng.nextInt(4),
      (i) => chance(0.5) ? {'name': 'n$i', 'n': i} : 'plain$i',
    ),
    'empty': const <Object?>[],
    'notAList': 'scalar',
    'user': chance(0.5)
        ? {
            'role': pick(const ['admin', 'guest']),
          }
        : null,
  };
}

/// A structurally valid document used as the seed corpus for mutation fuzzing.
const _validSource = '''
+++
title = "demo"
+++
view p-4 gap-2 rounded-lg bg-white
  h1 text-2xl font-bold "Dashboard"
  text text-sm text-muted "Signed in as {user.name}"
  stack row gap-2 items-center justify-between
    badge "live" tone=positive
    button "Refresh" onclick=prompt:refresh
    toggle "Notify" checked onchange=toggle:notify
  progress label="Sync" value=42 max=100
  meter label="Disk" value=3 min=0 max=10
  if user.role == "admin"
    text "admin panel"
  else
    text "no access"
  for item in items
    li onlongpress=pin:item
      text "{item.name}"
  ul
    li
      text "one"
    li
      text "two"
  ol
    li
      text "first"
  scroll overflow-y-auto
    text "scrolled"
  divider
  spacer size=12
  img src=https://example.invalid/a.png alt="pic"
''';

String _mutate(String source, Random rng) {
  var s = source;
  final rounds = 1 + rng.nextInt(4);
  for (var r = 0; r < rounds; r++) {
    if (s.isEmpty) return s;
    switch (rng.nextInt(8)) {
      case 0: // delete a random span
        final i = rng.nextInt(s.length);
        final j = min(s.length, i + 1 + rng.nextInt(20));
        s = s.substring(0, i) + s.substring(j);
      case 1: // truncate
        s = s.substring(0, rng.nextInt(s.length));
      case 2: // insert a hostile char
        final i = rng.nextInt(s.length);
        final c = const [
          '"',
          '{',
          '}',
          '\\',
          '\n',
          '\t',
          '#',
          '=',
          ' ',
          '-',
        ][rng.nextInt(10)];
        s = s.substring(0, i) + c + s.substring(i);
      case 3: // duplicate a line
        final lines = s.split('\n');
        if (lines.isEmpty) break;
        final i = rng.nextInt(lines.length);
        lines.insert(i, lines[i]);
        s = lines.join('\n');
      case 4: // reindent a line arbitrarily
        final lines = s.split('\n');
        if (lines.isEmpty) break;
        final i = rng.nextInt(lines.length);
        lines[i] = ' ' * rng.nextInt(24) + lines[i].trimLeft();
        s = lines.join('\n');
      case 5: // shuffle two lines
        final lines = s.split('\n');
        if (lines.length < 2) break;
        final i = rng.nextInt(lines.length);
        final j = rng.nextInt(lines.length);
        final tmp = lines[i];
        lines[i] = lines[j];
        lines[j] = tmp;
        s = lines.join('\n');
      case 6: // drop a random line
        final lines = s.split('\n');
        if (lines.isEmpty) break;
        lines.removeAt(rng.nextInt(lines.length));
        s = lines.join('\n');
      case 7: // swap a random char
        final i = rng.nextInt(s.length);
        s =
            s.substring(0, i) +
            String.fromCharCode(rng.nextInt(120)) +
            s.substring(min(s.length, i + 1));
    }
  }
  return s;
}

/// A random `.crepus` document built from the grammar rather than by mutation.
String _generateSource(Random rng) {
  const tags = [
    'view',
    'stack',
    'text',
    'h1',
    'p',
    'span',
    'button',
    'toggle',
    'checkbox',
    'progress',
    'meter',
    'badge',
    'divider',
    'spacer',
    'img',
    'ul',
    'ol',
    'li',
    'scroll',
    'if',
    'else',
    'for',
    'foreach',
    'input',
    'webview',
    'slot',
    'unknown-tag',
  ];
  const classes = [
    'row',
    'col',
    'flex',
    'flex-row',
    'flex-col',
    'gap-2',
    'gap-99',
    'gap-x',
    'p-4',
    'px-2',
    'py-0',
    'pt-1',
    'pb-3',
    'pl-5',
    'pr-7',
    'items-center',
    'items-stretch',
    'items-end',
    'justify-between',
    'justify-around',
    'justify-end',
    'text-xs',
    'text-3xl',
    'text-center',
    'text-red-500',
    'bg-blue-900',
    'bg-transparent',
    'font-bold',
    'font-thin',
    'italic',
    'underline',
    'line-through',
    'rounded',
    'rounded-full',
    'rounded-bogus',
    'overflow-y-auto',
    'scroll',
    'checked',
    'p-',
    'rounded-',
    'unknown-class',
  ];
  const attrs = [
    'src=https://example.invalid/a.png',
    'alt="pic"',
    'label="L"',
    'value=42',
    'value=nan',
    'max=0',
    'min=100',
    'size=-4',
    'size=1e400',
    'tone=positive',
    'tone=???',
    'bind=items',
    'checked=false',
    'checked=true',
    'onclick=prompt:go',
    'onclick={ do "a" }',
    'on-change=x',
    'onLongPress=y',
    'on_long_press=z',
    '#anchor',
    'ordered',
    '=leading-equals',
    'trailing=',
  ];
  const texts = [
    '"hello"',
    '"{name}"',
    '"{a.b}"',
    r'"esc \" \\ \n"',
    '"unterminated',
    '""',
    '"😀"',
  ];

  final sb = StringBuffer();
  final lines = 1 + rng.nextInt(14);
  for (var i = 0; i < lines; i++) {
    sb.write(' ' * rng.nextInt(10));
    final tag = tags[rng.nextInt(tags.length)];
    sb.write(tag);
    if (tag == 'if') {
      sb.write(
        ' ${const ['flag', '!flag', 'a == "b"', 'true', '', 'user.role != "x"'][rng.nextInt(6)]}',
      );
    } else if (tag == 'for' || tag == 'foreach') {
      sb.write(
        ' ${const ['item in items', 'items as item', 'items', 'in', 'as x', ''][rng.nextInt(6)]}',
      );
    } else {
      for (var k = rng.nextInt(4); k > 0; k--) {
        sb.write(' ${classes[rng.nextInt(classes.length)]}');
      }
      for (var k = rng.nextInt(3); k > 0; k--) {
        sb.write(' ${attrs[rng.nextInt(attrs.length)]}');
      }
      if (rng.nextBool()) sb.write(' ${texts[rng.nextInt(texts.length)]}');
    }
    sb.write('\n');
  }
  return sb.toString();
}

void main() {
  group('IR fuzzing', () {
    test('decoding a randomised document never throws', () {
      for (var seed = 0; seed < 4000; seed++) {
        final gen = _Gen(seed);
        final doc = gen.document();
        expect(
          () => ViewIr.fromJson(doc),
          returnsNormally,
          reason: 'seed $seed: ${jsonEncode(doc)}',
        );
      }
    });

    test(
      'a decoded document always respects the caps it was decoded under',
      () {
        for (var seed = 0; seed < 2000; seed++) {
          final doc = _Gen(seed).document();
          final ir = ViewIr.fromJson(doc);
          expect(
            crepusExceedsLimits(ir.root, CrepusLimits.defaults),
            isFalse,
            reason: 'seed $seed: ${jsonEncode(doc)}',
          );
        }
      },
    );

    test('decoding is deterministic and idempotent', () {
      for (var seed = 0; seed < 500; seed++) {
        final doc = _Gen(seed).document();
        final a = ViewIr.fromJson(doc);
        final b = ViewIr.fromJson(doc);
        expect(a.root.length, b.root.length, reason: 'seed $seed');
        expect(a.version, b.version, reason: 'seed $seed');
      }
    });

    testWidgets('rendering a randomised document never throws', (tester) async {
      for (var seed = 0; seed < 600; seed++) {
        final gen = _Gen(seed);
        final doc = gen.document();
        final data = gen.scope();
        await tester.pumpWidget(
          _host(CrepusView.fromIr(doc, data: data, onAction: (_) {})),
        );
        expect(
          _fatal(tester.takeException()),
          isNull,
          reason: 'seed $seed: ${jsonEncode(doc)}',
        );
      }
    });

    testWidgets('a randomised document always builds a widget tree', (
      tester,
    ) async {
      for (var seed = 1000; seed < 1200; seed++) {
        final gen = _Gen(seed);
        await tester.pumpWidget(
          _host(CrepusView.fromIr(gen.document(), data: gen.scope())),
        );
        expect(_fatal(tester.takeException()), isNull, reason: 'seed $seed');
        expect(find.byType(CrepusView), findsOneWidget, reason: 'seed $seed');
      }
    });

    testWidgets('every interactive seam in a randomised document is tappable', (
      tester,
    ) async {
      // Exercising the gesture paths, not just the build path: a document whose
      // controls throw on activation would otherwise never be caught.
      for (var seed = 2000; seed < 2120; seed++) {
        final gen = _Gen(seed);
        final fired = <String>[];
        await tester.pumpWidget(
          _host(
            CrepusView.fromIr(
              gen.document(),
              data: gen.scope(),
              onAction: fired.add,
            ),
          ),
        );
        if (_fatal(tester.takeException()) != null)
          fail('build threw at seed $seed');
        for (final finder in [
          find.byType(TextButton),
          find.byType(Switch),
          find.byType(Checkbox),
        ]) {
          final count = tester.widgetList(finder).length;
          for (var i = 0; i < count; i++) {
            await tester.tap(finder.at(i), warnIfMissed: false);
            expect(
              _fatal(tester.takeException()),
              isNull,
              reason: 'seed $seed',
            );
          }
        }
        // Every dispatched action must be a non-empty string the host can act
        // on — the package must never surface a null or empty action.
        for (final action in fired) {
          expect(action, isNotEmpty, reason: 'seed $seed');
        }
      }
    });
  });

  group('.crepus source fuzzing', () {
    test('a mutated valid document always parses to something inert', () {
      final rng = Random(7);
      for (var i = 0; i < 4000; i++) {
        final mutated = _mutate(_validSource, rng);
        late ViewIr ir;
        expect(
          () => ir = viewIrFromSource(mutated),
          returnsNormally,
          reason: 'mutation $i:\n$mutated',
        );
        expect(ir.version, IR_VERSION);
        expect(
          crepusExceedsLimits(ir.root, CrepusLimits.defaults),
          isFalse,
          reason: 'mutation $i:\n$mutated',
        );
      }
    });

    test('a grammar-generated document always parses', () {
      final rng = Random(11);
      for (var i = 0; i < 4000; i++) {
        final source = _generateSource(rng);
        expect(
          () => viewIrFromSource(source),
          returnsNormally,
          reason: 'case $i:\n$source',
        );
      }
    });

    test('parsing is a pure function of the source', () {
      final rng = Random(13);
      for (var i = 0; i < 300; i++) {
        final source = _generateSource(rng);
        final a = viewIrFromSource(source);
        final b = viewIrFromSource(source);
        expect(a.root.length, b.root.length, reason: 'case $i:\n$source');
      }
    });

    testWidgets('a mutated document renders without throwing', (tester) async {
      final rng = Random(17);
      for (var i = 0; i < 400; i++) {
        final mutated = _mutate(_validSource, rng);
        await tester.pumpWidget(
          _host(
            CrepusView.fromSource(
              mutated,
              data: _Gen(i).scope(),
              onAction: (_) {},
            ),
          ),
        );
        expect(
          _fatal(tester.takeException()),
          isNull,
          reason: 'mutation $i:\n$mutated',
        );
      }
    });

    testWidgets('a grammar-generated document renders without throwing', (
      tester,
    ) async {
      final rng = Random(19);
      for (var i = 0; i < 400; i++) {
        final source = _generateSource(rng);
        await tester.pumpWidget(
          _host(
            CrepusView.fromSource(
              source,
              data: _Gen(i).scope(),
              onAction: (_) {},
            ),
          ),
        );
        expect(
          _fatal(tester.takeException()),
          isNull,
          reason: 'case $i:\n$source',
        );
      }
    });

    testWidgets('a randomly generated document survives every tap', (
      tester,
    ) async {
      final rng = Random(23);
      for (var i = 0; i < 150; i++) {
        final source = _generateSource(rng);
        await tester.pumpWidget(
          _host(CrepusView.fromSource(source, onAction: (_) {})),
        );
        if (_fatal(tester.takeException()) != null)
          fail('build threw:\n$source');
        for (final finder in [
          find.byType(TextButton),
          find.byType(Switch),
          find.byType(Checkbox),
        ]) {
          final count = tester.widgetList(finder).length;
          for (var k = 0; k < count; k++) {
            await tester.tap(finder.at(k), warnIfMissed: false);
            expect(_fatal(tester.takeException()), isNull, reason: source);
          }
        }
      }
    });
  });

  group('adversarial shapes the generator is unlikely to reach', () {
    testWidgets('deeply and widely mixed containers still lay out', (
      tester,
    ) async {
      // Every container kind wrapping every leaf kind, one pair at a time, so a
      // pairing that cannot lay out (an unbounded-constraint crash) is isolated
      // rather than hidden in a random tree.
      const containers = ['stack', 'scroll', 'list', 'listItem'];
      const leaves = [
        'text',
        'button',
        'toggle',
        'checkbox',
        'progress',
        'meter',
        'badge',
        'divider',
        'spacer',
        'image',
      ];
      for (final outer in containers) {
        for (final axis in const ['row', 'column']) {
          for (final align in const [
            'start',
            'center',
            'end',
            'stretch',
            null,
          ]) {
            for (final leaf in leaves) {
              final doc = {
                'version': 5,
                'root': [
                  {
                    'kind': outer,
                    'axis': axis,
                    'alignItems': align,
                    'children': [
                      {
                        'kind': leaf,
                        'content': 'x',
                        'label': 'x',
                        'src': '',
                        'value': 1,
                        'max': 2,
                      },
                    ],
                  },
                ],
              };
              await tester.pumpWidget(_host(CrepusView.fromIr(doc)));
              expect(
                _fatal(tester.takeException()),
                isNull,
                reason: '$outer/$axis/$align/$leaf',
              );
            }
          }
        }
      }
    });

    testWidgets('containers nested inside every other container lay out', (
      tester,
    ) async {
      const containers = ['stack', 'scroll', 'list', 'listItem'];
      for (final outer in containers) {
        for (final outerAxis in const ['row', 'column']) {
          for (final inner in containers) {
            for (final innerAxis in const ['row', 'column']) {
              final doc = {
                'version': 5,
                'root': [
                  {
                    'kind': outer,
                    'axis': outerAxis,
                    'children': [
                      {
                        'kind': inner,
                        'axis': innerAxis,
                        'children': [
                          {'kind': 'text', 'content': 'leaf'},
                        ],
                      },
                    ],
                  },
                ],
              };
              await tester.pumpWidget(_host(CrepusView.fromIr(doc)));
              expect(
                _fatal(tester.takeException()),
                isNull,
                reason: '$outer($outerAxis) > $inner($innerAxis)',
              );
            }
          }
        }
      }
    });
  });
}
