<script setup lang="ts">
import { ref, onMounted, watch, computed } from 'vue';
import { useAdminStore } from '../../stores/admin';
import { useAuthStore } from '../../stores/auth';
import { Users, Plus, Search, MoreHorizontal, UserCheck, UserX, Edit2, Trash2, AlertTriangle, X, Eraser, RefreshCw, UserPlus } from 'lucide-vue-next';
import membershipPoliciesApi from '../../api/membershipPolicies';
import CreateUserModal from '../../components/modals/CreateUserModal.vue';
import EditUserModal from '../../components/modals/EditUserModal.vue';
import type { AdminUser } from '../../api/admin';

const adminStore = useAdminStore();
const authStore = useAuthStore();
const searchQuery = ref('');
const statusFilter = ref<'all' | 'active' | 'inactive'>('all');
const includeDeleted = ref(false);
const showCreateModal = ref(false);
const showEditModal = ref(false);
const editingUser = ref<AdminUser | null>(null);
const activeMenuUserId = ref<string | null>(null);
const showDeleteModal = ref(false);
const deletingUser = ref<AdminUser | null>(null);
const deleteConfirmInput = ref('');
const deleteReason = ref('');
const deleteSubmitting = ref(false);
const deleteError = ref('');
const showWipeModal = ref(false);
const wipingUser = ref<AdminUser | null>(null);
const wipeSubmitting = ref(false);
const wipeError = ref('');

// Re-sync membership state
const showResyncModal = ref(false);
const resyncingUser = ref<AdminUser | null>(null);
const resyncSubmitting = ref(false);
const resyncResult = ref<{ teams_processed: number; memberships_applied: number; memberships_failed: number } | null>(null);
const resyncError = ref('');

let searchTimeout: ReturnType<typeof setTimeout>;

onMounted(() => {
    fetchUsers();
});

function fetchUsers() {
    adminStore.fetchUsers({ 
        status: statusFilter.value,
        search: searchQuery.value || undefined,
        include_deleted: includeDeleted.value,
    });
}

// Watchers for filters
watch(statusFilter, () => {
    fetchUsers();
});

watch(searchQuery, () => {
    clearTimeout(searchTimeout);
    searchTimeout = setTimeout(() => {
        fetchUsers();
    }, 300);
});

watch(includeDeleted, () => {
    fetchUsers();
});

async function handleEdit(user: AdminUser) {
    editingUser.value = user;
    showEditModal.value = true;
    activeMenuUserId.value = null;
}

async function handleDeactivate(user: AdminUser) {
    if (!confirm(`Are you sure you want to deactivate ${user.username}?`)) return;
    try {
        await adminStore.deactivateUser(user.id);
        activeMenuUserId.value = null;
    } catch (e) {
        console.error('Failed to deactivate user', e);
    }
}

async function handleReactivate(user: AdminUser) {
    try {
        await adminStore.reactivateUser(user.id);
        activeMenuUserId.value = null;
    } catch (e) {
        console.error('Failed to reactivate user', e);
    }
}

function openDeleteModal(user: AdminUser) {
    deletingUser.value = user;
    deleteConfirmInput.value = '';
    deleteReason.value = '';
    deleteError.value = '';
    deleteSubmitting.value = false;
    showDeleteModal.value = true;
    activeMenuUserId.value = null;
}

function closeDeleteModal() {
    showDeleteModal.value = false;
    deletingUser.value = null;
    deleteConfirmInput.value = '';
    deleteReason.value = '';
    deleteError.value = '';
    deleteSubmitting.value = false;
}

const canDeleteSelectedUser = computed(() => {
    const user = deletingUser.value;
    if (!user) return false;
    const typed = deleteConfirmInput.value.trim();
    return typed === user.username || typed === user.email;
});

const isGlobalAdmin = computed(() => authStore.user?.role === 'system_admin');

async function confirmDeleteUser() {
    if (!deletingUser.value || !canDeleteSelectedUser.value) return;
    deleteSubmitting.value = true;
    deleteError.value = '';
    try {
        await adminStore.deleteUser(deletingUser.value.id, {
            confirm: deleteConfirmInput.value.trim(),
            reason: deleteReason.value.trim() || undefined,
        });
        closeDeleteModal();
        await adminStore.fetchUsers({
            status: statusFilter.value,
            search: searchQuery.value || undefined,
            include_deleted: includeDeleted.value,
        });
    } catch (e: any) {
        deleteError.value = e?.response?.data?.error?.message || e?.response?.data?.message || 'Failed to delete user';
    } finally {
        deleteSubmitting.value = false;
    }
}

