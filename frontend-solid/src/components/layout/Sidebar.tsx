// ============================================
// Sidebar - Left Navigation Sidebar
// ============================================

import { Show, createSignal, For, createMemo, createEffect } from 'solid-js';
import { A, useNavigate } from '@solidjs/router';
import {
  channelStore,
  fetchChannels,
  fetchJoinableChannels,
  joinChannel,
  selectChannel,
  createChannel,
  type Channel,
} from '@/stores/channels';
import { authStore } from '@/stores/auth';
import { uiStore } from '@/stores/ui';
import { unreadStore } from '@/stores/unreads';
import { cn } from '@/utils/cn';
import { isAdminRole } from '@/utils/roles';
import Modal from '@/components/ui/Modal';
import Button from '@/components/ui/Button';
import { client, getErrorMessage } from '@/api/client';

// Icons
import {
  HiOutlineChevronDown,
  HiOutlineChevronRight,
  HiOutlinePlus,
  HiOutlineHashtag,
  HiOutlineLockClosed,
  HiOutlineUser,
  HiOutlineStar,
  HiOutlineCog6Tooth,
  HiOutlineBookOpen,
  HiOutlineBars3,
  HiOutlineXMark,
} from 'solid-icons/hi';

// ============================================
// Props Interface
// ============================================

export interface SidebarProps {
  isMobile?: boolean;
}

interface TeamOption {
  id: string;
  name: string;
  display_name?: string | null;
}

interface DirectMessageCandidate {
  id: string;
  username: string;
  display_name?: string | null;
  avatar_url?: string | null;
}

const DEFAULT_TEAM_NAME = 'rustchat';
const DEFAULT_TEAM_DISPLAY_NAME = 'RustChat';

function normalizeTeams(payload: unknown): TeamOption[] {
  if (!Array.isArray(payload)) return [];
  return payload.filter((team): team is TeamOption => {
    const candidate = team as Partial<TeamOption> | null;
    return (
      typeof candidate?.id === 'string' &&
      typeof candidate?.name === 'string'
    );
  });
}

async function fetchTeamsWithBootstrap(): Promise<TeamOption[]> {
  const requestTeams = async () => {
    const response = await client.get<TeamOption[]>('/teams');
    return normalizeTeams(response.data);
  };

  let teams = await requestTeams();
  if (teams.length > 0) {
    return teams;
  }

  await client.get('/auth/me');
  teams = await requestTeams();
  if (teams.length > 0) {
    return teams;
  }

  // Recovery path: user has no current team memberships.
  // 1) Join an open public team if one exists (prefer "rustchat").
  try {
    const publicResponse = await client.get<TeamOption[]>('/teams/public');
    const publicTeams = normalizeTeams(publicResponse.data);
    const joinTarget =
      publicTeams.find((team) => team.name.toLowerCase() === DEFAULT_TEAM_NAME) ||
      publicTeams[0];

    if (joinTarget) {
      await client.post(`/teams/${joinTarget.id}/join`);
      teams = await requestTeams();
      if (teams.length > 0) {
        return teams;
      }
    }
  } catch (error) {
    console.warn('Failed to auto-join public team during bootstrap', error);
  }

  // 2) If still empty, create a default workspace team for this user.
  try {
    await client.post('/teams', {
      name: DEFAULT_TEAM_NAME,
      display_name: DEFAULT_TEAM_DISPLAY_NAME,
      description: 'Default RustChat workspace',
    });
  } catch (error) {
    const conflictName = `${DEFAULT_TEAM_NAME}-${Date.now().toString(36)}`;
    try {
      await client.post('/teams', {
        name: conflictName,
        display_name: DEFAULT_TEAM_DISPLAY_NAME,
        description: 'Default RustChat workspace',
      });
    } catch (fallbackError) {
      // Ignore create conflicts/races and continue with a final refresh.
      console.warn('Failed to auto-create default team during bootstrap', error, fallbackError);
    }
  }

  teams = await requestTeams();
  return teams;
}

function toChannelName(displayName: string): string {
  const normalized = displayName
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9\-_ ]+/g, '')
    .replace(/\s+/g, '-')
    .replace(/-+/g, '-')
    .replace(/^-|-$/g, '');

  return normalized || 'new-channel';
}

function toTeamName(displayName: string): string {
  const normalized = displayName
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9\-_ ]+/g, '')
    .replace(/\s+/g, '-')
    .replace(/-+/g, '-')
    .replace(/^-|-$/g, '');

  return normalized || 'rustchat-team';
}

// ============================================
// Sidebar Component
// ============================================

