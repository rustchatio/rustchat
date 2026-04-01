<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { 
    Users, 
    MessageSquare, 
    Search, 
    Eye, 
    Trash2, 
    ChevronLeft,
    ChevronRight,
    Hash,
    Lock,
    Building2,
    ArrowLeft,
    Plus,
    Edit2,
    UserPlus,
    UserMinus,
    X
} from 'lucide-vue-next';
import adminApi, { type AdminTeam, type AdminChannel, type AdminUser } from '../../api/admin';
import BaseInput from '../../components/atomic/BaseInput.vue';
import BaseButton from '../../components/atomic/BaseButton.vue';

// State
const teams = ref<AdminTeam[]>([]);
const totalTeams = ref(0);
const loading = ref(true);
const search = ref('');
const page = ref(1);
const perPage = 10;

const selectedTeam = ref<AdminTeam | null>(null);
const activeTab = ref<'channels' | 'members'>('channels');

// Channels State
const teamChannels = ref<AdminChannel[]>([]);
const channelsLoading = ref(false);
const showCreateChannelModal = ref(false);
const showEditChannelModal = ref(false);
const editingChannel = ref<AdminChannel | null>(null);
const channelForm = ref({
    name: '',
    display_name: '',
    purpose: '',
    type: 'public' as 'public' | 'private'
});

// Members State
const teamMembers = ref<any[]>([]);
const membersLoading = ref(false);
const showAddMemberModal = ref(false);
const memberSearch = ref('');
const memberSearchResults = ref<AdminUser[]>([]);
const searchingMembers = ref(false);

// Actions
async function fetchTeams() {
    loading.value = true;
    try {
        const { data } = await adminApi.listTeams({
            page: page.value,
            per_page: perPage,
            search: search.value
        });
        teams.value = data.teams;
        totalTeams.value = data.total;
    } catch (e) {
        console.error('Failed to fetch teams', e);
    } finally {
        loading.value = false;
    }
}

async function viewTeamDetails(team: AdminTeam) {
    selectedTeam.value = team;
    activeTab.value = 'channels';
    await fetchChannels();
}

async function fetchChannels() {
    if (!selectedTeam.value) return;
    channelsLoading.value = true;
    try {
        const { data } = await adminApi.listChannels({
            team_id: selectedTeam.value.id,
            per_page: 100
        });
        teamChannels.value = data.channels;
    } catch (e) {
        console.error('Failed to fetch channels', e);
    } finally {
        channelsLoading.value = false;
    }
}

async function fetchMembers() {
    if (!selectedTeam.value) return;
    membersLoading.value = true;
    try {
        const { data } = await adminApi.listTeamMembers(selectedTeam.value.id);
        teamMembers.value = data;
    } catch (e) {
        console.error('Failed to fetch members', e);
    } finally {
        membersLoading.value = false;
    }
}

function closeDetails() {
    selectedTeam.value = null;
    teamChannels.value = [];
    teamMembers.value = [];
}

async function deleteTeam(team: AdminTeam) {
    if (!confirm(`Are you sure you want to delete the team "${team.display_name || team.name}"? This will permanently delete all channels, messages, and files.`)) {
        return;
    }

    try {
        await adminApi.deleteTeam(team.id);
        await fetchTeams();
    } catch (e) {
        console.error('Failed to delete team', e);
        alert('Failed to delete team');
    }
}

// Channel Actions
function openCreateChannel() {
    channelForm.value = { name: '', display_name: '', purpose: '', type: 'public' };
    showCreateChannelModal.value = true;
}

async function createChannel() {
    if (!selectedTeam.value) return;
    try {
        await adminApi.createChannel({
            team_id: selectedTeam.value.id,
            name: channelForm.value.name,
            display_name: channelForm.value.display_name,
            purpose: channelForm.value.purpose,
            channel_type: channelForm.value.type
        });
        showCreateChannelModal.value = false;
        await fetchChannels();
    } catch (e) {
        console.error('Failed to create channel', e);
        alert('Failed to create channel');
    }
}