function openWipeModal(user: AdminUser) {
    wipingUser.value = user;
    wipeSubmitting.value = false;
    wipeError.value = '';
    showWipeModal.value = true;
    activeMenuUserId.value = null;
}

function closeWipeModal() {
    showWipeModal.value = false;
    wipingUser.value = null;
    wipeSubmitting.value = false;
    wipeError.value = '';
}

async function confirmWipeUser() {
    if (!wipingUser.value) return;
    wipeSubmitting.value = true;
    wipeError.value = '';
    try {
        await adminStore.wipeUser(wipingUser.value.id);
        closeWipeModal();
        await adminStore.fetchUsers({
            status: statusFilter.value,
            search: searchQuery.value || undefined,
            include_deleted: includeDeleted.value,
        });
    } catch (e: any) {
        wipeError.value = e?.response?.data?.error?.message || e?.response?.data?.message || 'Failed to wipe user';
    } finally {
        wipeSubmitting.value = false;
    }
}

function toggleMenu(userId: string) {
    if (activeMenuUserId.value === userId) {
        activeMenuUserId.value = null;
    } else {
        activeMenuUserId.value = userId;
    }
}

// Close menu when clicking outside (simple implementation)
function closeMenu() {
    activeMenuUserId.value = null;
}

// Re-sync membership functions
function openResyncModal(user: AdminUser) {
    resyncingUser.value = user;
    resyncSubmitting.value = false;
    resyncResult.value = null;
    resyncError.value = '';
    showResyncModal.value = true;
    activeMenuUserId.value = null;
}

function closeResyncModal() {
    showResyncModal.value = false;
    resyncingUser.value = null;
    resyncResult.value = null;
    resyncError.value = '';
}

async function confirmResyncUser() {
    if (!resyncingUser.value) return;
    resyncSubmitting.value = true;
    resyncError.value = '';
    try {
        const response = await membershipPoliciesApi.resyncUser(resyncingUser.value.id);
        resyncResult.value = response.data;
    } catch (e: any) {
        resyncError.value = e?.response?.data?.error?.message || e?.response?.data?.message || 'Failed to re-sync user memberships';
    } finally {
        resyncSubmitting.value = false;
    }
}

const roleColors: Record<string, string> = {
    system_admin: 'bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400',
    org_admin: 'bg-purple-100 text-purple-800 dark:bg-purple-900/30 dark:text-purple-400',
    team_admin: 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-400',
    member: 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-300',
    guest: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-400',
};

const formatDate = (date: string | null) => {
    if (!date) return 'Never';
    return new Date(date).toLocaleDateString();
};

const isDeleted = (user: AdminUser) => Boolean(user.deleted_at);
</script>

