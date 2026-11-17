import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:sharkord_mobile/core/theme/colors.dart';
import 'package:sharkord_mobile/core/theme/theme.dart';

void main() {
  group('SharkordColors Pure Neutral Dark Palette', () {
    test('defines correct color tokens', () {
      expect(SharkordColors.background, const Color(0xFF0A0B0E));
      expect(SharkordColors.sidebar, const Color(0xFF111216));
      expect(SharkordColors.surface, const Color(0xFF16171D));
      expect(SharkordColors.card, const Color(0xFF1C1E24));
      expect(SharkordColors.border, const Color(0xFF262833));
      expect(SharkordColors.primary, const Color(0xFF3B82F6));
      expect(SharkordColors.speaking, const Color(0xFF22C55E));
      expect(SharkordColors.danger, const Color(0xFFEF4444));
      expect(SharkordColors.textPrimary, const Color(0xFFF3F4F6));
      expect(SharkordColors.textMuted, const Color(0xFF9CA3AF));
    });
  });

  group('SharkordTheme', () {
    test('configures dark theme with Pure Neutral Dark tokens', () {
      final theme = SharkordTheme.darkTheme;

      expect(theme.brightness, Brightness.dark);
      expect(theme.scaffoldBackgroundColor, SharkordColors.background);
      expect(theme.cardColor, SharkordColors.card);
      expect(theme.dividerColor, SharkordColors.border);

      expect(theme.colorScheme.brightness, Brightness.dark);
      expect(theme.colorScheme.primary, SharkordColors.primary);
      expect(theme.colorScheme.surface, SharkordColors.surface);
      expect(theme.colorScheme.error, SharkordColors.danger);
    });
  });
}
