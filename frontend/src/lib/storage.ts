export interface JbodMount {
  id: number;
  name: string;
  mountPath: string;
  filesystem: string;
  description: string;
  driveIds: string[];
}

export const sampleJbodMounts: JbodMount[] = [
  {
    id: 1,
    name: 'Media',
    mountPath: '/mnt/media',
    filesystem: 'xfs',
    description: 'Bulk media storage mounted directly on the host.',
    driveIds: ['sda'],
  },
];

export function assignedDriveIds(jbodMounts: JbodMount[]) {
  return new Set(jbodMounts.flatMap((mount) => mount.driveIds));
}
