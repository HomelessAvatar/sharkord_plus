declare module '@vencord/venmic' {
  export type Node = Record<string, string | number | undefined>;
  export class PatchBay {
    static hasPipeWire(): boolean;
    link(rules: unknown): boolean;
    unlink(): void;
  }
}
