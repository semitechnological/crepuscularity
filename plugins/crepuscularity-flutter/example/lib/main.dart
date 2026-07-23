import 'package:crepuscularity_flutter/crepuscularity_flutter.dart';
import 'package:flutter/material.dart';

void main() => runApp(const ExampleApp());

const _source = '''
stack col gap-3 p-4
  text font-semibold text-lg "What matters next"
  text text-sm "Follow up on the invoice you flagged yesterday."
  progress label="Setup" value=60 max=100
  stack row gap-2
    button "Do it now" onclick=prompt:Draft an invoice follow-up
    button "Done" onclick=complete
''';

class ExampleApp extends StatelessWidget {
  const ExampleApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'crepuscularity_flutter',
      home: Scaffold(
        appBar: AppBar(title: const Text('crepuscularity_flutter')),
        body: Padding(
          padding: const EdgeInsets.all(16),
          child: CrepusView.fromSource(
            _source,
            onAction: (action) => debugPrint('action: $action'),
          ),
        ),
      ),
    );
  }
}