export function Sidebar(props: SidebarProps) {
  const navigate = useNavigate();
  const isCollapsed = () => uiStore.preferences.sidebarCollapsed && !props.isMobile;
  const canAccessAdmin = () => isAdminRole(authStore.user()?.role);

  const currentChannelId = () => channelStore.currentChannelId();
  const prefs = () => uiStore.preferences;

  // Section expansion states
  const isExpanded = (section: 'favorites' | 'channels' | 'directMessages') =>
    prefs().channelSectionsExpanded[section];

  // Filter channels by type
  const favoriteChannels = () => channelStore.favoriteChannels();
  const publicChannels = () => channelStore.publicChannels();
  const privateChannels = () => channelStore.privateChannels();
  const directMessages = () => channelStore.directMessages();

  const [isCreateChannelOpen, setIsCreateChannelOpen] = createSignal(false);
  const [isCreatingChannel, setIsCreatingChannel] = createSignal(false);
  const [createChannelError, setCreateChannelError] = createSignal<string | null>(null);
  const [channelDisplayName, setChannelDisplayName] = createSignal('');
  const [channelPurpose, setChannelPurpose] = createSignal('');
  const [channelType, setChannelType] = createSignal<'public' | 'private'>('public');
  const [channelTeams, setChannelTeams] = createSignal<TeamOption[]>([]);
  const [isLoadingChannelTeams, setIsLoadingChannelTeams] = createSignal(false);
  const [selectedTeamId, setSelectedTeamId] = createSignal('');
  const [isCreateDmOpen, setIsCreateDmOpen] = createSignal(false);
  const [isLoadingDmUsers, setIsLoadingDmUsers] = createSignal(false);
  const [isCreatingDm, setIsCreatingDm] = createSignal(false);
  const [dmUsers, setDmUsers] = createSignal<DirectMessageCandidate[]>([]);
  const [dmSearchQuery, setDmSearchQuery] = createSignal('');
  const [dmSelectedUserId, setDmSelectedUserId] = createSignal('');
  const [dmTeamId, setDmTeamId] = createSignal('');
  const [dmError, setDmError] = createSignal<string | null>(null);

  const loadChannelTeams = async () => {
    const safeTeams = await fetchTeamsWithBootstrap();
    setChannelTeams(safeTeams);
    if (!selectedTeamId() && safeTeams.length > 0) {
      const activeTeamId = channelStore.currentChannel()?.team_id;
      setSelectedTeamId(activeTeamId && safeTeams.some((team) => team.id === activeTeamId) ? activeTeamId : safeTeams[0]!.id);
    }
  };

  const loadDirectMessageUsers = async (teamId: string) => {
    if (!teamId) {
      setDmUsers([]);
      return;
    }

    const response = await client.get<Array<{ user_id?: unknown; username?: unknown; display_name?: unknown; avatar_url?: unknown }>>(
      `/teams/${teamId}/members`
    );
    const payload = response.data;
    const safeUsers = Array.isArray(payload)
      ? payload
        .filter(
          (
            user
          ): user is { user_id: string; username: string; display_name?: string | null; avatar_url?: string | null } =>
            typeof user?.user_id === 'string' && typeof user?.username === 'string'
        )
        .map((user) => ({
          id: user.user_id,
          username: user.username,
          display_name: user.display_name || null,
          avatar_url: user.avatar_url || null,
        }))
      : [];

    const currentUserId = authStore.user()?.id;
    setDmUsers(safeUsers.filter((user) => user.id !== currentUserId));
  };

  const openCreateChannelModal = () => {
    setCreateChannelError(null);
    setChannelDisplayName('');
    setChannelPurpose('');
    setChannelType('public');
    setSelectedTeamId(channelStore.currentChannel()?.team_id || '');
    setIsCreateChannelOpen(true);
    setIsLoadingChannelTeams(true);
    void loadChannelTeams()
      .catch((error) => {
        setCreateChannelError(getErrorMessage(error) || 'Failed to load teams');
      })
      .finally(() => {
        setIsLoadingChannelTeams(false);
      });
  };

  const submitCreateChannel = async () => {
    const displayName = channelDisplayName().trim();
    if (!displayName) {
      setCreateChannelError('Channel name is required.');
      return;
    }

    if (!selectedTeamId()) {
      setCreateChannelError('Select a team first.');
      return;
    }

    setIsCreatingChannel(true);
    setCreateChannelError(null);
    try {
      const channel = await createChannel({
        team_id: selectedTeamId(),
        name: toChannelName(displayName),
        display_name: displayName,
        channel_type: channelType(),
        purpose: channelPurpose().trim() || undefined,
      });

      await fetchChannels(channel.team_id);
      selectChannel(channel.id);
      navigate(`/channels/${channel.id}`);
      setIsCreateChannelOpen(false);
      uiStore.setMobileSidebarOpen(false);
    } catch (error) {
      setCreateChannelError(error instanceof Error ? error.message : 'Failed to create channel.');
    } finally {
      setIsCreatingChannel(false);
    }
  };

  const openCreateDirectMessageModal = () => {
    setDmError(null);
    setDmSearchQuery('');
    setDmUsers([]);
    setDmSelectedUserId('');
    setDmTeamId(channelStore.currentChannel()?.team_id || selectedTeamId() || '');
    setIsCreateDmOpen(true);
    setDmError(null);
    void fetchTeamsWithBootstrap()
      .then((teams) => {
        setChannelTeams(teams);
        const preferredTeamId = dmTeamId() || teams[0]?.id || '';
        setDmTeamId(preferredTeamId);
      })
      .catch((error) => {
        setDmError(getErrorMessage(error) || 'Failed to load teams for direct messages.');
      });
  };

  createEffect(() => {
    if (!isCreateDmOpen()) return;
    const teamId = dmTeamId();
    if (!teamId) {
      setDmUsers([]);
      return;
    }

    setIsLoadingDmUsers(true);
    setDmError(null);
    void loadDirectMessageUsers(teamId)
      .catch((error) => {
        setDmError(getErrorMessage(error) || 'Failed to load team members.');
      })
      .finally(() => {
        setIsLoadingDmUsers(false);
      });
  });

  const filteredDmUsers = createMemo(() => {
    const query = dmSearchQuery().trim().toLowerCase();
    if (!query) return dmUsers();
    return dmUsers().filter((user) => {
      const username = user.username.toLowerCase();
      const displayName = (user.display_name || '').toLowerCase();
      return username.includes(query) || displayName.includes(query);
    });
  });

  createEffect(() => {
    const options = filteredDmUsers();
    if (options.length === 0) {
      return;
    }
    if (!options.some((user) => user.id === dmSelectedUserId())) {
      setDmSelectedUserId(options[0]!.id);
    }
  });

  const submitCreateDirectMessage = async () => {
    const selectedUserId = dmSelectedUserId();
    if (!selectedUserId) {
      setDmError('Select a teammate to start a direct message.');
      return;
    }
    if (!dmTeamId()) {
      setDmError('Select a team first.');
      return;
    }

    setIsCreatingDm(true);
    setDmError(null);
    try {
      const selectedUser = dmUsers().find((user) => user.id === selectedUserId);
      const channel = await createChannel({
        team_id: dmTeamId(),
        name: '',
        display_name: selectedUser?.display_name || selectedUser?.username || 'Direct Message',
        channel_type: 'direct',
        target_user_id: selectedUserId,
      });

      await fetchChannels(channel.team_id);
      selectChannel(channel.id);
      navigate(`/channels/${channel.id}`);
      setIsCreateDmOpen(false);
      uiStore.setMobileSidebarOpen(false);
    } catch (error) {
      setDmError(getErrorMessage(error) || 'Failed to start direct message.');
    } finally {
      setIsCreatingDm(false);
    }
  };

  return (
    <aside
      class={cn(
        'h-full bg-bg-surface-1 border-r border-border-1 flex flex-col shrink-0 transition-all duration-200',
        props.isMobile ? 'w-[280px]' : isCollapsed() ? 'w-16' : 'w-[260px]'
      )}
    >
      {/* Sidebar Header */}
      <div class="h-14 border-b border-border-1 flex items-center px-3 shrink-0">
        <Show
          when={!isCollapsed()}
          fallback={
            <button
              type="button"
              class="p-2 rounded-lg text-text-2 hover:bg-bg-surface-2 hover:text-text-1 transition-colors mx-auto"
              onClick={() => uiStore.toggleSidebar()}
              aria-label="Expand sidebar"
              title="Expand sidebar (Ctrl+B)"
            >
              <HiOutlineBars3 size={20} />
            </button>
          }
        >
          <div class="flex items-center justify-between w-full">
            {/* Team Selector */}
            <TeamSelector />

            <Show when={!props.isMobile}>
              <button
                type="button"
                class="p-1.5 rounded-lg text-text-3 hover:bg-bg-surface-2 hover:text-text-1 transition-colors"
                onClick={() => uiStore.toggleSidebar()}
                aria-label="Collapse sidebar"
                title="Collapse sidebar (Ctrl+B)"
              >
                <HiOutlineXMark size={18} />
              </button>
            </Show>
          </div>
        </Show>
      </div>

      {/* Channel List */}
      <div class="flex-1 overflow-y-auto custom-scrollbar py-2">
        <Show when={!isCollapsed()}>
          {/* Favorites Section */}
          <Show when={favoriteChannels().length > 0}>
            <ChannelSection
              title="Favorites"
              icon={<HiOutlineStar size={16} />}
              isExpanded={isExpanded('favorites')}
              onToggle={() => uiStore.toggleChannelSection('favorites')}
            >
              <For each={favoriteChannels()}>
                {(channel) => (
                  <ChannelItem
                    channel={channel}
                    isActive={currentChannelId() === channel.id}
                    onClick={() => handleChannelClick(channel.id)}
                  />
                )}
              </For>
            </ChannelSection>
          </Show>

          {/* Channels Section */}
          <ChannelSection
            title="Channels"
            icon={<HiOutlineHashtag size={16} />}
            isExpanded={isExpanded('channels')}
            onToggle={() => uiStore.toggleChannelSection('channels')}
            action={
              <button
                type="button"
                class="p-1 rounded text-text-3 hover:bg-bg-surface-2 hover:text-text-1 opacity-0 group-hover:opacity-100 transition-opacity"
                aria-label="Create channel"
                title="Create channel"
                onClick={() => {
                  void openCreateChannelModal();
                }}
              >
                <HiOutlinePlus size={14} />
              </button>
            }
          >
            {/* Public Channels */}
            <For each={publicChannels()}>
              {(channel) => (
                <ChannelItem
                  channel={channel}
                  isActive={currentChannelId() === channel.id}
                  onClick={() => handleChannelClick(channel.id)}
                />
              )}
            </For>

            {/* Private Channels */}
            <For each={privateChannels()}>
              {(channel) => (
                <ChannelItem
                  channel={channel}
                  isActive={currentChannelId() === channel.id}
                  onClick={() => handleChannelClick(channel.id)}
                  isPrivate
                />
              )}
            </For>
          </ChannelSection>

          {/* Direct Messages Section */}
          <ChannelSection
            title="Direct Messages"
            icon={<HiOutlineUser size={16} />}
            isExpanded={isExpanded('directMessages')}
            onToggle={() => uiStore.toggleChannelSection('directMessages')}
            action={
              <button
                type="button"
                class="p-1 rounded text-text-3 hover:bg-bg-surface-2 hover:text-text-1 opacity-0 group-hover:opacity-100 transition-opacity"
                aria-label="Start direct message"
                title="Start direct message"
                onClick={() => {
                  openCreateDirectMessageModal();
                }}
              >
                <HiOutlinePlus size={14} />
              </button>
            }
          >
            <For each={directMessages()}>
              {(channel) => (
                <ChannelItem
                  channel={channel}
                  isActive={currentChannelId() === channel.id}
                  onClick={() => handleChannelClick(channel.id)}
                  isDirectMessage
                />
              )}
            </For>
          </ChannelSection>
        </Show>

        {/* Collapsed State - Just Icons */}
        <Show when={isCollapsed() && !props.isMobile}>
          <div class="flex flex-col items-center gap-1 py-2">
            <For each={publicChannels().slice(0, 5)}>
              {(channel) => (
                <A
                  href={`/channels/${channel.id}`}
                  class={cn(
                    'w-10 h-10 rounded-lg flex items-center justify-center text-text-2 transition-colors',
                    currentChannelId() === channel.id
                      ? 'bg-brand/10 text-brand'
                      : 'hover:bg-bg-surface-2 hover:text-text-1'
                  )}
                  title={channel.display_name}
                >
                  <HiOutlineHashtag size={18} />
                </A>
              )}
            </For>
          </div>
        </Show>
      </div>

      {/* Sidebar Footer */}
      <Show when={!isCollapsed()}>
        <div class="p-3 border-t border-border-1 shrink-0">
          <Show when={canAccessAdmin()}>
            <A
              href="/admin"
              class="w-full mb-2 flex items-center gap-2 px-3 py-2 rounded-lg bg-bg-app border border-border-1 hover:border-border-2 transition-colors text-left text-text-2 hover:text-text-1"
            >
              <HiOutlineCog6Tooth size={18} class="text-text-3" />
              <span class="text-sm">Admin Console</span>
            </A>
          </Show>
          <A
            href="/playbooks"
            class="w-full mb-2 flex items-center gap-2 px-3 py-2 rounded-lg bg-bg-app border border-border-1 hover:border-border-2 transition-colors text-left text-text-2 hover:text-text-1"
          >
            <HiOutlineBookOpen size={18} class="text-text-3" />
            <span class="text-sm">Playbooks</span>
          </A>
          <button
            type="button"
            class="w-full flex items-center gap-2 px-3 py-2 rounded-lg bg-bg-app border border-border-1 hover:border-border-2 transition-colors text-left"
            onClick={() => {
              void openCreateChannelModal();
            }}
          >
            <HiOutlinePlus size={18} class="text-text-3" />
            <span class="text-sm text-text-2">Add Channel</span>
          </button>
        </div>
      </Show>

      <Modal
        isOpen={isCreateChannelOpen()}
        onClose={() => setIsCreateChannelOpen(false)}
        title="Create Channel"
        description="Create a new public or private channel"
        size="md"
      >
        <div class="space-y-4">
          <Show when={createChannelError()}>
            <div class="rounded-lg border border-danger/30 bg-danger/10 px-3 py-2 text-sm text-danger">
              {createChannelError()}
            </div>
          </Show>

          <label class="block space-y-1">
            <span class="text-sm font-medium text-text-2">Team</span>
            <select
              value={selectedTeamId()}
              onChange={(event) => setSelectedTeamId(event.currentTarget.value)}
              disabled={isLoadingChannelTeams() || channelTeams().length === 0}
              class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
            >
              <For each={channelTeams()}>
                {(team) => (
                  <option value={team.id}>{team.display_name || team.name}</option>
                )}
              </For>
            </select>
            <Show when={isLoadingChannelTeams()}>
              <p class="text-xs text-text-3">Loading teams...</p>
            </Show>
            <Show when={!isLoadingChannelTeams() && channelTeams().length === 0}>
              <p class="text-xs text-warning">No teams available for channel creation.</p>
            </Show>
          </label>

          <label class="block space-y-1">
            <span class="text-sm font-medium text-text-2">Display Name</span>
            <input
              type="text"
              value={channelDisplayName()}
              onInput={(event) => setChannelDisplayName(event.currentTarget.value)}
              placeholder="e.g. Engineering"
              class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
            />
          </label>

          <label class="block space-y-1">
            <span class="text-sm font-medium text-text-2">Purpose (optional)</span>
            <input
              type="text"
              value={channelPurpose()}
              onInput={(event) => setChannelPurpose(event.currentTarget.value)}
              placeholder="What is this channel for?"
              class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
            />
          </label>

          <label class="block space-y-1">
            <span class="text-sm font-medium text-text-2">Visibility</span>
            <select
              value={channelType()}
              onChange={(event) => setChannelType(event.currentTarget.value as 'public' | 'private')}
              class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
            >
              <option value="public">Public</option>
              <option value="private">Private</option>
            </select>
          </label>

          <div class="flex justify-end gap-2 pt-2">
            <Button variant="ghost" onClick={() => setIsCreateChannelOpen(false)}>
              Cancel
            </Button>
            <Button
              variant="primary"
              loading={isCreatingChannel()}
              disabled={isLoadingChannelTeams() || channelTeams().length === 0}
              onClick={() => void submitCreateChannel()}
            >
              Create Channel
            </Button>
          </div>
        </div>
      </Modal>

      <Modal
        isOpen={isCreateDmOpen()}
        onClose={() => setIsCreateDmOpen(false)}
        title="Start Direct Message"
        description="Select a teammate to open a direct message channel"
        size="md"
      >
        <div class="space-y-4">
          <Show when={dmError()}>
            <div class="rounded-lg border border-danger/30 bg-danger/10 px-3 py-2 text-sm text-danger">
              {dmError()}
            </div>
          </Show>

          <label class="block space-y-1">
            <span class="text-sm font-medium text-text-2">Team</span>
            <select
              value={dmTeamId()}
              onChange={(event) => setDmTeamId(event.currentTarget.value)}
              disabled={isLoadingDmUsers() || channelTeams().length === 0}
              class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
            >
              <For each={channelTeams()}>
                {(team) => (
                  <option value={team.id}>{team.display_name || team.name}</option>
                )}
              </For>
            </select>
          </label>

          <label class="block space-y-1">
            <span class="text-sm font-medium text-text-2">Search</span>
            <input
              type="text"
              value={dmSearchQuery()}
              onInput={(event) => setDmSearchQuery(event.currentTarget.value)}
              placeholder="Find by username or display name"
              class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
            />
          </label>

          <div class="max-h-72 space-y-2 overflow-y-auto rounded-lg border border-border-1 bg-bg-app p-2">
            <Show when={!isLoadingDmUsers()} fallback={<p class="px-2 py-2 text-sm text-text-3">Loading users...</p>}>
              <Show when={filteredDmUsers().length > 0} fallback={<p class="px-2 py-2 text-sm text-text-3">No users found.</p>}>
                <For each={filteredDmUsers()}>
                  {(user) => (
                    <button
                      type="button"
                      onClick={() => setDmSelectedUserId(user.id)}
                      class={cn(
                        'flex w-full items-center justify-between rounded-lg border px-3 py-2 text-left transition-colors',
                        dmSelectedUserId() === user.id
                          ? 'border-brand bg-brand/10'
                          : 'border-border-1 hover:bg-bg-surface-2'
                      )}
                    >
                      <div class="min-w-0">
                        <p class="truncate text-sm font-medium text-text-1">
                          {user.display_name || user.username}
                        </p>
                        <p class="truncate text-xs text-text-3">@{user.username}</p>
                      </div>
                      <Show when={dmSelectedUserId() === user.id}>
                        <span class="text-xs font-semibold text-brand">Selected</span>
                      </Show>
                    </button>
                  )}
                </For>
              </Show>
            </Show>
          </div>

          <div class="flex justify-end gap-2 pt-2">
            <Button variant="ghost" onClick={() => setIsCreateDmOpen(false)}>
              Cancel
            </Button>
            <Button
              variant="primary"
              loading={isCreatingDm()}
              disabled={isLoadingDmUsers() || !dmSelectedUserId() || !dmTeamId()}
              onClick={() => {
                void submitCreateDirectMessage();
              }}
            >
              Start Message
            </Button>
          </div>
        </div>
      </Modal>
    </aside>
  );
}

