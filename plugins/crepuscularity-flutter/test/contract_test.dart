import 'package:crepuscularity_flutter/crepuscularity_flutter.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

/// The contract omi (and any other host) relies on. omi renders model-authored
/// UI through this package and treats the action string plus the kind allowlist
/// as its security boundary, so the properties pinned here are load-bearing:
///
/// * action strings reach `onAction` byte-for-byte, never interpreted;
/// * anything outside [kAllowedKinds] surfaces as [UnsupportedNode] rather than
///   being dropped or coerced into a renderable kind;
/// * the node/depth/source caps hold exactly at their documented boundaries.
Widget _host(Widget child) => MaterialApp(home: Scaffold(body: child));

Widget _doc(
  List<Map<String, Object?>> root, {
  CrepusActionCallback? onAction,
  Map<String, Object?> data = const {},
}) => _host(
  CrepusView.fromIr(
    {'version': 5, 'root': root},
    onAction: onAction,
    data: data,
  ),
);

/// Action strings chosen to break any host that normalises, unescapes, trims,
/// URL-decodes or scheme-rewrites on the way through.
const _hostileActions = <String>[
  'prompt:do it',
  'nav:/settings?x=1&y=2#frag',
  'javascript:alert(1)',
  'data:text/html;base64,PHNjcmlwdD4=',
  'file:///etc/passwd',
  '  leading and trailing  ',
  'multi\nline\r\naction',
  'tab\there',
  'quote"inside',
  r'back\slash',
  r'\n not a newline',
  'café — 日本語 🎉 ​',
  '%2Fencoded%20path%00',
  '{not interpolated}',
  r'${x}',
  '<script>alert(1)</script>',
  'a&b=c;d|e`f\$g',
  'UPPER:Case',
  '../../traversal',
  '0',
];

Map<String, Object?> _nest(int levels) {
  Map<String, Object?> node = const {'kind': 'text', 'content': 'leaf'};
  for (var i = 0; i < levels - 1; i++) {
    node = {
      'kind': 'stack',
      'children': [node],
    };
  }
  return node;
}

