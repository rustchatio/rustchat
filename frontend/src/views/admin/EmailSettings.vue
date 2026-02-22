<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { useAdminStore } from '../../stores/admin';
import { Mail, Send, Save, AlertCircle, CheckCircle } from 'lucide-vue-next';
import api from '../../api/client';

const adminStore = useAdminStore();

const form = ref({
    smtp_host: '',
    smtp_port: 587,
    smtp_username: '',
    smtp_password_encrypted: '',
    smtp_security: 'starttls',
    smtp_skip_cert_verify: false,
    from_address: '',
    from_name: 'RustChat',
    reply_to: '',
});

const testEmail = ref('');
const saving = ref(false);
const saveSuccess = ref(false);
const saveError = ref('');

const testing = ref(false);
const testSuccess = ref(false);
const testError = ref('');

onMounted(async () => {
    await adminStore.fetchConfig();
    if (adminStore.config?.email) {
        form.value = normalizeEmailConfig(adminStore.config.email);
    }
});

watch(() => adminStore.config?.email, (email) => {
    if (email) {
        form.value = normalizeEmailConfig(email);
    }
});

function normalizeEmailConfig(email: Record<string, any>) {
    const smtpSecurity = typeof email.smtp_security === 'string'
        ? email.smtp_security
        : email.smtp_tls === false
            ? 'none'
            : 'starttls';

    return {
        ...form.value,
        ...email,
        smtp_security: smtpSecurity,
        smtp_skip_cert_verify: Boolean(email.smtp_skip_cert_verify),
        reply_to: email.reply_to || '',
    };
}

const saveSettings = async () => {
    saving.value = true;
    saveError.value = '';
    saveSuccess.value = false;
    
    try {
        await adminStore.updateConfig('email', form.value);
        saveSuccess.value = true;
        setTimeout(() => saveSuccess.value = false, 3000);
    } catch (e: any) {
        saveError.value = e.response?.data?.message || 'Failed to save settings';
    } finally {
        saving.value = false;
    }
};

const sendTestEmail = async () => {
    if (!testEmail.value) return;
    testing.value = true;
    testError.value = '';
    testSuccess.value = false;

    try {
        const { data } = await api.post('/admin/email/test', { to: testEmail.value });
        if (data?.status && data.status !== 'success') {
            throw new Error(data.error || data.message || 'SMTP test failed');
        }
        testSuccess.value = true;
        setTimeout(() => testSuccess.value = false, 5000);
    } catch (e: any) {
        testError.value = e.response?.data?.error?.message || e.response?.data?.error || e.message || 'Failed to send test email';
    } finally {
        testing.value = false;
    }
};
</script>

<template>
    <div class="space-y-6">
        <div class="flex items-center justify-between">
            <div>
                <h1 class="text-2xl font-bold text-gray-900 dark:text-white">Email & SMTP</h1>
                <p class="text-gray-500 dark:text-gray-400 mt-1">Configure email delivery settings</p>
            </div>
            <div class="flex items-center gap-3">
                <span v-if="saveSuccess" class="flex items-center text-green-600 text-sm">
                    <CheckCircle class="w-4 h-4 mr-1" /> Saved
                </span>
                <button 
                    @click="saveSettings"
                    :disabled="saving"
                    class="flex items-center px-4 py-2 bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50 text-white rounded-lg font-medium transition-colors"
                >
                    <Save class="w-5 h-5 mr-2" />
                    {{ saving ? 'Saving...' : 'Save Changes' }}
                </button>
            </div>
        </div>

        <!-- Error Alert -->
        <div v-if="saveError" class="flex items-center gap-2 p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg text-red-700 dark:text-red-400">
            <AlertCircle class="w-5 h-5 shrink-0" />
            {{ saveError }}
        </div>

        <div class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-gray-200 dark:border-slate-700 p-6">
            <div class="flex items-center mb-6">
                <Mail class="w-5 h-5 text-gray-400 mr-2" />
                <h2 class="text-lg font-semibold text-gray-900 dark:text-white">SMTP Configuration</h2>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div>
                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">SMTP Host</label>
                    <input 
                        v-model="form.smtp_host"
                        type="text"
                        class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-slate-900 text-gray-900 dark:text-white"
                        placeholder="smtp.example.com"
                    />
                </div>
                <div>
                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">SMTP Port</label>
                    <input 
                        v-model.number="form.smtp_port"
                        type="number"
                        class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-slate-900 text-gray-900 dark:text-white"
                    />
                </div>
                <div>
                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Username</label>
                    <input 
                        v-model="form.smtp_username"
                        type="text"
                        class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-slate-900 text-gray-900 dark:text-white"
                    />
                </div>
                <div>
                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Password</label>
                    <input 
                        v-model="form.smtp_password_encrypted"
                        type="password"
                        class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-slate-900 text-gray-900 dark:text-white"
                    />
                </div>
                <div>
                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">From Address</label>
                    <input 
                        v-model="form.from_address"
                        type="email"
                        class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-slate-900 text-gray-900 dark:text-white"
                        placeholder="noreply@example.com"
                    />
                </div>
                <div>
                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">From Name</label>
                    <input 
                        v-model="form.from_name"
                        type="text"
                        class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-slate-900 text-gray-900 dark:text-white"
                    />
                </div>
            </div>

            <div class="mt-4 grid grid-cols-1 md:grid-cols-2 gap-4">
                <div>
                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">SMTP Security</label>
                    <select
                        v-model="form.smtp_security"
                        class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-slate-900 text-gray-900 dark:text-white"
                    >
                        <option value="starttls">STARTTLS (587)</option>
                        <option value="tls">Implicit TLS (465)</option>
                        <option value="none">Plain (no TLS)</option>
                    </select>
                </div>
                <div>
                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Reply-To (optional)</label>
                    <input
                        v-model="form.reply_to"
                        type="email"
                        class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-slate-900 text-gray-900 dark:text-white"
                        placeholder="support@example.com"
                    />
                </div>
            </div>

            <div class="mt-4">
                <label class="flex items-start">
                    <input type="checkbox" v-model="form.smtp_skip_cert_verify" class="w-4 h-4 text-indigo-600 rounded mr-3 mt-0.5" />
                    <span class="text-gray-700 dark:text-gray-300 text-sm">
                        Skip certificate verification (only for testing/self-signed certs)
                    </span>
                </label>
            </div>
        </div>

        <!-- Test Email -->
        <div class="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-gray-200 dark:border-slate-700 p-6">
            <h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">Send Test Email</h3>
            
            <div v-if="testSuccess" class="mb-4 p-3 bg-green-50 dark:bg-green-900/20 text-green-700 dark:text-green-400 rounded-lg text-sm">
                Test email sent successfully!
            </div>
            
            <div v-if="testError" class="mb-4 p-3 bg-red-50 dark:bg-red-900/20 text-red-700 dark:text-red-400 rounded-lg text-sm">
                {{ testError }}
            </div>

            <div class="flex items-center space-x-4">
                <input 
                    v-model="testEmail"
                    type="email"
                    class="flex-1 px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-slate-900 text-gray-900 dark:text-white"
                    placeholder="test@example.com"
                />
                <button 
                    @click="sendTestEmail"
                    :disabled="testing || !testEmail"
                    class="flex items-center px-4 py-2 bg-green-600 hover:bg-green-700 disabled:opacity-50 text-white rounded-lg font-medium transition-colors"
                >
                    <Send class="w-4 h-4 mr-2" />
                    {{ testing ? 'Sending...' : 'Send Test' }}
                </button>
            </div>
        </div>
    </div>
</template>
