import { Permission } from '@sharkord/shared';
import { z } from 'zod';
import { checkAndSyncExternalUsers } from '../../helpers/sync-external-users';
import { protectedProcedure } from '../../utils/trpc';

const syncUsersRoute = protectedProcedure
  .input(z.object({}).optional())
  .mutation(async ({ ctx }) => {
    await ctx.needsPermission(Permission.MANAGE_USERS);
    await checkAndSyncExternalUsers(true);

    return { success: true };
  });

export { syncUsersRoute };
