import { For, Show, createEffect, createMemo, createSignal } from 'solid-js';
import Modal from '@/components/ui/Modal';
import Button from '@/components/ui/Button';
import { channelsApi } from '@/api/channels';
import { usersApi } from '@/api/users';
import { client, getErrorMessage } from '@/api/client';
import { toast } from '@/hooks/useToast';
import { authStore } from '@/stores/auth';
import type { Channel, ChannelMember } from '@/stores/channels';

interface TeamMemberResponse {
  team_id: string;
  user_id: string;
  role: string;
  username: string;
  display_name?: string | null;
  avatar_url?: string | null;
}

interface TeamSettingsRecord {
  id: string;
  name: string;
  display_name?: string | null;
  description?: string | null;
  invite_id?: string;
  is_public?: boolean;
  allow_open_invite?: boolean;
}

interface UserOption {
  id: string;
  username: string;
  display_name?: string;
}

interface ChannelSettingsModalProps {
  isOpen: boolean;
  channel: Channel | null;
  onClose: () => void;
  onUpdated?: () => Promise<void> | void;
  onDeleted?: () => Promise<void> | void;
}

interface TeamSettingsModalProps {
  isOpen: boolean;
  teamId: string | null;
  onClose: () => void;
  onUpdated?: () => Promise<void> | void;
  onTeamRemoved?: () => Promise<void> | void;
}

function userLabel(user: { display_name?: string | null; username?: string | null; user_id?: string }): string {
  return user.display_name || user.username || user.user_id || 'Unknown user';
}

