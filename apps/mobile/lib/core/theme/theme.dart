import 'package:flutter/material.dart';
import 'package:sharkord_mobile/core/theme/colors.dart';

abstract final class SharkordTheme {
  static ThemeData get darkTheme {
    return ThemeData(
      useMaterial3: true,
      brightness: Brightness.dark,
      scaffoldBackgroundColor: SharkordColors.background,
      cardColor: SharkordColors.card,
      dividerColor: SharkordColors.border,
      colorScheme: const ColorScheme.dark(
        primary: SharkordColors.primary,
        surface: SharkordColors.surface,
        error: SharkordColors.danger,
        onPrimary: Colors.white,
        onSurface: SharkordColors.textPrimary,
        onError: Colors.white,
      ),
      appBarTheme: const AppBarTheme(
        backgroundColor: SharkordColors.surface,
        foregroundColor: SharkordColors.textPrimary,
        elevation: 0,
        scrolledUnderElevation: 0,
      ),
      textTheme: const TextTheme(
        bodyLarge: TextStyle(color: SharkordColors.textPrimary),
        bodyMedium: TextStyle(color: SharkordColors.textPrimary),
        bodySmall: TextStyle(color: SharkordColors.textMuted),
        titleLarge: TextStyle(color: SharkordColors.textPrimary, fontWeight: FontWeight.w600),
        titleMedium: TextStyle(color: SharkordColors.textPrimary, fontWeight: FontWeight.w600),
        titleSmall: TextStyle(color: SharkordColors.textMuted),
      ),
    );
  }
}
