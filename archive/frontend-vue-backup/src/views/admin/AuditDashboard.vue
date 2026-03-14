<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { 
    AlertTriangle, CheckCircle, XCircle, RefreshCw, 
    Download, Filter, Activity, TrendingUp,
    FileText, ChevronDown, ChevronUp
} from 'lucide-vue-next';
import { format, subDays } from 'date-fns';
import { useToast } from '../../composables/useToast';
import api from '../../api/client';

const toast = useToast();

// State
const loading = ref(false);
const summary = ref({
    total_operations_24h: 0,
    successful_operations_24h: 0,
    failed_operations_24h: 0,
    failure_rate_24h: 0,
    pending_operations: 0,
    policies_with_failures: 0
});
const recentFailures = ref<any[]>([]);
const policyStats = ref<any[]>([]);
const auditLogs = ref<any[]>([]);
const showFilters = ref(false);

// Filters
const filters = ref({
    status: '',
    action: '',
    policy_id: '',
    from_date: format(subDays(new Date(), 7), 'yyyy-MM-dd'),
    to_date: format(new Date(), 'yyyy-MM-dd')
});

// Computed
const hasFailures = computed(() => summary.value.failed_operations_24h > 0);
const failureRateClass = computed(() => {
    const rate = summary.value.failure_rate_24h;
    if (rate < 5) return 'text-green-600';
    if (rate < 15) return 'text-yellow-600';
    return 'text-red-600';
});

// Fetch all data
async function fetchDashboard() {
    loading.value = true;
    try {
        const [summaryRes, failuresRes, statsRes] = await Promise.all([
            api.get('/admin/audit/membership/summary'),
            api.get('/admin/audit/membership/recent-failures'),
            api.get('/admin/audit/membership/failures')
        ]);
        
        summary.value = summaryRes.data;
        recentFailures.value = failuresRes.data;
        policyStats.value = statsRes.data;
    } catch (error: any) {
        toast.error('Failed to load audit data', error.message);
    } finally {
        loading.value = false;
    }
}

// Fetch audit logs with filters
async function fetchAuditLogs() {
    loading.value = true;
    try {
        const params: any = {};
        if (filters.value.status) params.status = filters.value.status;
        if (filters.value.action) params.action = filters.value.action;
        if (filters.value.from_date) params.from_date = new Date(filters.value.from_date).toISOString();
        if (filters.value.to_date) params.to_date = new Date(filters.value.to_date).toISOString();
        
        const response = await api.get('/admin/audit/membership', { params });
        auditLogs.value = response.data;
    } catch (error: any) {
        toast.error('Failed to load audit logs', error.message);
    } finally {
        loading.value = false;
    }
}

// Export logs
async function exportLogs() {
    try {
        const response = await api.get('/admin/audit/membership/export', {
            params: filters.value,
            responseType: 'blob'
        });
        
        const blob = new Blob([JSON.stringify(response.data, null, 2)], { type: 'application/json' });
        const url = window.URL.createObjectURL(blob);
        const link = document.createElement('a');
        link.href = url;
        link.download = `audit-logs-${format(new Date(), 'yyyy-MM-dd')}.json`;
        link.click();
        window.URL.revokeObjectURL(url);
        
        toast.success('Audit logs exported');
    } catch (error: any) {
        toast.error('Export failed', error.message);
    }
}

// Refresh data
function refresh() {
    fetchDashboard();
    fetchAuditLogs();
}

onMounted(() => {
    fetchDashboard();
    fetchAuditLogs();
});
</script>

