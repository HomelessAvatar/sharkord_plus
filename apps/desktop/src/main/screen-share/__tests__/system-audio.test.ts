import { describe, expect, mock, test } from 'bun:test';

mock.module('electron', () => ({
  default: {
    app: {
      getPath: () => '',
      getVersion: () => '1.0.0',
      getAppMetrics: () => []
    }
  },
  app: {
    getPath: () => '',
    getVersion: () => '1.0.0',
    getAppMetrics: () => []
  }
}));

const { createAudioLinkRules } = await import('../system-audio');

describe('createAudioLinkRules', () => {
  test('creates rules targeted to a specific process ID when targetProcessId is provided', () => {
    const rules = createAudioLinkRules({
      allOutputs: false,
      targetProcessId: 1234,
      ownAudioNodes: [{ 'application.process.binary': 'sharkord' }]
    });

    expect(rules.include).toEqual([{ 'application.process.id': '1234' }]);
    expect(rules.exclude).toEqual([
      { 'application.process.binary': 'sharkord' },
      { 'media.class': 'Stream/Input/Audio' }
    ]);
    expect(rules.only_speakers).toBe(false);
    expect(rules.only_default_speakers).toBe(false);
    expect(rules.ignore_devices).toBe(true);
    expect(rules.mute).toBe(false);
  });

  test('creates default speaker rules when targetProcessId is not provided and allOutputs is false', () => {
    const rules = createAudioLinkRules({
      allOutputs: false,
      ownAudioNodes: [{ 'application.process.binary': 'sharkord' }]
    });

    expect(rules.include).toEqual([]);
    expect(rules.only_speakers).toBe(true);
    expect(rules.only_default_speakers).toBe(true);
    expect(rules.exclude).toEqual([
      { 'application.process.binary': 'sharkord' },
      { 'media.class': 'Stream/Input/Audio' }
    ]);
  });

  test('creates all-output rules when targetProcessId is not provided and allOutputs is true', () => {
    const rules = createAudioLinkRules({
      allOutputs: true,
      ownAudioNodes: []
    });

    expect(rules.include).toEqual([
      { 'media.class': 'Stream/Output/Audio', 'node.link-group': '' }
    ]);
    expect(rules.only_speakers).toBe(false);
    expect(rules.only_default_speakers).toBe(false);
    expect(rules.exclude).toEqual([{ 'media.class': 'Stream/Input/Audio' }]);
  });
});
