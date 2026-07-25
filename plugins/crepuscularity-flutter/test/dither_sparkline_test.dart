import 'package:crepuscularity_flutter/crepuscularity_flutter.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('parseSparklineValues accepts comma and space separated lists', () {
    expect(parseSparklineValues('1,2,3'), [1, 2, 3]);
    expect(parseSparklineValues('1 2 3 4'), [1, 2, 3, 4]);
    expect(parseSparklineValues('[10,20,30]'), [10, 20, 30]);
  });

  testWidgets('sparkline crepus node renders full-width dither chart', (
    tester,
  ) async {
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: CrepusView.fromSource('''
sparkline color=green variant=gradient values=2,4,3,8,6,9
'''),
        ),
      ),
    );
    expect(find.byType(DitherSparkline), findsOneWidget);
  });
}