void main() {
  group('the kind allowlist is the security boundary', () {
    test('the allowlist is exactly the audited set', () {
      // Widening this set is a deliberate security decision; it must not happen
      // as a side effect of an unrelated change.
      expect(kAllowedKinds, {
        'text',
        'stack',
        'scroll',
        'button',
        'toggle',
        'checkbox',
        'progress',
        'meter',
        'sparkline',
        'badge',
        'divider',
        'spacer',
        'image',
        'if',
        'forEach',
        'list',
        'listItem',
      });
      expect(kAllowedKinds, hasLength(17));
    });

    test('every executing/disallowed kind decodes to UnsupportedNode', () {
      const disallowed = [
        'webView',
        'webview',
        'iframe',
        'filePicker',
        'dropzone',
        'input',
        'textField',
        'picker',
        'slider',
        'tabs',
        'slot',
        'embed',
        'script',
        'mediaPicker',
        'shell',
        'exec',
      ];
      for (final kind in disallowed) {
        final ir = ViewIr.fromJson({
          'root': [
            {'kind': kind, 'content': 'x', 'children': <Object?>[]},
          ],
        });
        expect(ir.root.single, isA<UnsupportedNode>(), reason: kind);
        expect((ir.root.single as UnsupportedNode).kind, kind, reason: kind);
      }
    });

    test('an allowed kind is never matched case-insensitively or trimmed', () {
      for (final kind in const ['Text', 'TEXT', ' text', 'text ', 'te xt']) {
        final node = ViewNode.fromJson({'kind': kind, 'content': 'x'});
        expect(node, isA<UnsupportedNode>(), reason: kind);
        expect((node as UnsupportedNode).kind, kind, reason: kind);
      }
    });

    test('a missing or non-string kind is unsupported, not a default kind', () {
      for (final kind in const <Object?>[null, 1, true, <Object?>[], {}]) {
        final node = ViewNode.fromJson({'kind': kind});
        expect(node, isA<UnsupportedNode>(), reason: '$kind');
        expect((node as UnsupportedNode).kind, '<no-kind>');
      }
      expect(ViewNode.fromJson(null), isA<UnsupportedNode>());
      expect(ViewNode.fromJson('text'), isA<UnsupportedNode>());
      expect(ViewNode.fromJson(<Object?>[]), isA<UnsupportedNode>());
      expect(
        (ViewNode.fromJson('text') as UnsupportedNode).kind,
        '<non-object>',
      );
    });

    test('a disallowed kind nested anywhere still surfaces, never dropped', () {
      final ir = ViewIr.fromJson({
        'root': [
          {
            'kind': 'stack',
            'children': [
              {
                'kind': 'listItem',
                'children': [
                  {'kind': 'webView', 'src': 'https://evil.example'},
                ],
              },
            ],
          },
          {
            'kind': 'if',
            'condition': 'x',
            'thenChildren': [
              {'kind': 'input'},
            ],
            'elseChildren': [
              {'kind': 'slot'},
            ],
          },
          {
            'kind': 'forEach',
            'bind': 'xs',
            'itemBody': [
              {'kind': 'dropzone'},
            ],
          },
        ],
      });
      final found = <String>[];
      void walk(List<ViewNode> nodes) {
        for (final n in nodes) {
          if (n is UnsupportedNode) found.add(n.kind);
          walk(childrenOf(n));
        }
      }

      walk(ir.root);
      expect(found, ['webView', 'input', 'slot', 'dropzone']);
    });

    testWidgets('an unsupported node never renders an interactive affordance', (
      tester,
    ) async {
      final fired = <String>[];
      await tester.pumpWidget(
        _doc(const [
          {
            'kind': 'webView',
            'label': 'Tap me',
            'onClick': 'evil',
            'src': 'https://evil.example',
          },
        ], onAction: fired.add),
      );
      expect(find.byType(TextButton), findsNothing);
      expect(find.byType(Switch), findsNothing);
      expect(find.byType(Checkbox), findsNothing);
      expect(find.byType(GestureDetector), findsNothing);
      expect(find.byType(Image), findsNothing);
      expect(find.text('Tap me'), findsNothing);
      expect(fired, isEmpty);
    });

    test('the .crepus parser refuses the same executing tags', () {
      const tags = [
        'iframe',
        'webview',
        'input',
        'textfield',
        'textinput',
        'textarea',
        'picker',
        'select',
        'slider',
        'tabs',
        'tabview',
        'page-switcher',
        'dropzone',
        'file-picker',
        'filepicker',
        'media-picker',
        'slot',
        'slot-rotate',
        'embed',
      ];
      for (final tag in tags) {
        final root = viewIrFromSource('$tag onclick=evil src=x "label"').root;
        expect(root.single, isA<UnsupportedNode>(), reason: tag);
        expect((root.single as UnsupportedNode).kind, tag, reason: tag);
      }
      // Case-insensitively, too — the parser lowercases the tag.
      expect(viewIrFromSource('WebView').root.single, isA<UnsupportedNode>());
    });

    test('an unrecognised .crepus tag degrades to an inert container', () {
      // Unlike the IR decoder, the parser treats an unknown tag as a plain
      // container. That is safe (a stack has no action seam of its own beyond
      // an explicit onlongpress) but it is a real difference between the two
      // front-ends, so it is pinned rather than assumed.
      final node = viewIrFromSource('script "alert(1)"').root.single;
      expect(node, isA<StackNode>());
      expect((node as StackNode).children, isEmpty);
    });
  });

  group('actions pass through verbatim', () {
    testWidgets('button onClick reaches the host unmodified', (tester) async {
      for (final action in _hostileActions) {
        final fired = <String>[];
        await tester.pumpWidget(
          _doc([
            {'kind': 'button', 'label': 'Go', 'onClick': action},
          ], onAction: fired.add),
        );
        await tester.tap(find.byType(TextButton));
        expect(fired, [action], reason: action);
      }
    });

    testWidgets('toggle, checkbox and long-press seams are equally verbatim', (
      tester,
    ) async {
      for (final action in _hostileActions) {
        final fired = <String>[];
        await tester.pumpWidget(
          _doc([
            {'kind': 'toggle', 'label': 'T', 'onChange': action},
            {'kind': 'checkbox', 'label': 'C', 'onChange': action},
            {
              'kind': 'stack',
              'onLongPress': action,
              'children': [
                {'kind': 'text', 'content': 'LP'},
              ],
            },
          ], onAction: fired.add),
        );
        await tester.tap(find.byType(Switch));
        await tester.tap(find.byType(Checkbox));
        await tester.longPress(find.text('LP'));
        expect(fired, [action, action, action], reason: action);
      }
    });

    testWidgets('an action string is never run through interpolation', (
      tester,
    ) async {
      final fired = <String>[];
      await tester.pumpWidget(
        _doc(
          const [
            {'kind': 'button', 'label': 'Go', 'onClick': 'open:{secret}'},
          ],
          onAction: fired.add,
          data: const {'secret': 'leaked'},
        ),
      );
      await tester.tap(find.byType(TextButton));
      expect(fired, ['open:{secret}']);
    });

    testWidgets('an empty action is inert rather than dispatched as ""', (
      tester,
    ) async {
      final fired = <String>[];
      await tester.pumpWidget(
        _doc(const [
          {'kind': 'button', 'label': 'Go', 'onClick': ''},
          {
            'kind': 'stack',
            'onLongPress': '',
            'children': [
              {'kind': 'text', 'content': 'LP'},
            ],
          },
        ], onAction: fired.add),
      );
      await tester.tap(find.byType(TextButton));
      await tester.longPress(find.text('LP'));
      expect(fired, isEmpty);
    });

    testWidgets('an empty long-press action installs no gesture detector', (
      tester,
    ) async {
      await tester.pumpWidget(
        _doc(const [
          {
            'kind': 'stack',
            'onLongPress': '',
            'children': [
              {'kind': 'text', 'content': 'LP'},
            ],
          },
        ]),
      );
      expect(find.byType(GestureDetector), findsNothing);
    });

    testWidgets('a null action disables the control instead of firing null', (
      tester,
    ) async {
      final fired = <String>[];
      await tester.pumpWidget(
        _doc(const [
          {'kind': 'button', 'label': 'Go'},
          {'kind': 'toggle', 'label': 'T'},
          {'kind': 'checkbox', 'label': 'C'},
        ], onAction: fired.add),
      );
      expect(
        tester.widget<TextButton>(find.byType(TextButton)).onPressed,
        isNull,
      );
      expect(tester.widget<Switch>(find.byType(Switch)).onChanged, isNull);
      expect(tester.widget<Checkbox>(find.byType(Checkbox)).onChanged, isNull);
      await tester.tap(find.byType(TextButton));
      expect(fired, isEmpty);
    });

    testWidgets('a host with no onAction callback still cannot crash', (
      tester,
    ) async {
      await tester.pumpWidget(
        _host(
          CrepusView.fromIr(const {
            'version': 5,
            'root': [
              {'kind': 'button', 'label': 'Go', 'onClick': 'boom'},
            ],
          }),
        ),
      );
      await tester.tap(find.byType(TextButton));
      expect(tester.takeException(), isNull);
    });

    test('the IR decoder stores the action byte-for-byte', () {
      for (final action in _hostileActions) {
        final node =
            ViewNode.fromJson({
                  'kind': 'button',
                  'label': 'x',
                  'onClick': action,
                  'onLongPress': action,
                })
                as ButtonNode;
        expect(node.onClick, action);
        expect(identical(node.onClick, action), isTrue, reason: action);
        expect(node.onLongPress, action);
      }
    });

    test('an unquoted .crepus action value is not rewritten', () {
      for (final action in const [
        'prompt:do-it',
        'nav:/a/b?c=d&e=f',
        'javascript:alert(1)',
        '../../traversal',
        'a&b=c;d|e',
      ]) {
        final node =
            viewIrFromSource('button "Go" onclick=$action').root.single
                as ButtonNode;
        expect(node.onClick, action, reason: action);
      }
    });

    test(
      'a braced .crepus action keeps its interior verbatim bar trimming',
      () {
        final node =
            viewIrFromSource(
                  'button "Go" onclick={ prompt:do "it" now }',
                ).root.single
                as ButtonNode;
        expect(node.onClick, 'prompt:do "it" now');
      },
    );

    test('a quoted .crepus action is a string literal, so it unescapes', () {
      // Documented, not a rewrite of the action: `"..."` is a source-level
      // string literal and honours the usual escapes. Hosts that need an
      // opaque action should use the unquoted or `{...}` form.
      final node =
          viewIrFromSource(r'button "Go" onclick="a\nb\\c\"d"').root.single
              as ButtonNode;
      expect(node.onClick, 'a\nb\\c"d');
    });
  });

  group('caps hold exactly at their boundaries', () {
    test('the documented default caps are 60 nodes / depth 8 / 8000 chars', () {
      const limits = CrepusLimits.defaults;
      expect(limits.maxNodes, 60);
      expect(limits.maxDepth, 8);
      expect(limits.maxSourceLength, 8000);
    });

    test('60 nodes pass and 61 do not', () {
      List<Map<String, Object?>> flat(int n) =>
          List.generate(n, (i) => {'kind': 'text', 'content': '$i'});
      expect(ViewIr.fromJson({'root': flat(59)}).root, hasLength(59));
      expect(ViewIr.fromJson({'root': flat(60)}).root, hasLength(60));
      expect(ViewIr.fromJson({'root': flat(61)}).root, isEmpty);
      expect(
        crepusExceedsLimits(
          ViewIr.fromJson({'root': flat(60)}).root,
          CrepusLimits.defaults,
        ),
        isFalse,
      );
    });

    test('the node cap counts nested and unsupported nodes alike', () {
      // 30 stacks each holding one unsupported child = 60 nodes exactly.
      List<Map<String, Object?>> pairs(int n) => List.generate(n, (i) {
        return {
          'kind': 'stack',
          'children': [
            {'kind': 'webView'},
          ],
        };
      });
      expect(ViewIr.fromJson({'root': pairs(30)}).root, hasLength(30));
      expect(ViewIr.fromJson({'root': pairs(31)}).root, isEmpty);
    });

    test('depth 8 passes and depth 9 does not', () {
      expect(
        ViewIr.fromJson({
          'root': [_nest(8)],
        }).root,
        hasLength(1),
      );
      expect(
        ViewIr.fromJson({
          'root': [_nest(9)],
        }).root,
        isEmpty,
      );
    });

    test('an if/forEach branch counts toward depth like any other child', () {
      // if(1) > then stack(2) > forEach(3) > listItem(4) > list(5) >
      // listItem(6) > stack(7) > text(8) — exactly at the cap.
      Map<String, Object?> atCap = const {'kind': 'text', 'content': 'x'};
      atCap = {
        'kind': 'stack',
        'children': [atCap],
      };
      atCap = {
        'kind': 'listItem',
        'children': [atCap],
      };
      atCap = {
        'kind': 'list',
        'children': [atCap],
      };
      atCap = {
        'kind': 'listItem',
        'children': [atCap],
      };
      atCap = {
        'kind': 'forEach',
        'bind': 'xs',
        'itemBody': [atCap],
      };
      atCap = {
        'kind': 'stack',
        'children': [atCap],
      };
      final ok = {
        'kind': 'if',
        'condition': 'c',
        'thenChildren': [atCap],
      };
      expect(
        ViewIr.fromJson({
          'root': [ok],
        }).root,
        hasLength(1),
      );
      final tooDeep = {
        'kind': 'if',
        'condition': 'c',
        'thenChildren': [
          {
            'kind': 'stack',
            'children': [ok],
          },
        ],
      };
      expect(
        ViewIr.fromJson({
          'root': [tooDeep],
        }).root,
        isEmpty,
      );
    });

    test('an if node counts both branches toward the node cap', () {
      List<Map<String, Object?>> leaves(int n) =>
          List.generate(n, (i) => {'kind': 'text', 'content': '$i'});
      Map<String, Object?> ifWith(int then, int els) => {
        'kind': 'if',
        'condition': 'c',
        'thenChildren': leaves(then),
        'elseChildren': leaves(els),
      };
      // 1 (the if) + 30 + 29 = 60.
      expect(
        ViewIr.fromJson({
          'root': [ifWith(30, 29)],
        }).root,
        hasLength(1),
      );
      expect(
        ViewIr.fromJson({
          'root': [ifWith(30, 30)],
        }).root,
        isEmpty,
      );
    });

    test('a source of exactly maxSourceLength parses; one more does not', () {
      String src(int length) {
        final body = 'a' * (length - 7);
        return 'text "$body"';
      }

      expect(src(8000), hasLength(8000));
      expect(viewIrFromSource(src(8000)).root, hasLength(1));
      expect(viewIrFromSource(src(8001)).root, isEmpty);
      // The version is still reported on a rejected document.
      expect(viewIrFromSource(src(8001)).version, IR_VERSION);
    });

    test('custom limits are honoured at their own boundaries', () {
      const tight = CrepusLimits(maxNodes: 2, maxDepth: 2);
      const shortSource = CrepusLimits(maxSourceLength: 12);
      expect(
        viewIrFromSource('text "a"\ntext "b"', limits: tight).root,
        hasLength(2),
      );
      expect(
        viewIrFromSource('text "a"\ntext "b"\ntext "c"', limits: tight).root,
        isEmpty,
      );
      expect(
        viewIrFromSource('view\n  text "a"', limits: tight).root,
        hasLength(1),
      );
      expect(
        viewIrFromSource('view\n  view\n    text "a"', limits: tight).root,
        isEmpty,
      );
      expect(
        viewIrFromSource('text "abcde"', limits: shortSource).root,
        hasLength(1),
      );
      expect(
        viewIrFromSource('text "abcdef"', limits: shortSource).root,
        isEmpty,
      );
    });

    testWidgets('an over-cap document renders nothing but still builds', (
      tester,
    ) async {
      await tester.pumpWidget(
        _host(
          CrepusView.fromIr({
            'version': 5,
            'root': List.generate(
              200,
              (i) => {'kind': 'text', 'content': 'node$i'},
            ),
          }),
        ),
      );
      expect(tester.takeException(), isNull);
      expect(find.text('node0'), findsNothing);
      expect(find.byType(Text), findsNothing);
    });
  });
}
