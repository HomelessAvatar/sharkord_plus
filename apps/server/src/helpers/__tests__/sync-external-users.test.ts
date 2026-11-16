import { ServerEvents } from '@sharkord/shared';
import { afterEach, beforeEach, describe, expect, spyOn, test } from 'bun:test';
import { eq } from 'drizzle-orm';
import { initTest } from '../../__tests__/helpers';
import { db } from '../../db';
import { users } from '../../db/schema';
import { pubsub } from '../../utils/pubsub';
import {
  checkAndSyncExternalUsers,
  initUserSnapshots,
  resetExternalUsersSyncState
} from '../sync-external-users';

describe('external users sync', () => {
  beforeEach(async () => {
    resetExternalUsersSyncState();
    await initUserSnapshots();
  });

  afterEach(() => {
    resetExternalUsersSyncState();
  });

  test('should throw when user lacks MANAGE_USERS permission on trpc.users.sync', async () => {
    const { caller } = await initTest(2);

    await expect(caller.users.sync()).rejects.toThrow(
      'Insufficient permissions'
    );
  });

  test('should succeed when admin calls trpc.users.sync', async () => {
    const { caller } = await initTest(1);

    const result = await caller.users.sync();
    expect(result).toEqual({ success: true });
  });

  test('should detect direct database update and broadcast USER_UPDATE event', async () => {
    const publishSpy = spyOn(pubsub, 'publish');

    // Simulate an external script or website modifying SQLite directly
    await db
      .update(users)
      .set({ name: 'ExternallyRenamed' })
      .where(eq(users.id, 2))
      .run();

    await checkAndSyncExternalUsers(true);

    expect(publishSpy).toHaveBeenCalledWith(
      ServerEvents.USER_UPDATE,
      expect.objectContaining({
        id: 2,
        name: 'ExternallyRenamed'
      })
    );

    publishSpy.mockRestore();
  });
});
