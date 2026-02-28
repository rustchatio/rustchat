<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { 
    X, Trash2, Building2, Hash, AlertCircle
} from 'lucide-vue-next';
import membershipPoliciesApi, { 
    type PolicyWithTargets,
    type CreatePolicyRequest,
    type CreatePolicyTarget,
    type PolicyScopeType,
    type PolicySourceType
} from '../../api/membershipPolicies';
import { teamsApi } from '../../api/teams';
import { channelsApi } from '../../api/channels';
import { useToast } from '../../composables/useToast';

const props = defineProps<{
    policy: PolicyWithTargets | null;
}>();

const emit = defineEmits<{
    (e: 'close'): void;
    (e: 'saved', policy: PolicyWithTargets): void;
}>();

const toast = useToast();

// Form state
const name = ref('');
const description = ref('');
const scopeType = ref<PolicyScopeType>('global');
const teamId = ref('');
const sourceType = ref<PolicySourceType>('all_users');
const sourceConfig = ref<Record<string, any>>({});
const enabled = ref(true);
const priority = ref(0);
const targets = ref<CreatePolicyTarget[]>([]);

// Available options
const teams = ref<{ id: string; name: string; display_name: string }[]>([]);
const channels = ref<{ id: string; name: string; display_name: string; team_id: string }[]>([]);
const loadingOptions = ref(false);
const saving = ref(false);
const showPreview = ref(false);

// Source type options
const sourceTypeOptions = [
    { value: 'all_users' as PolicySourceType, label: 'All Users', description: 'Apply to all users' },
    { value: 'auth_service' as PolicySourceType, label: 'Auth Service', description: 'Users from specific auth provider' },
    { value: 'group' as PolicySourceType, label: 'Group', description: 'Members of specific groups' },
    { value: 'role' as PolicySourceType, label: 'Role', description: 'Users with specific roles' },
    { value: 'org' as PolicySourceType, label: 'Organization', description: 'Users from specific organization' },
];

// Computed
const isEditing = computed(() => !!props.policy);
const modalTitle = computed(() => isEditing.value ? 'Edit Policy' : 'New Policy');

// Available targets based on scope
const availableTargets = computed(() => {
    if (scopeType.value === 'team' && teamId.value) {
        // For team-scoped policies, show channels in that team
        return {
            teams: teams.value.filter(t => t.id === teamId.value),
            channels: channels.value.filter(c => c.team_id === teamId.value)
        };
    }
    // For global policies, show all teams and their channels
    return {
        teams: teams.value,
        channels: channels.value
    };
});

// Load data
async function loadOptions() {
    loadingOptions.value = true;
    try {
        // Load teams first
        const teamsRes = await teamsApi.list();
        teams.value = teamsRes.data;
        
        // Load channels for each team
        const allChannels: any[] = [];
        for (const team of teams.value) {
            try {
                const channelsRes = await channelsApi.list(team.id);
                allChannels.push(...channelsRes.data);
            } catch (e) {
                // Skip teams we can't access
            }
        }
        channels.value = allChannels;
    } catch (error: any) {
        toast.error('Failed to load teams and channels', error.message);
    } finally {
        loadingOptions.value = false;
    }
}

// Initialize form
function initForm() {
    if (props.policy) {
        name.value = props.policy.name;
        description.value = props.policy.description || '';
        scopeType.value = props.policy.scope_type;
        teamId.value = props.policy.team_id || '';
        sourceType.value = props.policy.source_type;
        sourceConfig.value = { ...props.policy.source_config };
        enabled.value = props.policy.enabled;
        priority.value = props.policy.priority;
        targets.value = props.policy.targets.map(t => ({
            target_type: t.target_type,
            target_id: t.target_id,
            role_mode: t.role_mode
        }));
    } else {
        // Defaults for new policy
        name.value = '';
        description.value = '';
        scopeType.value = 'global';
        teamId.value = '';
        sourceType.value = 'all_users';
        sourceConfig.value = {};
        enabled.value = true;
        priority.value = 0;
        targets.value = [];
    }
}