// ============================================
// Team Selector Component
// ============================================

function TeamSelector() {
  const navigate = useNavigate();
  const [isOpen, setIsOpen] = createSignal(false);
  const [teams, setTeams] = createSignal<TeamOption[]>([]);
  const [isLoadingTeams, setIsLoadingTeams] = createSignal(false);
  const [teamError, setTeamError] = createSignal<string | null>(null);
  const [isTeamModalOpen, setIsTeamModalOpen] = createSignal(false);
  const [teamModalMode, setTeamModalMode] = createSignal<'join' | 'create'>('join');
  const [publicTeams, setPublicTeams] = createSignal<TeamOption[]>([]);
  const [teamModalError, setTeamModalError] = createSignal<string | null>(null);
  const [isTeamActionLoading, setIsTeamActionLoading] = createSignal(false);
  const [newTeamName, setNewTeamName] = createSignal('');
  const [newTeamDisplayName, setNewTeamDisplayName] = createSignal('');
  const [newTeamDescription, setNewTeamDescription] = createSignal('');

  const loadTeams = async () => {
    if (!authStore.token) return;

    setIsLoadingTeams(true);
    setTeamError(null);

    try {
      setTeams(await fetchTeamsWithBootstrap());
    } catch (error) {
      setTeams([]);
      setTeamError(getErrorMessage(error) || 'Failed to fetch teams');
    } finally {
      setIsLoadingTeams(false);
    }
  };

  createEffect(() => {
    if (authStore.isAuthenticated) {
      void loadTeams();
    }
  });

  const loadPublicTeams = async () => {
    if (!authStore.token) return;

    const response =
      await client.get<Array<{ id?: unknown; name?: unknown; display_name?: unknown }>>(
        '/teams/public'
      );
    setPublicTeams(normalizeTeams(response.data));
  };

  const openTeamModal = async (mode: 'join' | 'create') => {
    setTeamModalMode(mode);
    setTeamModalError(null);
    setNewTeamName('');
    setNewTeamDisplayName('');
    setNewTeamDescription('');
    setIsOpen(false);
    setIsTeamModalOpen(true);
    if (mode === 'join') {
      try {
        await Promise.all([loadTeams(), loadPublicTeams()]);
      } catch (error) {
        setTeamModalError(error instanceof Error ? error.message : 'Failed to load teams.');
      }
    }
  };

  const createNewTeam = async () => {
    if (!authStore.token) return;

    const displayName = (newTeamDisplayName() || newTeamName()).trim();
    if (!displayName) {
      setTeamModalError('Team name is required.');
      return;
    }

    setIsTeamActionLoading(true);
    setTeamModalError(null);
    try {
      const response = await client.post<TeamOption>('/teams', {
          name: toTeamName(newTeamName().trim() || displayName),
          display_name: displayName,
          description: newTeamDescription().trim() || undefined,
      });
      const createdTeam = response.data;
      await loadTeams();
      await switchTeam(createdTeam.id);
      setIsTeamModalOpen(false);
    } catch (error) {
      setTeamModalError(getErrorMessage(error) || 'Failed to create team.');
    } finally {
      setIsTeamActionLoading(false);
    }
  };

  const joinTeam = async (teamId: string) => {
    if (!authStore.token) return;

    setIsTeamActionLoading(true);
    setTeamModalError(null);
    try {
      await client.post(`/teams/${teamId}/join`);

      await loadTeams();
      await switchTeam(teamId);
      setIsTeamModalOpen(false);
    } catch (error) {
      setTeamModalError(getErrorMessage(error) || 'Failed to join team.');
    } finally {
      setIsTeamActionLoading(false);
    }
  };

  const currentTeam = createMemo(() => {
    const currentTeamId = channelStore.currentChannel()?.team_id;
    if (currentTeamId) {
      const matched = teams().find((team) => team.id === currentTeamId);
      if (matched) return matched;
    }
    return teams()[0] || null;
  });

  const switchTeam = async (teamId: string) => {
    if (currentTeam()?.id === teamId) {
      setIsOpen(false);
      return;
    }

    try {
      await fetchChannels(teamId);

      let nextChannelId = channelStore.currentChannelId();
      if (!nextChannelId) {
        await fetchJoinableChannels(teamId);
        const firstJoinable = channelStore.joinableChannels[0];
        if (firstJoinable) {
          await joinChannel(firstJoinable.id);
          await fetchChannels(teamId);
          nextChannelId = channelStore.currentChannelId();
        }
      }

      if (nextChannelId) {
        navigate(`/channels/${nextChannelId}`);
      } else {
        navigate('/');
      }
      setIsOpen(false);
      uiStore.setMobileSidebarOpen(false);
    } catch (error) {
      setTeamError(getErrorMessage(error) || 'Failed to switch team');
    }
  };

  const unreadForTeam = (teamId: string) => unreadStore.teamUnreads[teamId] || 0;
  const teamInitial = (team: { name: string; display_name?: string | null }) =>
    (team.display_name || team.name).charAt(0).toUpperCase();

  return (
    <div class="relative flex-1">
      <button
        type="button"
        class="flex items-center gap-2 px-2 py-1.5 rounded-lg hover:bg-bg-surface-2 transition-colors w-full text-left"
        onClick={() => setIsOpen((prev) => !prev)}
        aria-expanded={isOpen()}
        aria-haspopup="listbox"
      >
        <div class="w-6 h-6 rounded bg-brand flex items-center justify-center text-white text-xs font-bold">
          {currentTeam() ? teamInitial(currentTeam()!) : 'R'}
        </div>
        <span class="font-semibold text-text-1 truncate">
          {currentTeam()?.display_name || currentTeam()?.name || 'RustChat'}
        </span>
        <HiOutlineChevronDown size={16} class="ml-auto text-text-3" />
      </button>

      <Show when={isOpen()}>
        <>
          <div
            class="fixed inset-0 z-40"
            onClick={() => setIsOpen(false)}
            aria-hidden="true"
          />
          <div class="absolute left-0 top-full mt-1 w-56 bg-bg-surface-1 border border-border-1 rounded-lg shadow-lg z-50 py-2">
            <div class="px-3 py-2 border-b border-border-1">
              <p class="text-xs text-text-3 uppercase tracking-wider">Your Teams</p>
            </div>
            <Show when={teamError()}>
              <p class="px-3 py-2 text-xs text-danger">{teamError()}</p>
            </Show>
            <Show when={isLoadingTeams()}>
              <p class="px-3 py-2 text-xs text-text-3">Loading teams...</p>
            </Show>
            <For each={teams()}>
              {(team) => (
                <button
                  type="button"
                  class={cn(
                    'w-full px-3 py-2 flex items-center gap-2 hover:bg-bg-surface-2 transition-colors',
                    team.id === currentTeam()?.id ? 'bg-bg-surface-2' : ''
                  )}
                  onClick={() => {
                    void switchTeam(team.id);
                  }}
                >
                  <div class="w-6 h-6 rounded bg-brand flex items-center justify-center text-white text-xs font-bold">
                    {teamInitial(team)}
                  </div>
                  <span class="text-sm text-text-1 truncate">{team.display_name || team.name}</span>
                  <Show when={unreadForTeam(team.id) > 0}>
                    <span class="ml-auto min-w-[18px] h-[18px] px-1.5 bg-danger text-white text-[10px] font-semibold rounded-full flex items-center justify-center">
                      {unreadForTeam(team.id) > 99 ? '99+' : unreadForTeam(team.id)}
                    </span>
                  </Show>
                </button>
              )}
            </For>
            <div class="border-t border-border-1 mt-1 pt-1">
                  <button
                    type="button"
                    class="w-full px-3 py-2 text-left text-sm text-text-2 hover:bg-bg-surface-2 hover:text-text-1 flex items-center gap-2"
                    onClick={() => {
                      void openTeamModal('join');
                    }}
                  >
                    <HiOutlinePlus size={16} />
                    Create or Join Team
                  </button>
                </div>
              </div>
            </>
          </Show>
      <Modal
        isOpen={isTeamModalOpen()}
        onClose={() => setIsTeamModalOpen(false)}
        title={teamModalMode() === 'create' ? 'Create Team' : 'Create or Join Team'}
        description={
          teamModalMode() === 'create'
            ? 'Create a new workspace team'
            : 'Join a public team or create your own'
        }
        size="md"
      >
        <div class="space-y-4">
          <div class="flex gap-2">
            <Button
              variant={teamModalMode() === 'join' ? 'primary' : 'secondary'}
              onClick={() => {
                void openTeamModal('join');
              }}
            >
              Join Team
            </Button>
            <Button
              variant={teamModalMode() === 'create' ? 'primary' : 'secondary'}
              onClick={() => {
                setTeamModalMode('create');
                setTeamModalError(null);
              }}
            >
              Create Team
            </Button>
          </div>

          <Show when={teamModalError()}>
            <div class="rounded-lg border border-danger/30 bg-danger/10 px-3 py-2 text-sm text-danger">
              {teamModalError()}
            </div>
          </Show>

          <Show when={teamModalMode() === 'create'}>
            <div class="space-y-3">
              <label class="block space-y-1">
                <span class="text-sm font-medium text-text-2">Team Name</span>
                <input
                  type="text"
                  value={newTeamName()}
                  onInput={(event) => setNewTeamName(event.currentTarget.value)}
                  placeholder="engineering"
                  class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
                />
              </label>
              <label class="block space-y-1">
                <span class="text-sm font-medium text-text-2">Display Name</span>
                <input
                  type="text"
                  value={newTeamDisplayName()}
                  onInput={(event) => setNewTeamDisplayName(event.currentTarget.value)}
                  placeholder="Engineering Team"
                  class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
                />
              </label>
              <label class="block space-y-1">
                <span class="text-sm font-medium text-text-2">Description (optional)</span>
                <textarea
                  value={newTeamDescription()}
                  onInput={(event) => setNewTeamDescription(event.currentTarget.value)}
                  rows={3}
                  class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1 resize-none"
                />
              </label>
              <div class="flex justify-end gap-2 pt-1">
                <Button variant="ghost" onClick={() => setIsTeamModalOpen(false)}>
                  Cancel
                </Button>
                <Button
                  variant="primary"
                  loading={isTeamActionLoading()}
                  onClick={() => {
                    void createNewTeam();
                  }}
                >
                  Create Team
                </Button>
              </div>
            </div>
          </Show>

          <Show when={teamModalMode() === 'join'}>
            <div class="space-y-3">
              <Show when={!isLoadingTeams()} fallback={<p class="text-sm text-text-3">Loading public teams...</p>}>
                <For each={publicTeams().filter((team) => !teams().some((joined) => joined.id === team.id))}>
                  {(team) => (
                    <div class="flex items-center justify-between rounded-lg border border-border-1 bg-bg-app px-3 py-2">
                      <div class="min-w-0">
                        <p class="text-sm font-medium text-text-1 truncate">{team.display_name || team.name}</p>
                        <p class="text-xs text-text-3 truncate">{team.name}</p>
                      </div>
                      <Button
                        variant="secondary"
                        size="sm"
                        loading={isTeamActionLoading()}
                        onClick={() => {
                          void joinTeam(team.id);
                        }}
                      >
                        Join
                      </Button>
                    </div>
                  )}
                </For>
              </Show>
              <Show when={publicTeams().filter((team) => !teams().some((joined) => joined.id === team.id)).length === 0 && !isLoadingTeams()}>
                <p class="text-sm text-text-3">No additional public teams available. You can create a new one.</p>
              </Show>
            </div>
          </Show>
        </div>
      </Modal>
    </div>
  );
}