function openEditChannel(channel: AdminChannel) {
    editingChannel.value = channel;
    channelForm.value = {
        name: channel.name,
        display_name: channel.display_name || '',
        purpose: channel.purpose || '',
        type: channel.channel_type === 'public' || channel.channel_type === 'private' ? channel.channel_type : 'public'
    };
    showEditChannelModal.value = true;
}

async function updateChannel() {
    if (!editingChannel.value) return;
    try {
        await adminApi.updateChannel(editingChannel.value.id, {
            display_name: channelForm.value.display_name,
            purpose: channelForm.value.purpose
        });
        showEditChannelModal.value = false;
        await fetchChannels();
    } catch (e) {
        console.error('Failed to update channel', e);
        alert('Failed to update channel');
    }
}

async function deleteChannel(channel: AdminChannel) {
    if (!confirm(`Are you sure you want to delete the channel "#${channel.name}"?`)) {
        return;
    }

    try {
        await adminApi.deleteChannel(channel.id);
        await fetchChannels();
    } catch (e) {
        console.error('Failed to delete channel', e);
        alert('Failed to delete channel');
    }
}

// Member Actions
async function searchUsers() {
    if (!memberSearch.value) {
        memberSearchResults.value = [];
        return;
    }
    searchingMembers.value = true;
    try {
        const { data } = await adminApi.listUsers({ search: memberSearch.value, per_page: 5 });
        // Filter out existing members
        const currentIds = new Set(teamMembers.value.map(m => m.user_id));
        memberSearchResults.value = data.users.filter(u => !currentIds.has(u.id));
    } catch (e) {
        console.error('Failed to search users', e);
    } finally {
        searchingMembers.value = false;
    }
}

async function addMember(user: AdminUser) {
    if (!selectedTeam.value) return;
    try {
        await adminApi.addTeamMember(selectedTeam.value.id, user.id);
        memberSearchResults.value = memberSearchResults.value.filter(u => u.id !== user.id);
        await fetchMembers();
    } catch (e) {
        console.error('Failed to add member', e);
        alert('Failed to add member');
    }
}

async function removeMember(member: any) {
    if (!selectedTeam.value) return;
    if (!confirm(`Remove ${member.username} from the team?`)) return;
    try {
        await adminApi.removeTeamMember(selectedTeam.value.id, member.user_id);
        await fetchMembers();
    } catch (e) {
        console.error('Failed to remove member', e);
        alert('Failed to remove member');
    }
}

// Watchers
watch([search, page], () => {
    fetchTeams();
});

watch(activeTab, (tab) => {
    if (tab === 'members') fetchMembers();
    else fetchChannels();
});

let searchTimeout: ReturnType<typeof setTimeout>;
watch(memberSearch, () => {
    clearTimeout(searchTimeout);
    searchTimeout = setTimeout(searchUsers, 300);
});

onMounted(fetchTeams);
</script>