// Add target
function addTeamTarget(team: typeof teams.value[0]) {
    if (!targets.value.some(t => t.target_type === 'team' && t.target_id === team.id)) {
        targets.value.push({
            target_type: 'team',
            target_id: team.id,
            role_mode: 'member'
        });
    }
}

function addChannelTarget(channel: typeof channels.value[0]) {
    if (!targets.value.some(t => t.target_type === 'channel' && t.target_id === channel.id)) {
        targets.value.push({
            target_type: 'channel',
            target_id: channel.id,
            role_mode: 'member'
        });
    }
}

// Remove target
function removeTarget(index: number) {
    targets.value.splice(index, 1);
}



// Get target name
function getTargetName(target: CreatePolicyTarget) {
    if (target.target_type === 'team') {
        const team = teams.value.find(t => t.id === target.target_id);
        return team?.display_name || team?.name || 'Unknown Team';
    } else {
        const channel = channels.value.find(c => c.id === target.target_id);
        return channel?.display_name || channel?.name || 'Unknown Channel';
    }
}

// Preview
async function previewImpact() {
    if (!validateForm()) return;
    showPreview.value = true;
}

// Validation
function validateForm(): boolean {
    if (!name.value.trim()) {
        toast.error('Policy name is required');
        return false;
    }
    if (scopeType.value === 'team' && !teamId.value) {
        toast.error('Team is required for team-scoped policies');
        return false;
    }
    if (targets.value.length === 0) {
        toast.error('At least one target is required');
        return false;
    }
    return true;
}

// Save
async function savePolicy() {
    if (!validateForm()) return;
    
    saving.value = true;
    try {
        const request: CreatePolicyRequest = {
            name: name.value.trim(),
            description: description.value.trim() || undefined,
            scope_type: scopeType.value,
            team_id: scopeType.value === 'team' ? teamId.value : undefined,
            source_type: sourceType.value,
            source_config: sourceConfig.value,
            enabled: enabled.value,
            priority: priority.value,
            targets: targets.value
        };

        let response;
        if (isEditing.value && props.policy) {
            response = await membershipPoliciesApi.updatePolicy(props.policy.id, request);
        } else {
            response = await membershipPoliciesApi.createPolicy(request);
        }

        emit('saved', response.data);
    } catch (error: any) {
        toast.error('Failed to save policy', error.response?.data?.message || error.message);
    } finally {
        saving.value = false;
    }
}

// Cancel
function cancel() {
    emit('close');
}

onMounted(() => {
    initForm();
    loadOptions();
});
</script>