<template>
    <div class="space-y-6" @click="closeMenu">
        <!-- Header -->
        <div class="flex items-center justify-between">
            <div>
                <h1 class="text-2xl font-bold text-gray-900 dark:text-white">User Management</h1>
                <p class="text-gray-500 dark:text-gray-400 mt-1">Manage users, roles, and permissions</p>
            </div>
            <button 
                @click.stop="showCreateModal = true"
                class="flex items-center px-4 py-2 bg-indigo-600 hover:bg-indigo-700 text-white rounded-lg font-medium transition-colors"
            >
                <Plus class="w-5 h-5 mr-2" />
                Add User
            </button>
        </div>

        <!-- Filters -->
        <div class="flex items-center space-x-4">
            <div class="relative flex-1 max-w-md">
                <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-400" />
                <input 
                    v-model="searchQuery"
                    type="text"
                    placeholder="Search by name or email..."
                    class="w-full pl-10 pr-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-slate-800 text-gray-900 dark:text-white focus:ring-2 focus:ring-indigo-500 focus:border-transparent"
                />
            </div>
            <select 
                v-model="statusFilter"
                class="px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-slate-800 text-gray-900 dark:text-white"
            >
                <option value="all">All Users</option>
                <option value="active">Active</option>
                <option value="inactive">Inactive</option>
            </select>
            <label class="inline-flex items-center gap-2 text-sm text-gray-700 dark:text-gray-300">
                <input v-model="includeDeleted" type="checkbox" class="w-4 h-4 text-indigo-600 rounded" />
                Show deleted
            </label>
        </div>

        <!-- Users Table -->
        <div class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-gray-200 dark:border-slate-700 overflow-hidden">
            <table class="min-w-full divide-y divide-gray-200 dark:divide-slate-700">
                <thead class="bg-gray-50 dark:bg-slate-900">
                    <tr>
                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">User</th>
                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Role</th>
                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Status</th>
                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Last Login</th>
                        <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Actions</th>
                    </tr>
                </thead>
                <tbody class="divide-y divide-gray-200 dark:divide-slate-700">
                    <tr
                        v-for="user in adminStore.users"
                        :key="user.id"
                        :class="[
                            'hover:bg-gray-50 dark:hover:bg-slate-700/50',
                            isDeleted(user) ? 'opacity-80' : ''
                        ]"
                    >
                        <td class="px-6 py-4 whitespace-nowrap">
                            <div class="flex items-center">
                                <div class="w-10 h-10 rounded-full bg-indigo-600 flex items-center justify-center text-white font-bold">
                                    {{ user.username.charAt(0).toUpperCase() }}
                                </div>
                                <div class="ml-4">
                                    <div class="text-sm font-medium text-gray-900 dark:text-white flex items-center gap-2">
                                        {{ user.display_name || user.username }}
                                        <span v-if="isDeleted(user)" class="px-2 py-0.5 text-xs rounded bg-rose-100 text-rose-700 dark:bg-rose-900/30 dark:text-rose-300">
                                            Deleted
                                        </span>
                                    </div>
                                    <div class="text-sm text-gray-500 dark:text-gray-400">{{ user.email }}</div>
                                    <div v-if="isDeleted(user) && user.delete_reason" class="text-xs text-rose-600 dark:text-rose-300">
                                        Reason: {{ user.delete_reason }}
                                    </div>
                                </div>
                            </div>
                        </td>
                        <td class="px-6 py-4 whitespace-nowrap">
                            <span :class="[roleColors[user.role] || roleColors.member, 'px-2 py-1 text-xs font-medium rounded-full']">
                                {{ user.role.replace('_', ' ') }}
                            </span>
                        </td>
                        <td class="px-6 py-4 whitespace-nowrap">
                            <span v-if="isDeleted(user)" class="flex items-center text-rose-600 dark:text-rose-400">
                                <Trash2 class="w-4 h-4 mr-1" /> Deleted
                            </span>
                            <span v-else-if="user.is_active" class="flex items-center text-green-600 dark:text-green-400">
                                <UserCheck class="w-4 h-4 mr-1" /> Active
                            </span>
                            <span v-else class="flex items-center text-gray-500">
                                <UserX class="w-4 h-4 mr-1" /> Inactive
                            </span>
                        </td>
                        <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                            {{ formatDate(user.last_login_at) }}
                        </td>
                        <td class="px-6 py-4 whitespace-nowrap text-right text-sm font-medium relative">
                            <button 
                                @click.stop="handleEdit(user)"
                                class="text-indigo-600 hover:text-indigo-900 dark:text-indigo-400 mr-3"
                                title="Edit User"
                            >
                                <Edit2 class="w-4 h-4" />
                            </button>
                            <div class="relative inline-block text-left">
                                <button 
                                    @click.stop="toggleMenu(user.id)"
                                    class="text-gray-400 hover:text-gray-600"
                                >
                                    <MoreHorizontal class="w-4 h-4" />
                                </button>
                                <!-- Dropdown -->
                                <div 
                                    v-if="activeMenuUserId === user.id"
                                    class="absolute right-0 mt-2 w-48 bg-white dark:bg-slate-800 rounded-md shadow-lg py-1 z-10 border border-gray-200 dark:border-slate-700 ring-1 ring-black ring-opacity-5"
                                >
                                    <button
                                        v-if="user.is_active && !isDeleted(user)"
                                        @click.stop="handleDeactivate(user)"
                                        class="flex w-full items-center px-4 py-2 text-sm text-red-600 hover:bg-gray-100 dark:hover:bg-slate-700"
                                    >
                                        <UserX class="w-4 h-4 mr-2" />
                                        Deactivate User
                                    </button>
                                    <button
                                        v-else-if="!isDeleted(user)"
                                        @click.stop="handleReactivate(user)"
                                        class="flex w-full items-center px-4 py-2 text-sm text-green-600 hover:bg-gray-100 dark:hover:bg-slate-700"
                                    >
                                        <UserCheck class="w-4 h-4 mr-2" />
                                        Reactivate User
                                    </button>
                                    <button
                                        v-if="isGlobalAdmin && !isDeleted(user)"
                                        @click.stop="openDeleteModal(user)"
                                        class="flex w-full items-center px-4 py-2 text-sm text-rose-700 hover:bg-gray-100 dark:hover:bg-slate-700 dark:text-rose-400"
                                    >
                                        <Trash2 class="w-4 h-4 mr-2" />
                                        Delete User
                                    </button>
                                    <button
                                        v-if="isGlobalAdmin && isDeleted(user)"
                                        @click.stop="openWipeModal(user)"
                                        class="flex w-full items-center px-4 py-2 text-sm text-orange-700 hover:bg-gray-100 dark:hover:bg-slate-700 dark:text-orange-400"
                                    >
                                        <Eraser class="w-4 h-4 mr-2" />
                                        Wipe User
                                    </button>
                                    <button
                                        v-if="!isDeleted(user)"
                                        @click.stop="openResyncModal(user)"
                                        class="flex w-full items-center px-4 py-2 text-sm text-indigo-600 hover:bg-gray-100 dark:hover:bg-slate-700 dark:text-indigo-400"
                                    >
                                        <RefreshCw class="w-4 h-4 mr-2" />
                                        Re-sync Memberships
                                    </button>
                                </div>
                            </div>
                        </td>
                    </tr>
                    <tr v-if="adminStore.users.length === 0 && !adminStore.loading">
                        <td colspan="5" class="px-6 py-12 text-center text-gray-500">
                            <Users class="w-12 h-12 mx-auto mb-4 text-gray-300 dark:text-gray-600" />
                            <p>No users found matching your criteria</p>
                        </td>
                    </tr>
                </tbody>
            </table>
        </div>

        <CreateUserModal 
            :open="showCreateModal" 
            @close="showCreateModal = false"
            @created="fetchUsers"
        />

        <EditUserModal
            :open="showEditModal"
            :user="editingUser"
            @close="showEditModal = false; editingUser = null"
            @updated="fetchUsers"
        />

        <div
            v-if="showDeleteModal && deletingUser"
            class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4"
            @click.self="closeDeleteModal"
        >
            <div class="w-full max-w-lg rounded-xl border border-gray-200 dark:border-slate-700 bg-white dark:bg-slate-800 shadow-xl">
                <div class="flex items-start justify-between p-5 border-b border-gray-200 dark:border-slate-700">
                    <div class="flex items-start gap-3">
                        <div class="rounded-full bg-rose-100 p-2 dark:bg-rose-900/30">
                            <AlertTriangle class="w-5 h-5 text-rose-700 dark:text-rose-300" />
                        </div>
                        <div>
                            <h3 class="text-lg font-semibold text-gray-900 dark:text-white">Delete User</h3>
                            <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">
                                This soft-deletes the account, revokes access, and hides it from normal user lists.
                            </p>
                        </div>
                    </div>
                    <button @click="closeDeleteModal" class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200">
                        <X class="w-5 h-5" />
                    </button>
                </div>

                <div class="p-5 space-y-4">
                    <div class="rounded-lg border border-rose-200 bg-rose-50 p-3 text-sm text-rose-800 dark:border-rose-900/40 dark:bg-rose-900/20 dark:text-rose-200">
                        <p><strong>Consequences:</strong> login is blocked immediately, active sessions are revoked, and the account is marked deleted for audit/history. Messages remain for history.</p>
                    </div>

                    <div class="text-sm text-gray-700 dark:text-gray-300">
                        <div>Type the exact <span class="font-semibold">username</span> or <span class="font-semibold">email</span> to confirm deletion.</div>
                        <div class="mt-2 rounded-md bg-gray-50 dark:bg-slate-900 px-3 py-2 font-mono text-xs break-all">
                            {{ deletingUser.username }} or {{ deletingUser.email }}
                        </div>
                    </div>

                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Confirmation text</label>
                        <input
                            v-model="deleteConfirmInput"
                            type="text"
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-slate-900 text-gray-900 dark:text-white"
                            :placeholder="deletingUser.username"
                        />
                    </div>

                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Reason (optional)</label>
                        <textarea
                            v-model="deleteReason"
                            rows="3"
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-slate-900 text-gray-900 dark:text-white"
                            placeholder="Why this account is being deleted"
                        />
                    </div>

                    <div v-if="deleteError" class="rounded-lg border border-rose-200 bg-rose-50 px-3 py-2 text-sm text-rose-700 dark:border-rose-900/40 dark:bg-rose-900/20 dark:text-rose-300">
                        {{ deleteError }}
                    </div>
                </div>

                <div class="flex items-center justify-end gap-3 p-5 border-t border-gray-200 dark:border-slate-700">
                    <button
                        @click="closeDeleteModal"
                        class="px-4 py-2 rounded-lg border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-200"
                    >
                        Cancel
                    </button>
                    <button
                        @click="confirmDeleteUser"
                        :disabled="deleteSubmitting || !canDeleteSelectedUser"
                        class="px-4 py-2 rounded-lg bg-rose-600 hover:bg-rose-700 disabled:opacity-50 text-white font-medium"
                    >
                        {{ deleteSubmitting ? 'Deleting...' : 'Delete User' }}
                    </button>
                </div>
            </div>
        </div>

        <!-- Wipe User Modal -->
        <div
            v-if="showWipeModal && wipingUser"
            class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4"
            @click.self="closeWipeModal"
        >
            <div class="w-full max-w-lg rounded-xl border border-gray-200 dark:border-slate-700 bg-white dark:bg-slate-800 shadow-xl">
                <div class="flex items-start justify-between p-5 border-b border-gray-200 dark:border-slate-700">
                    <div class="flex items-start gap-3">
                        <div class="rounded-full bg-orange-100 p-2 dark:bg-orange-900/30">
                            <Eraser class="w-5 h-5 text-orange-700 dark:text-orange-300" />
                        </div>
                        <div>
                            <h3 class="text-lg font-semibold text-gray-900 dark:text-white">Wipe User</h3>
                            <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">
                                Permanently delete this user from the database. This action cannot be undone.
                            </p>
                        </div>
                    </div>
                    <button @click="closeWipeModal" class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200">
                        <X class="w-5 h-5" />
                    </button>
                </div>

                <div class="p-5 space-y-4">
                    <div class="rounded-lg border border-orange-200 bg-orange-50 p-3 text-sm text-orange-800 dark:border-orange-900/40 dark:bg-orange-900/20 dark:text-orange-200">
                        <p><strong>Warning:</strong> This will permanently remove the user record from the database. Only users with no messages can be wiped.</p>
                    </div>

                    <div class="text-sm text-gray-700 dark:text-gray-300">
                        <p>You are about to wipe user:</p>
                        <div class="mt-2 rounded-md bg-gray-50 dark:bg-slate-900 px-3 py-2 font-mono text-xs break-all">
                            {{ wipingUser.username }} ({{ wipingUser.email }})
                        </div>
                        <p class="mt-2 text-xs text-gray-500">Deleted at: {{ formatDate(wipingUser.deleted_at ?? null) }}</p>
                    </div>

                    <div v-if="wipeError" class="rounded-lg border border-rose-200 bg-rose-50 px-3 py-2 text-sm text-rose-700 dark:border-rose-900/40 dark:bg-rose-900/20 dark:text-rose-300">
                        {{ wipeError }}
                    </div>
                </div>

                <div class="flex items-center justify-end gap-3 p-5 border-t border-gray-200 dark:border-slate-700">
                    <button
                        @click="closeWipeModal"
                        class="px-4 py-2 rounded-lg border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-200"
                    >
                        Cancel
                    </button>
                    <button
                        @click="confirmWipeUser"
                        :disabled="wipeSubmitting"
                        class="px-4 py-2 rounded-lg bg-orange-600 hover:bg-orange-700 disabled:opacity-50 text-white font-medium"
                    >
                        {{ wipeSubmitting ? 'Wiping...' : 'Permanently Wipe User' }}
                    </button>
                </div>
            </div>
        </div>
    </div>
</template>
