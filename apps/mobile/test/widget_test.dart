import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:sharkord_mobile/main.dart';

void main() {
  testWidgets('SharkordApp smoke test', (WidgetTester tester) async {
    await tester.pumpWidget(const ProviderScope(child: SharkordApp()));

    expect(find.text('Sharkord Mobile'), findsOneWidget);
  });
}