export function ChannelSettingsModal(props: ChannelSettingsModalProps) {
  const [displayName, setDisplayName] = createSignal('');
  const [purpose, setPurpose] = createSignal('');
  const [header, setHeader] = createSignal('');
  const [search, setSearch] = createSignal('');
  const [teamMembers, setTeamMembers] = createSignal<TeamMemberResponse[]>([]);
  const [channelMembers, setChannelMembers] = createSignal<ChannelMember[]>([]);
  const [isLoadingMembers, setIsLoadingMembers] = createSignal(false);
  const [isSaving, setIsSaving] = createSignal(false);
  const [isDeleting, setIsDeleting] = createSignal(false);
  const [addingUserId, setAddingUserId] = createSignal<string | null>(null);
  const [removingUserId, setRemovingUserId] = createSignal<string | null>(null);
  const [error, setError] = createSignal<string | null>(null);

  const loadMembers = async () => {
    const channel = props.channel;
    if (!channel) return;

    setIsLoadingMembers(true);
    setError(null);
    try {
      const [members, team] = await Promise.all([
        channelsApi.getMembers(channel.id),
        client.get<TeamMemberResponse[]>(`/teams/${channel.team_id}/members`),
      ]);
      setChannelMembers(members);
      setTeamMembers(team.data);
    } catch (err) {
      setError(getErrorMessage(err) || 'Failed to load channel members.');
    } finally {
      setIsLoadingMembers(false);
    }
  };

  createEffect(() => {
    if (!props.isOpen || !props.channel) return;

    setDisplayName(props.channel.display_name || '');
    setPurpose(props.channel.purpose || '');
    setHeader(props.channel.header || '');
    setSearch('');
    void loadMembers();
  });

  const currentMemberIds = createMemo(() => new Set(channelMembers().map((member) => member.user_id)));
  const filteredCandidates = createMemo(() => {
    const query = search().trim().toLowerCase();
    if (!query) return [] as TeamMemberResponse[];

    return teamMembers()
      .filter((member) => !currentMemberIds().has(member.user_id))
      .filter((member) => {
        const name = userLabel(member).toLowerCase();
        const username = member.username.toLowerCase();
        return name.includes(query) || username.includes(query);
      })
      .slice(0, 8);
  });

  const save = async () => {
    const channel = props.channel;
    if (!channel || isSaving()) return;

    setIsSaving(true);
    setError(null);
    try {
      await channelsApi.update(channel.id, {
        display_name: displayName().trim() || channel.display_name,
        purpose: purpose().trim(),
        header: header().trim(),
      });
      toast.success('Channel settings saved', 'Channel details were updated.');
      await props.onUpdated?.();
      props.onClose();
    } catch (err) {
      const message = getErrorMessage(err) || 'Failed to save channel settings.';
      setError(message);
      toast.error('Unable to save settings', message);
    } finally {
      setIsSaving(false);
    }
  };

  const addMember = async (userId: string) => {
    const channel = props.channel;
    if (!channel || addingUserId()) return;

    setAddingUserId(userId);
    try {
      await channelsApi.addMember(channel.id, userId);
      await loadMembers();
      setSearch('');
      toast.success('Member added', 'User added to channel.');
    } catch (err) {
      toast.error('Unable to add member', getErrorMessage(err) || 'Please try again.');
    } finally {
      setAddingUserId(null);
    }
  };

  const removeMember = async (userId: string) => {
    const channel = props.channel;
    if (!channel || removingUserId()) return;

    setRemovingUserId(userId);
    try {
      await channelsApi.removeMember(channel.id, userId);
      await loadMembers();
      toast.success('Member removed', 'User removed from channel.');
    } catch (err) {
      toast.error('Unable to remove member', getErrorMessage(err) || 'Please try again.');
    } finally {
      setRemovingUserId(null);
    }
  };

  const archive = async () => {
    const channel = props.channel;
    if (!channel || isDeleting()) return;

    const confirmed = window.confirm(
      `Archive #${channel.display_name || channel.name}? You can restore it later from admin tools.`
    );
    if (!confirmed) return;

    setIsDeleting(true);
    try {
      await channelsApi.delete(channel.id);
      toast.success('Channel archived', `${channel.display_name || channel.name} was archived.`);
      await props.onDeleted?.();
      props.onClose();
    } catch (err) {
      toast.error('Unable to archive channel', getErrorMessage(err) || 'Please try again.');
    } finally {
      setIsDeleting(false);
    }
  };

  return (
    <Modal
      isOpen={props.isOpen}
      onClose={props.onClose}
      title="Channel Settings"
      description={props.channel ? `Manage #${props.channel.display_name || props.channel.name}` : undefined}
      size="xl"
    >
      <div class="grid gap-6 lg:grid-cols-[1fr_1fr]">
        <div class="space-y-4">
          <Show when={error()}>
            <div class="rounded-lg border border-danger/30 bg-danger/10 px-3 py-2 text-sm text-danger">
              {error()}
            </div>
          </Show>

          <label class="block space-y-1">
            <span class="text-sm font-medium text-text-2">Channel Name</span>
            <input
              type="text"
              value={props.channel?.name || ''}
              disabled
              class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-3"
            />
          </label>

          <label class="block space-y-1">
            <span class="text-sm font-medium text-text-2">Display Name</span>
            <input
              type="text"
              value={displayName()}
              onInput={(event) => setDisplayName(event.currentTarget.value)}
              class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
            />
          </label>

          <label class="block space-y-1">
            <span class="text-sm font-medium text-text-2">Purpose</span>
            <textarea
              value={purpose()}
              onInput={(event) => setPurpose(event.currentTarget.value)}
              rows={3}
              class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1 resize-none"
            />
          </label>

          <label class="block space-y-1">
            <span class="text-sm font-medium text-text-2">Header</span>
            <textarea
              value={header()}
              onInput={(event) => setHeader(event.currentTarget.value)}
              rows={3}
              class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1 resize-none"
            />
          </label>

          <div class="flex flex-wrap justify-end gap-2 pt-2 border-t border-border-1">
            <Button variant="ghost" onClick={props.onClose}>
              Cancel
            </Button>
            <Button variant="danger" loading={isDeleting()} onClick={() => void archive()}>
              Archive Channel
            </Button>
            <Button variant="primary" loading={isSaving()} onClick={() => void save()}>
              Save
            </Button>
          </div>
        </div>

        <div class="space-y-4">
          <div>
            <h3 class="text-sm font-semibold text-text-1">Members</h3>
            <p class="text-xs text-text-3">{channelMembers().length} in channel</p>
          </div>

          <label class="block space-y-1">
            <span class="text-sm font-medium text-text-2">Add Member</span>
            <input
              type="text"
              value={search()}
              onInput={(event) => setSearch(event.currentTarget.value)}
              placeholder="Search team members"
              class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
            />
          </label>

          <Show when={search().trim() && filteredCandidates().length > 0}>
            <div class="max-h-40 space-y-1 overflow-y-auto rounded-lg border border-border-1 bg-bg-app p-2">
              <For each={filteredCandidates()}>
                {(candidate) => (
                  <div class="flex items-center justify-between rounded-lg border border-border-1 bg-bg-surface-1 px-2 py-1.5">
                    <div class="min-w-0">
                      <p class="truncate text-sm text-text-1">{userLabel(candidate)}</p>
                      <p class="truncate text-xs text-text-3">@{candidate.username}</p>
                    </div>
                    <Button
                      size="sm"
                      variant="secondary"
                      loading={addingUserId() === candidate.user_id}
                      onClick={() => void addMember(candidate.user_id)}
                    >
                      Add
                    </Button>
                  </div>
                )}
              </For>
            </div>
          </Show>

          <div class="max-h-72 space-y-2 overflow-y-auto rounded-lg border border-border-1 bg-bg-app p-2">
            <Show when={!isLoadingMembers()} fallback={<p class="text-sm text-text-3">Loading members...</p>}>
              <For each={channelMembers()}>
                {(member) => {
                  const selfId = authStore.user()?.id;
                  const isSelf = () => member.user_id === selfId;
                  return (
                    <div class="flex items-center justify-between rounded-lg border border-border-1 bg-bg-surface-1 px-2 py-1.5">
                      <div class="min-w-0">
                        <p class="truncate text-sm text-text-1">{userLabel(member)}</p>
                        <p class="truncate text-xs text-text-3">@{member.username || member.user_id}</p>
                      </div>
                      <Show when={!isSelf()}>
                        <Button
                          size="sm"
                          variant="ghost"
                          loading={removingUserId() === member.user_id}
                          onClick={() => void removeMember(member.user_id)}
                        >
                          Remove
                        </Button>
                      </Show>
                      <Show when={isSelf()}>
                        <span class="text-xs text-text-3">You</span>
                      </Show>
                    </div>
                  );
                }}
              </For>
            </Show>
          </div>
        </div>
      </div>
    </Modal>
  );
}