// ============================================
// Channel Section Component
// ============================================

interface ChannelSectionProps {
  title: string;
  icon: ReturnType<typeof HiOutlineHashtag>;
  isExpanded: boolean;
  onToggle: () => void;
  children: ReturnType<typeof For>;
  action?: ReturnType<typeof Show>;
}

function ChannelSection(props: ChannelSectionProps) {
  return (
    <div class="mb-1">
      <div class="group flex items-center gap-1 px-3 py-1">
        <button
          type="button"
          class="flex min-w-0 flex-1 items-center gap-1 text-text-3 hover:text-text-2 transition-colors"
          onClick={props.onToggle}
          aria-expanded={props.isExpanded}
        >
          {props.isExpanded ? (
            <HiOutlineChevronDown size={14} />
          ) : (
            <HiOutlineChevronRight size={14} />
          )}
          <span class="flex items-center gap-1.5 text-xs font-semibold uppercase tracking-wider">
            {props.icon}
            {props.title}
          </span>
        </button>
        <Show when={props.action}>
          <span>{props.action}</span>
        </Show>
      </div>
      <Show when={props.isExpanded}>
        <div class="mt-0.5">{props.children}</div>
      </Show>
    </div>
  );
}

// ============================================
// Channel Item Component
// ============================================

interface ChannelItemProps {
  channel: Channel;
  isActive: boolean;
  onClick: () => void;
  isPrivate?: boolean;
  isDirectMessage?: boolean;
}

