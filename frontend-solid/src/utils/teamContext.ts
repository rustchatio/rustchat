import { client } from '@/api/client';
import { channelStore } from '@/stores/channels';

interface TeamSummary {
  id: string;
  name?: string;
  display_name?: string | null;
}

function normalizeTeams(payload: unknown): TeamSummary[] {
  if (!Array.isArray(payload)) {
    return [];
  }

  return payload.filter((team): team is TeamSummary => {
    const candidate = team as Partial<TeamSummary> | null;
    return typeof candidate?.id === 'string';
  });
}

async function fetchTeams(): Promise<TeamSummary[]> {
  const response = await client.get<TeamSummary[]>('/teams');
  return normalizeTeams(response.data);
}

export async function resolveActiveTeamId(): Promise<string | null> {
  const fromCurrentChannel = channelStore.currentChannel()?.team_id;
  if (fromCurrentChannel) {
    return fromCurrentChannel;
  }

  const fromLoadedChannels = channelStore.channels.find(
    (channel) =>
      channel.channel_type === 'public' || channel.channel_type === 'private'
  )?.team_id;
  if (fromLoadedChannels) {
    return fromLoadedChannels;
  }

  let teams = await fetchTeams();
  if (teams.length === 0) {
    await client.get('/auth/me');
    teams = await fetchTeams();
  }

  if (teams.length === 0) {
    return null;
  }

  const preferred =
    teams.find((team) => team.name?.toLowerCase() === 'rustchat') || teams[0];
  return preferred?.id ?? null;
}