<template>
    <div class="space-y-6">
        <!-- Header -->
        <div class="flex items-center justify-between">
            <div>
                <h1 class="text-2xl font-bold text-gray-900 dark:text-white">Audit Dashboard</h1>
                <p class="text-gray-500 dark:text-gray-400 mt-1">
                    Monitor membership policy execution and failures
                </p>
            </div>
            <div class="flex space-x-2">
                <button
                    @click="exportLogs"
                    class="flex items-center px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
                >
                    <Download class="w-4 h-4 mr-2" />
                    Export
                </button>
                <button
                    @click="refresh"
                    class="flex items-center px-4 py-2 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 transition-colors"
                    :disabled="loading"
                >
                    <RefreshCw class="w-4 h-4 mr-2" :class="{ 'animate-spin': loading }" />
                    Refresh
                </button>
            </div>
        </div>

        <!-- Summary Cards -->
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
            <div class="p-6 bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700">
                <div class="flex items-center justify-between">
                    <div>
                        <p class="text-sm font-medium text-gray-500 dark:text-gray-400">Total Operations (24h)</p>
                        <p class="text-2xl font-bold text-gray-900 dark:text-white mt-1">{{ summary.total_operations_24h }}</p>
                    </div>
                    <div class="p-3 bg-blue-100 dark:bg-blue-900/30 rounded-lg">
                        <Activity class="w-6 h-6 text-blue-600 dark:text-blue-400" />
                    </div>
                </div>
            </div>

            <div class="p-6 bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700">
                <div class="flex items-center justify-between">
                    <div>
                        <p class="text-sm font-medium text-gray-500 dark:text-gray-400">Successful (24h)</p>
                        <p class="text-2xl font-bold text-green-600 dark:text-green-400 mt-1">{{ summary.successful_operations_24h }}</p>
                    </div>
                    <div class="p-3 bg-green-100 dark:bg-green-900/30 rounded-lg">
                        <CheckCircle class="w-6 h-6 text-green-600 dark:text-green-400" />
                    </div>
                </div>
            </div>

            <div class="p-6 bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700">
                <div class="flex items-center justify-between">
                    <div>
                        <p class="text-sm font-medium text-gray-500 dark:text-gray-400">Failed (24h)</p>
                        <p class="text-2xl font-bold text-red-600 dark:text-red-400 mt-1">{{ summary.failed_operations_24h }}</p>
                    </div>
                    <div class="p-3 bg-red-100 dark:bg-red-900/30 rounded-lg">
                        <XCircle class="w-6 h-6 text-red-600 dark:text-red-400" />
                    </div>
                </div>
                <div v-if="hasFailures" class="mt-2 flex items-center text-sm text-red-600">
                    <AlertTriangle class="w-4 h-4 mr-1" />
                    <span>{{ summary.policies_with_failures }} policies affected</span>
                </div>
            </div>

            <div class="p-6 bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700">
                <div class="flex items-center justify-between">
                    <div>
                        <p class="text-sm font-medium text-gray-500 dark:text-gray-400">Failure Rate (24h)</p>
                        <p class="text-2xl font-bold mt-1" :class="failureRateClass">
                            {{ summary.failure_rate_24h.toFixed(1) }}%
                        </p>
                    </div>
                    <div class="p-3 bg-yellow-100 dark:bg-yellow-900/30 rounded-lg">
                        <TrendingUp class="w-6 h-6 text-yellow-600 dark:text-yellow-400" />
                    </div>
                </div>
            </div>
        </div>

        <!-- Alert Banner -->
        <div 
            v-if="hasFailures && summary.failure_rate_24h > 15" 
            class="p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg flex items-start"
        >
            <AlertTriangle class="w-5 h-5 text-red-600 dark:text-red-400 mr-3 mt-0.5" />
            <div>
                <h3 class="text-sm font-semibold text-red-800 dark:text-red-300">High Failure Rate Detected</h3>
                <p class="text-sm text-red-700 dark:text-red-400 mt-1">
                    The membership policy failure rate is {{ summary.failure_rate_24h.toFixed(1) }}% in the last 24 hours. 
                    Please review the recent failures below.
                </p>
            </div>
        </div>

        <!-- Recent Failures -->
        <div v-if="recentFailures.length > 0" class="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700">
            <div class="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
                <h2 class="text-lg font-semibold text-gray-900 dark:text-white flex items-center">
                    <AlertTriangle class="w-5 h-5 mr-2 text-red-500" />
                    Recent Failures (Last Hour)
                </h2>
            </div>
            <div class="divide-y divide-gray-200 dark:divide-gray-700">
                <div
                    v-for="failure in recentFailures.slice(0, 5)"
                    :key="failure.id"
                    class="px-6 py-4 hover:bg-gray-50 dark:hover:bg-gray-700/50"
                >
                    <div class="flex items-start justify-between">
                        <div>
                            <p class="text-sm font-medium text-gray-900 dark:text-white">
                                {{ failure.policy_name || 'Unknown Policy' }}
                            </p>
                            <p class="text-xs text-gray-500 mt-1">
                                User: @{{ failure.username || failure.user_id.slice(0, 8) }} | 
                                Target: {{ failure.target_type }} {{ failure.target_id.slice(0, 8) }}...
                            </p>
                            <p class="text-xs text-red-600 mt-1 font-mono">{{ failure.error_message }}</p>
                        </div>
                        <span class="text-xs text-gray-400">
                            {{ format(new Date(failure.created_at), 'HH:mm:ss') }}
                        </span>
                    </div>
                </div>
            </div>
        </div>

        <!-- Policy Failure Stats -->
        <div v-if="policyStats.length > 0" class="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700">
            <div class="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
                <h2 class="text-lg font-semibold text-gray-900 dark:text-white">Policies with Failures</h2>
            </div>
            <table class="w-full">
                <thead class="bg-gray-50 dark:bg-gray-900">
                    <tr>
                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Policy</th>
                        <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase">Total</th>
                        <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase">Failed</th>
                        <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase">Rate</th>
                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Last Error</th>
                    </tr>
                </thead>
                <tbody class="divide-y divide-gray-200 dark:divide-gray-700">
                    <tr v-for="stat in policyStats" :key="stat.policy_id" class="hover:bg-gray-50 dark:hover:bg-gray-700/50">
                        <td class="px-6 py-4 text-sm font-medium text-gray-900 dark:text-white">{{ stat.policy_name }}</td>
                        <td class="px-6 py-4 text-sm text-right text-gray-600 dark:text-gray-400">{{ stat.total_operations }}</td>
                        <td class="px-6 py-4 text-sm text-right text-red-600 dark:text-red-400 font-semibold">{{ stat.failed_operations }}</td>
                        <td class="px-6 py-4 text-sm text-right">
                            <span :class="{
                                'text-green-600': stat.failure_rate < 5,
                                'text-yellow-600': stat.failure_rate >= 5 && stat.failure_rate < 15,
                                'text-red-600': stat.failure_rate >= 15
                            }">{{ stat.failure_rate.toFixed(1) }}%</span>
                        </td>
                        <td class="px-6 py-4 text-xs text-gray-500 truncate max-w-xs" :title="stat.last_error_message">
                            {{ stat.last_error_message || '-' }}
                        </td>
                    </tr>
                </tbody>
            </table>
        </div>

        <!-- Audit Logs Section -->
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700">
            <div class="px-6 py-4 border-b border-gray-200 dark:border-gray-700 flex items-center justify-between">
                <h2 class="text-lg font-semibold text-gray-900 dark:text-white flex items-center">
                    <FileText class="w-5 h-5 mr-2" />
                    Audit Logs
                </h2>
                <button
                    @click="showFilters = !showFilters"
                    class="flex items-center text-sm text-gray-600 hover:text-gray-900"
                >
                    <Filter class="w-4 h-4 mr-1" />
                    Filters
                    <ChevronDown v-if="!showFilters" class="w-4 h-4 ml-1" />
                    <ChevronUp v-else class="w-4 h-4 ml-1" />
                </button>
            </div>

            <!-- Filters -->
            <div v-if="showFilters" class="px-6 py-4 bg-gray-50 dark:bg-gray-900 border-b border-gray-200 dark:border-gray-700">
                <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
                    <div>
                        <label class="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">Status</label>
                        <select
                            v-model="filters.status"
                            @change="fetchAuditLogs"
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-sm"
                        >
                            <option value="">All</option>
                            <option value="success">Success</option>
                            <option value="failed">Failed</option>
                            <option value="pending">Pending</option>
                        </select>
                    </div>
                    <div>
                        <label class="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">Action</label>
                        <select
                            v-model="filters.action"
                            @change="fetchAuditLogs"
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-sm"
                        >
                            <option value="">All</option>
                            <option value="add">Add</option>
                            <option value="remove">Remove</option>
                            <option value="skip">Skip</option>
                        </select>
                    </div>
                    <div>
                        <label class="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">From</label>
                        <input
                            v-model="filters.from_date"
                            type="date"
                            @change="fetchAuditLogs"
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-sm"
                        />
                    </div>
                    <div>
                        <label class="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">To</label>
                        <input
                            v-model="filters.to_date"
                            type="date"
                            @change="fetchAuditLogs"
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-sm"
                        />
                    </div>
                </div>
            </div>

            <!-- Logs Table -->
            <div class="overflow-x-auto">
                <table class="w-full">
                    <thead class="bg-gray-50 dark:bg-gray-900">
                        <tr>
                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Time</th>
                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Policy</th>
                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">User</th>
                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Target</th>
                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Action</th>
                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Status</th>
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-gray-200 dark:divide-gray-700">
                        <tr v-for="log in auditLogs" :key="log.id" class="hover:bg-gray-50 dark:hover:bg-gray-700/50">
                            <td class="px-6 py-4 text-sm text-gray-500">
                                {{ format(new Date(log.created_at), 'MMM d, HH:mm:ss') }}
                            </td>
                            <td class="px-6 py-4 text-sm font-medium text-gray-900 dark:text-white">
                                {{ log.policy_name || 'Unknown' }}
                            </td>
                            <td class="px-6 py-4 text-sm text-gray-600 dark:text-gray-400">
                                @{{ log.username || log.user_id.slice(0, 8) }}...
                            </td>
                            <td class="px-6 py-4 text-sm text-gray-600 dark:text-gray-400">
                                {{ log.target_type }} {{ log.target_id.slice(0, 8) }}...
                            </td>
                            <td class="px-6 py-4 text-sm capitalize">{{ log.action }}</td>
                            <td class="px-6 py-4">
                                <span
                                    :class="{
                                        'px-2 py-1 text-xs rounded-full font-medium': true,
                                        'bg-green-100 text-green-800': log.status === 'success',
                                        'bg-red-100 text-red-800': log.status === 'failed',
                                        'bg-yellow-100 text-yellow-800': log.status === 'pending'
                                    }"
                                >
                                    {{ log.status }}
                                </span>
                            </td>
                        </tr>
                        <tr v-if="auditLogs.length === 0">
                            <td colspan="6" class="px-6 py-8 text-center text-gray-500">
                                <FileText class="w-12 h-12 mx-auto mb-2 text-gray-300" />
                                <p>No audit logs found</p>
                            </td>
                        </tr>
                    </tbody>
                </table>
            </div>
        </div>
    </div>
</template>