<template>
    <div class="space-y-6">
        <!-- Header -->
        <div class="flex items-center justify-between">
            <div>
                <div v-if="selectedTeam" class="flex items-center space-x-2 mb-2">
                    <button @click="closeDetails" class="text-text-3 hover:text-text-2">
                        <ArrowLeft class="w-4 h-4" />
                    </button>
                    <span class="text-text-4">Teams</span>
                </div>
                <h1 class="text-2xl font-bold text-text-1">
                    {{ selectedTeam ? (selectedTeam.display_name || selectedTeam.name) : 'Teams & Channels' }}
                </h1>
                <p class="text-text-3 mt-1">
                    {{ selectedTeam ? 'Manage channels and visibility for this team' : 'Manage teams, channels, and memberships' }}
                </p>
            </div>
        </div>

        <!-- Team List View -->
        <div v-if="!selectedTeam" class="space-y-4">
            <!-- Search & Filters -->
            <div class="flex flex-col md:flex-row md:items-center justify-between gap-4 bg-bg-surface-1 p-4 rounded-xl border border-border-1">
                <div class="relative flex-1 max-w-md">
                    <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-text-4" />
                    <input 
                        v-model="search"
                        type="text" 
                        placeholder="Search teams..." 
                        class="w-full pl-10 pr-4 py-2 bg-bg-surface-2 border border-border-1 rounded-lg focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none text-sm transition-all"
                    />
                </div>
            </div>

            <!-- Teams Table -->
            <div class="bg-bg-surface-1 rounded-xl border border-border-1 overflow-hidden shadow-sm">
                <div class="overflow-x-auto">
                    <table class="w-full text-left">
                        <thead>
                            <tr class="bg-bg-surface-2 border-b border-border-1">
                                <th class="px-6 py-4 text-xs font-semibold text-text-3 uppercase tracking-wider">Team</th>
                                <th class="px-6 py-4 text-xs font-semibold text-text-3 uppercase tracking-wider">Visibility</th>
                                <th class="px-6 py-4 text-xs font-semibold text-text-3 uppercase tracking-wider text-center">Members</th>
                                <th class="px-6 py-4 text-xs font-semibold text-text-3 uppercase tracking-wider text-center">Channels</th>
                                <th class="px-6 py-4 text-xs font-semibold text-text-3 uppercase tracking-wider">Created</th>
                                <th class="px-6 py-4 text-xs font-semibold text-text-3 uppercase tracking-wider text-right">Actions</th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-border-1">
                            <tr v-if="loading" v-for="i in 3" :key="i" class="animate-pulse">
                                <td v-for="j in 6" :key="j" class="px-6 py-4">
                                    <div class="h-4 bg-bg-surface-2 rounded w-2/3"></div>
                                </td>
                            </tr>
                            <tr v-else-if="teams.length === 0" class="text-center py-12">
                                <td colspan="6" class="px-6 py-12">
                                    <div class="flex flex-col items-center justify-center">
                                        <Building2 class="w-12 h-12 text-text-4 mb-4" />
                                        <p class="text-text-3">No teams found</p>
                                    </div>
                                </td>
                            </tr>
                            <tr v-for="team in teams" :key="team.id" class="hover:bg-bg-surface-2 transition-colors">
                                <td class="px-6 py-4">
                                    <div class="flex items-center space-x-3">
                                        <div class="w-10 h-10 rounded-lg bg-primary/10 flex items-center justify-center text-primary font-bold">
                                            {{ (team.display_name || team.name).charAt(0).toUpperCase() }}
                                        </div>
                                        <div>
                                            <div class="font-medium text-text-1">{{ team.display_name || team.name }}</div>
                                            <div class="text-xs text-text-3">@{{ team.name }}</div>
                                        </div>
                                    </div>
                                </td>
                                <td class="px-6 py-4">
                                    <span v-if="team.is_public" class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-success/10 text-success border border-success/20">
                                        Public
                                    </span>
                                    <span v-else class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-primary/10 text-primary border border-primary/20">
                                        Private
                                    </span>
                                </td>
                                <td class="px-6 py-4 text-center">
                                    <div class="flex items-center justify-center space-x-1 text-text-2">
                                        <Users class="w-4 h-4" />
                                        <span>{{ team.members_count }}</span>
                                    </div>
                                </td>
                                <td class="px-6 py-4 text-center">
                                    <div class="flex items-center justify-center space-x-1 text-text-2">
                                        <MessageSquare class="w-4 h-4" />
                                        <span>{{ team.channels_count }}</span>
                                    </div>
                                </td>
                                <td class="px-6 py-4 text-sm text-text-3">
                                    {{ new Date(team.created_at).toLocaleDateString() }}
                                </td>
                                <td class="px-6 py-4 text-right">
                                    <div class="flex items-center justify-end space-x-2">
                                        <button 
                                            @click="viewTeamDetails(team)"
                                            class="p-2 text-text-4 hover:text-primary transition-colors"
                                            title="View Details"
                                        >
                                            <Eye class="w-4 h-4" />
                                        </button>
                                        <button 
                                            @click="deleteTeam(team)"
                                            class="p-2 text-text-4 hover:text-danger transition-colors"
                                            title="Delete Team"
                                        >
                                            <Trash2 class="w-4 h-4" />
                                        </button>
                                    </div>
                                </td>
                            </tr>
                        </tbody>
                    </table>
                </div>

                <!-- Pagination -->
                <div class="px-6 py-4 flex items-center justify-between border-t border-border-1 bg-bg-surface-2/50">
                    <div class="text-sm text-text-3">
                        Showing {{ ((page - 1) * perPage) + 1 }} to {{ Math.min(page * perPage, totalTeams) }} of {{ totalTeams }} teams
                    </div>
                    <div class="flex items-center space-x-2">
                        <button 
                            @click="page--"
                            :disabled="page === 1"
                            class="p-2 border border-border-1 rounded-lg hover:bg-bg-surface-1 transition-all disabled:opacity-50 disabled:cursor-not-allowed"
                        >
                            <ChevronLeft class="w-4 h-4" />
                        </button>
                        <button 
                            @click="page++"
                            :disabled="page * perPage >= totalTeams"
                            class="p-2 border border-border-1 rounded-lg hover:bg-bg-surface-2 transition-all disabled:opacity-50 disabled:cursor-not-allowed"
                        >
                            <ChevronRight class="w-4 h-4" />
                        </button>
                    </div>
                </div>
            </div>
        </div>

        <!-- Team Details View -->
        <div v-else class="space-y-6">
            <div class="bg-bg-surface-1 rounded-xl border border-border-1 shadow-sm overflow-hidden">
                <!-- Details Header with Tabs -->
                <div class="border-b border-border-1">
                    <div class="px-6 py-4">
                        <h2 class="text-xl font-bold text-text-1">{{ selectedTeam.display_name || selectedTeam.name }}</h2>
                        <div class="flex items-center space-x-4 mt-2 text-sm text-text-3">
                            <span>@{{ selectedTeam.name }}</span>
                            <span>•</span>
                            <span>{{ selectedTeam.members_count }} members</span>
                            <span>•</span>
                            <span>{{ selectedTeam.channels_count }} channels</span>
                        </div>
                    </div>
                    <div class="flex px-6 space-x-6">
                        <button 
                            @click="activeTab = 'channels'"
                            class="pb-3 text-sm font-medium border-b-2 transition-colors"
                            :class="activeTab === 'channels' ? 'border-primary text-primary' : 'border-transparent text-text-3 hover:text-text-2'"
                        >
                            Channels
                        </button>
                        <button 
                            @click="activeTab = 'members'"
                            class="pb-3 text-sm font-medium border-b-2 transition-colors"
                            :class="activeTab === 'members' ? 'border-primary text-primary' : 'border-transparent text-text-3 hover:text-text-2'"
                        >
                            Members
                        </button>
                    </div>
                </div>

                <!-- Channels Tab -->
                <div v-if="activeTab === 'channels'" class="p-6">
                    <div class="flex justify-between items-center mb-4">
                        <h3 class="font-semibold text-text-1">Team Channels</h3>
                        <BaseButton @click="openCreateChannel">
                            <Plus class="w-4 h-4 mr-2" />
                            Create Channel
                        </BaseButton>
                    </div>

                    <div v-if="channelsLoading" class="p-12 text-center">
                        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary mx-auto"></div>
                    </div>
                    
                    <div v-else-if="teamChannels.length === 0" class="p-12 text-center text-text-3">
                        No channels in this team.
                    </div>

                    <div v-else class="divide-y divide-border-1 border border-border-1 rounded-lg">
                        <div v-for="channel in teamChannels" :key="channel.id" class="px-6 py-4 flex items-center justify-between hover:bg-bg-surface-2">
                            <div class="flex items-center space-x-3">
                                <div v-if="channel.channel_type === 'public'" class="text-text-4">
                                    <Hash class="w-5 h-5" />
                                </div>
                                <div v-else class="text-primary">
                                    <Lock class="w-5 h-5" />
                                </div>
                                <div>
                                    <div class="font-medium text-text-1">
                                        {{ channel.display_name || channel.name }}
                                        <span v-if="channel.is_archived" class="ml-2 px-1.5 py-0.5 rounded text-[10px] font-bold uppercase bg-bg-surface-2 text-text-3">Archived</span>
                                    </div>
                                    <p class="text-xs text-text-3">{{ channel.purpose || 'No purpose set' }}</p>
                                </div>
                            </div>
                            <div class="flex items-center space-x-2">
                                <button 
                                    @click="openEditChannel(channel)"
                                    class="p-2 text-text-4 hover:text-primary transition-colors"
                                    title="Edit Channel"
                                >
                                    <Edit2 class="w-4 h-4" />
                                </button>
                                <button 
                                    @click="deleteChannel(channel)"
                                    class="p-2 text-text-4 hover:text-danger transition-colors"
                                    title="Delete Channel"
                                >
                                    <Trash2 class="w-4 h-4" />
                                </button>
                            </div>
                        </div>
                    </div>
                </div>

                <!-- Members Tab -->
                <div v-if="activeTab === 'members'" class="p-6">
                    <div class="flex justify-between items-center mb-4">
                        <h3 class="font-semibold text-text-1">Team Members</h3>
                        <BaseButton @click="showAddMemberModal = true">
                            <UserPlus class="w-4 h-4 mr-2" />
                            Add Member
                        </BaseButton>
                    </div>

                    <div v-if="membersLoading" class="p-12 text-center">
                        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary mx-auto"></div>
                    </div>

                    <div v-else-if="teamMembers.length === 0" class="p-12 text-center text-text-3">
                        No members found.
                    </div>

                    <div v-else class="divide-y divide-border-1 border border-border-1 rounded-lg">
                        <div v-for="member in teamMembers" :key="member.user_id" class="px-6 py-4 flex items-center justify-between hover:bg-bg-surface-2">
                            <div class="flex items-center space-x-3">
                                <div class="w-10 h-10 rounded-full bg-bg-surface-2 flex items-center justify-center text-text-2 font-bold">
                                    {{ (member.display_name || member.username).charAt(0).toUpperCase() }}
                                </div>
                                <div>
                                    <div class="font-medium text-text-1">{{ member.display_name || member.username }}</div>
                                    <div class="text-xs text-text-3">@{{ member.username }} • {{ member.role }}</div>
                                </div>
                            </div>
                            <button 
                                @click="removeMember(member)"
                                class="p-2 text-text-4 hover:text-danger transition-colors"
                                title="Remove Member"
                            >
                                <UserMinus class="w-4 h-4" />
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </div>

        <!-- Create Channel Modal -->
        <div v-if="showCreateChannelModal" class="fixed inset-0 z-50 flex items-center justify-center p-4">
            <div class="fixed inset-0 bg-black/50" @click="showCreateChannelModal = false"></div>
            <div class="relative bg-bg-surface-1 rounded-xl shadow-xl w-full max-w-md p-6 space-y-4">
                <div class="flex justify-between items-center">
                    <h3 class="text-lg font-bold text-text-1">Create Channel</h3>
                    <button @click="showCreateChannelModal = false"><X class="w-5 h-5 text-text-3" /></button>
                </div>
                <BaseInput v-model="channelForm.name" label="Name" placeholder="e.g. general" required />
                <BaseInput v-model="channelForm.display_name" label="Display Name" placeholder="e.g. General" />
                <BaseInput v-model="channelForm.purpose" label="Purpose" placeholder="Channel purpose" />
                <div>
                    <label class="block text-sm font-medium text-text-2 mb-1">Type</label>
                    <select v-model="channelForm.type" class="w-full px-3 py-2 bg-bg-surface-2 border border-border-2 rounded-lg focus:ring-2 focus:ring-primary">
                        <option value="public">Public</option>
                        <option value="private">Private</option>
                    </select>
                </div>
                <div class="flex justify-end space-x-2 pt-2">
                    <BaseButton variant="secondary" @click="showCreateChannelModal = false">Cancel</BaseButton>
                    <BaseButton @click="createChannel">Create</BaseButton>
                </div>
            </div>
        </div>

        <!-- Edit Channel Modal -->
        <div v-if="showEditChannelModal" class="fixed inset-0 z-50 flex items-center justify-center p-4">
            <div class="fixed inset-0 bg-black/50" @click="showEditChannelModal = false"></div>
            <div class="relative bg-bg-surface-1 rounded-xl shadow-xl w-full max-w-md p-6 space-y-4">
                <div class="flex justify-between items-center">
                    <h3 class="text-lg font-bold text-text-1">Edit Channel</h3>
                    <button @click="showEditChannelModal = false"><X class="w-5 h-5 text-text-3" /></button>
                </div>
                <BaseInput v-model="channelForm.name" label="Name" disabled />
                <BaseInput v-model="channelForm.display_name" label="Display Name" />
                <BaseInput v-model="channelForm.purpose" label="Purpose" />
                <div class="flex justify-end space-x-2 pt-2">
                    <BaseButton variant="secondary" @click="showEditChannelModal = false">Cancel</BaseButton>
                    <BaseButton @click="updateChannel">Save</BaseButton>
                </div>
            </div>
        </div>

        <!-- Add Member Modal -->
        <div v-if="showAddMemberModal" class="fixed inset-0 z-50 flex items-center justify-center p-4">
            <div class="fixed inset-0 bg-black/50" @click="showAddMemberModal = false"></div>
            <div class="relative bg-bg-surface-1 rounded-xl shadow-xl w-full max-w-md p-6 space-y-4">
                <div class="flex justify-between items-center">
                    <h3 class="text-lg font-bold text-text-1">Add Team Member</h3>
                    <button @click="showAddMemberModal = false"><X class="w-5 h-5 text-text-3" /></button>
                </div>
                <div class="relative">
                    <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-text-4" />
                    <input 
                        v-model="memberSearch"
                        type="text"
                        placeholder="Search users..."
                        class="w-full pl-10 pr-4 py-2 bg-bg-surface-2 border border-border-1 rounded-lg focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none text-sm"
                    />
                </div>
                <div class="max-h-60 overflow-y-auto space-y-2">
                    <div v-if="searchingMembers" class="text-center py-4 text-sm text-text-3">Searching...</div>
                    <div v-else-if="memberSearchResults.length === 0" class="text-center py-4 text-sm text-text-3">No users found</div>
                    <div v-else v-for="user in memberSearchResults" :key="user.id" class="flex items-center justify-between p-2 hover:bg-bg-surface-2 rounded-lg">
                        <div class="flex items-center space-x-3">
                            <div class="w-8 h-8 rounded-full bg-bg-surface-2 flex items-center justify-center text-text-2 font-bold text-xs">
                                {{ (user.display_name || user.username).charAt(0).toUpperCase() }}
                            </div>
                            <div>
                                <div class="text-sm font-medium text-text-1">{{ user.display_name || user.username }}</div>
                                <div class="text-xs text-text-3">@{{ user.username }}</div>
                            </div>
                        </div>
                        <button @click="addMember(user)" class="p-1.5 bg-primary/10 text-primary rounded hover:bg-primary/20 transition-colors">
                            <Plus class="w-4 h-4" />
                        </button>
                    </div>
                </div>
            </div>
        </div>

        <!-- Create Channel Modal -->
        <div v-if="showCreateChannelModal" class="fixed inset-0 z-50 flex items-center justify-center p-4">
            <div class="fixed inset-0 bg-black/50" @click="showCreateChannelModal = false"></div>
            <div class="relative bg-bg-surface-1 rounded-xl shadow-xl w-full max-w-md p-6 space-y-4">
                <div class="flex justify-between items-center">
                    <h3 class="text-lg font-bold text-text-1">Create Channel</h3>
                    <button @click="showCreateChannelModal = false"><X class="w-5 h-5 text-text-3" /></button>
                </div>
                <BaseInput v-model="channelForm.name" label="Name" placeholder="e.g. general" required />
                <BaseInput v-model="channelForm.display_name" label="Display Name" placeholder="e.g. General" />
                <BaseInput v-model="channelForm.purpose" label="Purpose" placeholder="Channel purpose" />
                <div>
                    <label class="block text-sm font-medium text-text-2 mb-1">Type</label>
                    <select v-model="channelForm.type" class="w-full px-3 py-2 bg-bg-surface-2 border border-border-2 rounded-lg focus:ring-2 focus:ring-primary">
                        <option value="public">Public</option>
                        <option value="private">Private</option>
                    </select>
                </div>
                <div class="flex justify-end space-x-2 pt-2">
                    <BaseButton variant="secondary" @click="showCreateChannelModal = false">Cancel</BaseButton>
                    <BaseButton @click="createChannel">Create</BaseButton>
                </div>
            </div>
        </div>

        <!-- Edit Channel Modal -->
        <div v-if="showEditChannelModal" class="fixed inset-0 z-50 flex items-center justify-center p-4">
            <div class="fixed inset-0 bg-black/50" @click="showEditChannelModal = false"></div>
            <div class="relative bg-bg-surface-1 rounded-xl shadow-xl w-full max-w-md p-6 space-y-4">
                <div class="flex justify-between items-center">
                    <h3 class="text-lg font-bold text-text-1">Edit Channel</h3>
                    <button @click="showEditChannelModal = false"><X class="w-5 h-5 text-text-3" /></button>
                </div>
                <BaseInput v-model="channelForm.name" label="Name" disabled />
                <BaseInput v-model="channelForm.display_name" label="Display Name" />
                <BaseInput v-model="channelForm.purpose" label="Purpose" />
                <div class="flex justify-end space-x-2 pt-2">
                    <BaseButton variant="secondary" @click="showEditChannelModal = false">Cancel</BaseButton>
                    <BaseButton @click="updateChannel">Save</BaseButton>
                </div>
            </div>
        </div>

        <!-- Add Member Modal -->
        <div v-if="showAddMemberModal" class="fixed inset-0 z-50 flex items-center justify-center p-4">
            <div class="fixed inset-0 bg-black/50" @click="showAddMemberModal = false"></div>
            <div class="relative bg-bg-surface-1 rounded-xl shadow-xl w-full max-w-md p-6 space-y-4">
                <div class="flex justify-between items-center">
                    <h3 class="text-lg font-bold text-text-1">Add Team Member</h3>
                    <button @click="showAddMemberModal = false"><X class="w-5 h-5 text-text-3" /></button>
                </div>
                <div class="relative">
                    <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-text-4" />
                    <input
                        v-model="memberSearch"
                        type="text"
                        placeholder="Search users..."
                        class="w-full pl-10 pr-4 py-2 bg-bg-surface-2 border border-border-1 rounded-lg focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none text-sm"
                    />
                </div>
                <div class="max-h-60 overflow-y-auto space-y-2">
                    <div v-if="searchingMembers" class="text-center py-4 text-sm text-text-3">Searching...</div>
                    <div v-else-if="memberSearchResults.length === 0" class="text-center py-4 text-sm text-text-3">No users found</div>
                    <div v-else v-for="user in memberSearchResults" :key="user.id" class="flex items-center justify-between p-2 hover:bg-bg-surface-2 rounded-lg">
                        <div class="flex items-center space-x-3">
                            <div class="w-8 h-8 rounded-full bg-bg-surface-2 flex items-center justify-center text-text-2 font-bold text-xs">
                                {{ (user.display_name || user.username).charAt(0).toUpperCase() }}
                            </div>
                            <div>
                                <div class="text-sm font-medium text-text-1">{{ user.display_name || user.username }}</div>
                                <div class="text-xs text-text-3">@{{ user.username }}</div>
                            </div>
                        </div>
                        <button @click="addMember(user)" class="p-1.5 bg-primary/10 text-primary rounded hover:bg-primary/20 transition-colors">
                            <Plus class="w-4 h-4" />
                        </button>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>
