import { sql } from 'drizzle-orm';
import { db } from '../db';
import { publishUser } from '../db/publishers';
import { users } from '../db/schema';
import { logger } from '../logger';

type TUserSnapshot = {
  name: string;
  profileColor: string;
  bio: string | null;
  avatarId: number | null;
  bannerId: number | null;
  banned: boolean;
};

const SYNC_INTERVAL_MS = 2500;

let lastDataVersion: number | null = null;
const userSnapshots = new Map<number, TUserSnapshot>();
let isInitialized = false;
let syncTimer: ReturnType<typeof setInterval> | null = null;

const initUserSnapshots = async () => {
  try {
    const versionRow = await db.get<{ data_version: number }>(
      sql`PRAGMA data_version;`
    );
    lastDataVersion = versionRow?.data_version ?? 0;

    const allUsers = await db
      .select({
        id: users.id,
        name: users.name,
        profileColor: users.profileColor,
        bio: users.bio,
        avatarId: users.avatarId,
        bannerId: users.bannerId,
        banned: users.banned
      })
      .from(users)
      .all();

    userSnapshots.clear();
    for (const u of allUsers) {
      userSnapshots.set(u.id, {
        name: u.name,
        profileColor: u.profileColor,
        bio: u.bio,
        avatarId: u.avatarId,
        bannerId: u.bannerId,
        banned: u.banned
      });
    }

    isInitialized = true;
  } catch (error) {
    logger.error('Failed to initialize user snapshots for external sync', {
      error
    });
  }
};

const checkAndSyncExternalUsers = async (force = false) => {
  if (!isInitialized) {
    await initUserSnapshots();
    return;
  }

  try {
    const versionRow = await db.get<{ data_version: number }>(
      sql`PRAGMA data_version;`
    );
    const currentVersion = versionRow?.data_version ?? 0;

    if (
      !force &&
      lastDataVersion !== null &&
      currentVersion === lastDataVersion
    ) {
      return;
    }

    lastDataVersion = currentVersion;

    const allUsers = await db
      .select({
        id: users.id,
        name: users.name,
        profileColor: users.profileColor,
        bio: users.bio,
        avatarId: users.avatarId,
        bannerId: users.bannerId,
        banned: users.banned
      })
      .from(users)
      .all();

    for (const u of allUsers) {
      const prev = userSnapshots.get(u.id);

      if (!prev) {
        userSnapshots.set(u.id, {
          name: u.name,
          profileColor: u.profileColor,
          bio: u.bio,
          avatarId: u.avatarId,
          bannerId: u.bannerId,
          banned: u.banned
        });
        await publishUser(u.id, 'create');
      } else if (
        prev.name !== u.name ||
        prev.profileColor !== u.profileColor ||
        prev.bio !== u.bio ||
        prev.avatarId !== u.avatarId ||
        prev.bannerId !== u.bannerId ||
        prev.banned !== u.banned
      ) {
        userSnapshots.set(u.id, {
          name: u.name,
          profileColor: u.profileColor,
          bio: u.bio,
          avatarId: u.avatarId,
          bannerId: u.bannerId,
          banned: u.banned
        });
        await publishUser(u.id, 'update');
      }
    }
  } catch (error) {
    logger.error('Error during external user sync check', { error });
  }
};

const startExternalUsersWatcher = () => {
  if (syncTimer) return;
  void initUserSnapshots();
  syncTimer = setInterval(() => {
    void checkAndSyncExternalUsers();
  }, SYNC_INTERVAL_MS);
  syncTimer.unref?.();
};

const stopExternalUsersWatcher = () => {
  if (syncTimer) {
    clearInterval(syncTimer);
    syncTimer = null;
  }
};

const resetExternalUsersSyncState = () => {
  stopExternalUsersWatcher();
  userSnapshots.clear();
  lastDataVersion = null;
  isInitialized = false;
};

export {
  checkAndSyncExternalUsers,
  initUserSnapshots,
  resetExternalUsersSyncState,
  startExternalUsersWatcher,
  stopExternalUsersWatcher
};