<template>
    <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="cancel">
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-4xl w-full mx-4 max-h-[90vh] flex flex-col">
            <!-- Header -->
            <div class="flex items-center justify-between p-6 border-b border-gray-200 dark:border-gray-700">
                <h2 class="text-xl font-bold text-gray-900 dark:text-white">{{ modalTitle }}</h2>
                <button @click="cancel" class="p-2 text-gray-500 hover:text-gray-700 rounded-lg">
                    <X class="w-5 h-5" />
                </button>
            </div>

            <!-- Form -->
            <div class="flex-1 overflow-auto p-6 space-y-6">
                <!-- Basic Info -->
                <div class="space-y-4">
                    <h3 class="text-sm font-medium text-gray-900 dark:text-white uppercase tracking-wider">Basic Information</h3>
                    
                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Policy Name *</label>
                        <input
                            v-model="name"
                            type="text"
                            placeholder="e.g., Engineering Team Auto-Join"
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                        />
                    </div>

                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Description</label>
                        <textarea
                            v-model="description"
                            rows="2"
                            placeholder="Brief description of this policy..."
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                        />
                    </div>

                    <div class="grid grid-cols-2 gap-4">
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Scope *</label>
                            <select
                                v-model="scopeType"
                                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                            >
                                <option value="global">Global (all teams)</option>
                                <option value="team">Specific Team</option>
                            </select>
                        </div>

                        <div v-if="scopeType === 'team'">
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Team *</label>
                            <select
                                v-model="teamId"
                                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                            >
                                <option value="">Select a team...</option>
                                <option v-for="team in teams" :key="team.id" :value="team.id">
                                    {{ team.display_name || team.name }}
                                </option>
                            </select>
                        </div>
                    </div>
                </div>

                <!-- Source -->
                <div class="space-y-4">
                    <h3 class="text-sm font-medium text-gray-900 dark:text-white uppercase tracking-wider">Applies To</h3>
                    
                    <div class="grid grid-cols-1 gap-3">
                        <label
                            v-for="option in sourceTypeOptions"
                            :key="option.value"
                            class="flex items-start p-4 border rounded-lg cursor-pointer transition-colors"
                            :class="[
                                sourceType === option.value
                                    ? 'border-indigo-500 bg-indigo-50 dark:bg-indigo-900/20'
                                    : 'border-gray-300 dark:border-gray-600 hover:bg-gray-50 dark:hover:bg-gray-700'
                            ]"
                        >
                            <input
                                v-model="sourceType"
                                type="radio"
                                :value="option.value"
                                class="mt-1 w-4 h-4 text-indigo-600"
                            />
                            <div class="ml-3">
                                <span class="block font-medium text-gray-900 dark:text-white">{{ option.label }}</span>
                                <span class="block text-sm text-gray-500">{{ option.description }}</span>
                            </div>
                        </label>
                    </div>

                    <!-- Source Config (conditional) -->
                    <div v-if="sourceType === 'auth_service'" class="pl-4 border-l-2 border-indigo-200">
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Auth Provider</label>
                        <select
                            v-model="sourceConfig.auth_service"
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                        >
                            <option value="">Any</option>
                            <option value="ldap">LDAP</option>
                            <option value="saml">SAML</option>
                            <option value="oauth">OAuth</option>
                            <option value="gitlab">GitLab</option>
                            <option value="github">GitHub</option>
                            <option value="google">Google</option>
                        </select>
                    </div>
                </div>

                <!-- Targets -->
                <div class="space-y-4">
                    <h3 class="text-sm font-medium text-gray-900 dark:text-white uppercase tracking-wider">Target Memberships *</h3>
                    
                    <div v-if="loadingOptions" class="text-center py-4 text-gray-500">
                        Loading teams and channels...
                    </div>

                    <div v-else class="space-y-4">
                        <!-- Selected Targets -->
                        <div v-if="targets.length > 0" class="space-y-2">
                            <div
                                v-for="(target, index) in targets"
                                :key="index"
                                class="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-700 rounded-lg"
                            >
                                <div class="flex items-center space-x-3">
                                    <component
                                        :is="target.target_type === 'team' ? Building2 : Hash"
                                        class="w-5 h-5 text-gray-400"
                                    />
                                    <span class="font-medium">{{ getTargetName(target) }}</span>
                                    <span class="text-xs text-gray-500 uppercase">{{ target.target_type }}</span>
                                </div>
                                <div class="flex items-center space-x-2">
                                    <select
                                        v-model="target.role_mode"
                                        class="text-sm px-2 py-1 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-600"
                                    >
                                        <option value="member">Member</option>
                                        <option value="admin">Admin</option>
                                    </select>
                                    <button
                                        @click="removeTarget(index)"
                                        class="p-1 text-red-500 hover:bg-red-50 rounded"
                                    >
                                        <Trash2 class="w-4 h-4" />
                                    </button>
                                </div>
                            </div>
                        </div>

                        <!-- Available Teams -->
                        <div>
                            <h4 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">Add Team</h4>
                            <div class="flex flex-wrap gap-2">
                                <button
                                    v-for="team in availableTargets.teams"
                                    :key="team.id"
                                    @click="addTeamTarget(team)"
                                    :disabled="targets.some(t => t.target_type === 'team' && t.target_id === team.id)"
                                    class="px-3 py-1.5 text-sm border rounded-lg transition-colors"
                                    :class="[
                                        targets.some(t => t.target_type === 'team' && t.target_id === team.id)
                                            ? 'border-gray-200 text-gray-400 cursor-not-allowed'
                                            : 'border-gray-300 hover:border-indigo-500 hover:text-indigo-600'
                                    ]"
                                >
                                    <Building2 class="w-4 h-4 inline mr-1" />
                                    {{ team.display_name || team.name }}
                                </button>
                            </div>
                        </div>

                        <!-- Available Channels -->
                        <div>
                            <h4 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">Add Channel</h4>
                            <div class="flex flex-wrap gap-2">
                                <button
                                    v-for="channel in availableTargets.channels"
                                    :key="channel.id"
                                    @click="addChannelTarget(channel)"
                                    :disabled="targets.some(t => t.target_type === 'channel' && t.target_id === channel.id)"
                                    class="px-3 py-1.5 text-sm border rounded-lg transition-colors"
                                    :class="[
                                        targets.some(t => t.target_type === 'channel' && t.target_id === channel.id)
                                            ? 'border-gray-200 text-gray-400 cursor-not-allowed'
                                            : 'border-gray-300 hover:border-indigo-500 hover:text-indigo-600'
                                    ]"
                                >
                                    <Hash class="w-4 h-4 inline mr-1" />
                                    {{ channel.display_name || channel.name }}
                                </button>
                            </div>
                        </div>
                    </div>
                </div>

                <!-- Settings -->
                <div class="space-y-4">
                    <h3 class="text-sm font-medium text-gray-900 dark:text-white uppercase tracking-wider">Settings</h3>
                    
                    <div class="grid grid-cols-2 gap-4">
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Priority</label>
                            <input
                                v-model.number="priority"
                                type="number"
                                min="0"
                                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                            />
                            <p class="text-xs text-gray-500 mt-1">Higher priority policies apply first</p>
                        </div>

                        <div class="flex items-center">
                            <label class="flex items-center space-x-3 cursor-pointer">
                                <input
                                    v-model="enabled"
                                    type="checkbox"
                                    class="w-5 h-5 text-indigo-600 rounded"
                                />
                                <span class="text-sm font-medium text-gray-700 dark:text-gray-300">Enabled</span>
                            </label>
                        </div>
                    </div>
                </div>
            </div>

            <!-- Footer -->
            <div class="flex items-center justify-between p-6 border-t border-gray-200 dark:border-gray-700">
                <button
                    @click="previewImpact"
                    class="flex items-center px-4 py-2 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
                >
                    <AlertCircle class="w-5 h-5 mr-2" />
                    Preview Impact
                </button>
                
                <div class="flex space-x-3">
                    <button
                        @click="cancel"
                        class="px-4 py-2 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
                    >
                        Cancel
                    </button>
                    <button
                        @click="savePolicy"
                        :disabled="saving"
                        class="flex items-center px-6 py-2 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 transition-colors disabled:opacity-50"
                    >
                        <span v-if="saving" class="mr-2">Saving...</span>
                        {{ isEditing ? 'Save Changes' : 'Create Policy' }}
                    </button>
                </div>
            </div>
        </div>
    </div>

    <!-- Preview Modal -->
    <div
        v-if="showPreview"
        class="fixed inset-0 bg-black/50 flex items-center justify-center z-[60]"
        @click.self="showPreview = false"
    >
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-lg w-full mx-4 p-6">
            <h3 class="text-lg font-bold text-gray-900 dark:text-white mb-4">Policy Preview</h3>
            <p class="text-gray-600 dark:text-gray-400 mb-4">
                This policy will affect approximately <strong>all users matching "{{ sourceTypeOptions.find(o => o.value === sourceType)?.label }}"</strong> criteria.
            </p>
            <div class="space-y-2 mb-6">
                <div class="flex justify-between text-sm">
                    <span class="text-gray-500">Target Teams:</span>
                    <span class="font-medium">{{ targets.filter(t => t.target_type === 'team').length }}</span>
                </div>
                <div class="flex justify-between text-sm">
                    <span class="text-gray-500">Target Channels:</span>
                    <span class="font-medium">{{ targets.filter(t => t.target_type === 'channel').length }}</span>
                </div>
            </div>
            <div class="flex justify-end space-x-3">
                <button
                    @click="showPreview = false"
                    class="px-4 py-2 text-gray-700 dark:text-gray-300 hover:bg-gray-100 rounded-lg"
                >
                    Close
                </button>
                <button
                    @click="showPreview = false; savePolicy()"
                    class="px-4 py-2 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700"
                >
                    Confirm & Save
                </button>
            </div>
        </div>
    </div>
</template>