function ChannelItem(props: ChannelItemProps) {
  const href = () => `/channels/${props.channel.id}`;

  const handleClick = (e: MouseEvent) => {
    // Handle middle-click or Ctrl+click to open in new tab
    if (e.ctrlKey || e.metaKey || e.button === 1) {
      return; // Let default behavior happen
    }
    e.preventDefault();
    props.onClick();
    // Close mobile sidebar
    uiStore.setMobileSidebarOpen(false);
  };

  const icon = () => {
    if (props.isDirectMessage) {
      return <HiOutlineUser size={16} />;
    }
    if (props.isPrivate) {
      return <HiOutlineLockClosed size={16} />;
    }
    return <HiOutlineHashtag size={16} />;
  };

  const unreadCount = () => props.channel.unreadCount || 0;
  const mentionCount = () => props.channel.mentionCount || 0;

  return (
    <A
      href={href()}
      onClick={handleClick}
      class={cn(
        'group flex items-center gap-2 px-3 py-1.5 mx-2 rounded-lg text-sm transition-colors',
        props.isActive
          ? 'bg-brand/10 text-brand'
          : unreadCount() > 0
          ? 'text-text-1 hover:bg-bg-surface-2'
          : 'text-text-2 hover:bg-bg-surface-2 hover:text-text-1'
      )}
      aria-current={props.isActive ? 'page' : undefined}
    >
      <span class={cn('flex-shrink-0', props.isActive ? 'text-brand' : 'text-text-3')}>
        {icon()}
      </span>
      <span class="flex-1 truncate font-medium">
        {props.channel.display_name}
      </span>
      <Show when={mentionCount() > 0}>
        <span class="flex-shrink-0 min-w-[18px] h-[18px] px-1.5 bg-danger text-white text-[10px] font-semibold rounded-full flex items-center justify-center">
          {mentionCount() > 99 ? '99+' : mentionCount()}
        </span>
      </Show>
      <Show when={unreadCount() > 0 && mentionCount() === 0}>
        <span class="flex-shrink-0 w-2 h-2 bg-brand rounded-full" />
      </Show>
    </A>
  );
}

// ============================================
// Handler Functions
// ============================================

function handleChannelClick(channelId: string) {
  selectChannel(channelId);
  uiStore.setMobileSidebarOpen(false);
}

export default Sidebar;