export function TeamSettingsModal(props: TeamSettingsModalProps) {
  const [team, setTeam] = createSignal<TeamSettingsRecord | null>(null);
  const [displayName, setDisplayName] = createSignal('');
  const [description, setDescription] = createSignal('');
  const [isPublic, setIsPublic] = createSignal(false);
  const [allowOpenInvite, setAllowOpenInvite] = createSignal(false);
  const [members, setMembers] = createSignal<TeamMemberResponse[]>([]);
  const [search, setSearch] = createSignal('');
  const [searchResults, setSearchResults] = createSignal<UserOption[]>([]);
  const [error, setError] = createSignal<string | null>(null);
  const [isLoading, setIsLoading] = createSignal(false);
  const [isSaving, setIsSaving] = createSignal(false);
  const [isLeaving, setIsLeaving] = createSignal(false);
  const [isDeleting, setIsDeleting] = createSignal(false);
  const [addingUserId, setAddingUserId] = createSignal<string | null>(null);
  const [removingUserId, setRemovingUserId] = createSignal<string | null>(null);

  const loadTeam = async () => {
    const teamId = props.teamId;
    if (!teamId) return;

    setIsLoading(true);
    setError(null);
    try {
      const [teamResponse, membersResponse] = await Promise.all([
        client.get<TeamSettingsRecord>(`/teams/${teamId}`),
        client.get<TeamMemberResponse[]>(`/teams/${teamId}/members`),
      ]);
      setTeam(teamResponse.data);
      setDisplayName(teamResponse.data.display_name || teamResponse.data.name || '');
      setDescription(teamResponse.data.description || '');
      setIsPublic(Boolean(teamResponse.data.is_public));
      setAllowOpenInvite(Boolean(teamResponse.data.allow_open_invite));
      setMembers(membersResponse.data);
    } catch (err) {
      setError(getErrorMessage(err) || 'Failed to load team settings.');
    } finally {
      setIsLoading(false);
    }
  };

  createEffect(() => {
    if (!props.isOpen || !props.teamId) return;
    setSearch('');
    setSearchResults([]);
    void loadTeam();
  });

  createEffect(() => {
    if (!props.isOpen || !props.teamId) return;

    const query = search().trim();
    if (query.length < 2) {
      setSearchResults([]);
      return;
    }

    let cancelled = false;
    const timer = window.setTimeout(() => {
      void usersApi
        .list({ q: query, per_page: 20 })
        .then((users) => {
          if (cancelled) return;
          const memberIds = new Set(members().map((member) => member.user_id));
          const filtered = users
            .filter((user) => !memberIds.has(user.id))
            .map((user) => ({
              id: user.id,
              username: user.username,
              display_name: user.display_name,
            }));
          setSearchResults(filtered);
        })
        .catch(() => {
          if (cancelled) return;
          setSearchResults([]);
        });
    }, 250);

    return () => {
      cancelled = true;
      window.clearTimeout(timer);
    };
  });

  const save = async () => {
    const currentTeam = team();
    if (!currentTeam || isSaving()) return;

    setIsSaving(true);
    setError(null);
    try {
      const response = await client.put<TeamSettingsRecord>(`/teams/${currentTeam.id}`, {
        display_name: displayName().trim(),
        description: description().trim(),
        is_public: isPublic(),
        allow_open_invite: allowOpenInvite(),
      });
      setTeam(response.data);
      toast.success('Team settings saved', 'Team details were updated.');
      await props.onUpdated?.();
    } catch (err) {
      const message = getErrorMessage(err) || 'Failed to save team settings.';
      setError(message);
      toast.error('Unable to save team settings', message);
    } finally {
      setIsSaving(false);
    }
  };

  const addMember = async (userId: string) => {
    const currentTeam = team();
    if (!currentTeam || addingUserId()) return;

    setAddingUserId(userId);
    try {
      await client.post(`/teams/${currentTeam.id}/members`, {
        user_id: userId,
        role: 'member',
      });
      await loadTeam();
      setSearch('');
      toast.success('Member added', 'User added to team.');
    } catch (err) {
      toast.error('Unable to add member', getErrorMessage(err) || 'Please try again.');
    } finally {
      setAddingUserId(null);
    }
  };

  const removeMember = async (userId: string) => {
    const currentTeam = team();
    if (!currentTeam || removingUserId()) return;

    setRemovingUserId(userId);
    try {
      await client.delete(`/teams/${currentTeam.id}/members/${userId}`);
      await loadTeam();
      toast.success('Member removed', 'User removed from team.');
    } catch (err) {
      toast.error('Unable to remove member', getErrorMessage(err) || 'Please try again.');
    } finally {
      setRemovingUserId(null);
    }
  };

  const leaveTeam = async () => {
    const currentTeam = team();
    if (!currentTeam || isLeaving()) return;

    const confirmed = window.confirm(`Leave ${currentTeam.display_name || currentTeam.name}?`);
    if (!confirmed) return;

    setIsLeaving(true);
    try {
      await client.post(`/teams/${currentTeam.id}/leave`);
      toast.success('Left team', `${currentTeam.display_name || currentTeam.name} was left.`);
      await props.onTeamRemoved?.();
      props.onClose();
    } catch (err) {
      toast.error('Unable to leave team', getErrorMessage(err) || 'Please try again.');
    } finally {
      setIsLeaving(false);
    }
  };

  const deleteTeam = async () => {
    const currentTeam = team();
    if (!currentTeam || isDeleting()) return;

    const confirmed = window.confirm(
      `Delete ${currentTeam.display_name || currentTeam.name}? This action removes the whole team.`
    );
    if (!confirmed) return;

    setIsDeleting(true);
    try {
      await client.delete(`/teams/${currentTeam.id}`);
      toast.success('Team deleted', `${currentTeam.display_name || currentTeam.name} was deleted.`);
      await props.onTeamRemoved?.();
      props.onClose();
    } catch (err) {
      toast.error('Unable to delete team', getErrorMessage(err) || 'Please try again.');
    } finally {
      setIsDeleting(false);
    }
  };

  const inviteLink = createMemo(() => {
    const currentTeam = team();
    if (!currentTeam?.invite_id) return '';
    return `${window.location.origin}/join/${currentTeam.invite_id}`;
  });

  const copyInvite = async () => {
    const link = inviteLink();
    if (!link) return;

    try {
      await navigator.clipboard.writeText(link);
      toast.success('Invite link copied', 'Share the link with your teammates.');
    } catch {
      toast.error('Unable to copy link', 'Clipboard permission was denied.');
    }
  };

  return (
    <Modal
      isOpen={props.isOpen}
      onClose={props.onClose}
      title="Team Settings"
      description={team() ? `Manage ${team()?.display_name || team()?.name}` : undefined}
      size="xl"
    >
      <Show when={!isLoading()} fallback={<p class="text-sm text-text-3">Loading team settings...</p>}>
        <Show when={team()} fallback={<p class="text-sm text-text-3">Team unavailable.</p>}>
          <div class="grid gap-6 lg:grid-cols-[1fr_1fr]">
            <div class="space-y-4">
              <Show when={error()}>
                <div class="rounded-lg border border-danger/30 bg-danger/10 px-3 py-2 text-sm text-danger">
                  {error()}
                </div>
              </Show>

              <label class="block space-y-1">
                <span class="text-sm font-medium text-text-2">Team Name</span>
                <input
                  type="text"
                  value={team()?.name || ''}
                  disabled
                  class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-3"
                />
              </label>

              <label class="block space-y-1">
                <span class="text-sm font-medium text-text-2">Display Name</span>
                <input
                  type="text"
                  value={displayName()}
                  onInput={(event) => setDisplayName(event.currentTarget.value)}
                  class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
                />
              </label>

              <label class="block space-y-1">
                <span class="text-sm font-medium text-text-2">Description</span>
                <textarea
                  value={description()}
                  onInput={(event) => setDescription(event.currentTarget.value)}
                  rows={3}
                  class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1 resize-none"
                />
              </label>

              <label class="flex items-center justify-between rounded-lg border border-border-1 bg-bg-app px-3 py-2">
                <span class="text-sm text-text-2">Public Team</span>
                <input
                  type="checkbox"
                  checked={isPublic()}
                  onChange={(event) => setIsPublic(event.currentTarget.checked)}
                  class="h-4 w-4 rounded border-border-2"
                />
              </label>

              <label class="flex items-center justify-between rounded-lg border border-border-1 bg-bg-app px-3 py-2">
                <span class="text-sm text-text-2">Allow Open Invite</span>
                <input
                  type="checkbox"
                  checked={allowOpenInvite()}
                  onChange={(event) => setAllowOpenInvite(event.currentTarget.checked)}
                  class="h-4 w-4 rounded border-border-2"
                />
              </label>

              <Show when={inviteLink()}>
                <div class="rounded-lg border border-border-1 bg-bg-app p-3 space-y-2">
                  <p class="text-xs font-semibold uppercase tracking-wider text-text-3">Invite Link</p>
                  <p class="truncate text-sm text-text-2">{inviteLink()}</p>
                  <Button variant="secondary" size="sm" onClick={() => void copyInvite()}>
                    Copy Link
                  </Button>
                </div>
              </Show>

              <div class="flex flex-wrap justify-end gap-2 pt-2 border-t border-border-1">
                <Button variant="ghost" onClick={props.onClose}>
                  Close
                </Button>
                <Button variant="secondary" loading={isLeaving()} onClick={() => void leaveTeam()}>
                  Leave Team
                </Button>
                <Button variant="danger" loading={isDeleting()} onClick={() => void deleteTeam()}>
                  Delete Team
                </Button>
                <Button variant="primary" loading={isSaving()} onClick={() => void save()}>
                  Save
                </Button>
              </div>
            </div>

            <div class="space-y-4">
              <div>
                <h3 class="text-sm font-semibold text-text-1">Members</h3>
                <p class="text-xs text-text-3">{members().length} in team</p>
              </div>

              <label class="block space-y-1">
                <span class="text-sm font-medium text-text-2">Add Member</span>
                <input
                  type="text"
                  value={search()}
                  onInput={(event) => setSearch(event.currentTarget.value)}
                  placeholder="Search users"
                  class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
                />
              </label>

              <Show when={search().trim() && searchResults().length > 0}>
                <div class="max-h-40 space-y-1 overflow-y-auto rounded-lg border border-border-1 bg-bg-app p-2">
                  <For each={searchResults()}>
                    {(user) => (
                      <div class="flex items-center justify-between rounded-lg border border-border-1 bg-bg-surface-1 px-2 py-1.5">
                        <div class="min-w-0">
                          <p class="truncate text-sm text-text-1">{userLabel(user)}</p>
                          <p class="truncate text-xs text-text-3">@{user.username}</p>
                        </div>
                        <Button
                          size="sm"
                          variant="secondary"
                          loading={addingUserId() === user.id}
                          onClick={() => void addMember(user.id)}
                        >
                          Add
                        </Button>
                      </div>
                    )}
                  </For>
                </div>
              </Show>

              <div class="max-h-72 space-y-2 overflow-y-auto rounded-lg border border-border-1 bg-bg-app p-2">
                <For each={members()}>
                  {(member) => {
                    const selfId = authStore.user()?.id;
                    const isSelf = () => member.user_id === selfId;
                    return (
                      <div class="flex items-center justify-between rounded-lg border border-border-1 bg-bg-surface-1 px-2 py-1.5">
                        <div class="min-w-0">
                          <p class="truncate text-sm text-text-1">{userLabel(member)}</p>
                          <p class="truncate text-xs text-text-3">
                            @{member.username} · {member.role}
                          </p>
                        </div>
                        <Show when={!isSelf()}>
                          <Button
                            size="sm"
                            variant="ghost"
                            loading={removingUserId() === member.user_id}
                            onClick={() => void removeMember(member.user_id)}
                          >
                            Remove
                          </Button>
                        </Show>
                        <Show when={isSelf()}>
                          <span class="text-xs text-text-3">You</span>
                        </Show>
                      </div>
                    );
                  }}
                </For>
              </div>
            </div>
          </div>
        </Show>
      </Show>
    </Modal>
  );
}
