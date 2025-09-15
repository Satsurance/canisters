<template>
  <Transition name="fade">
    <div
        v-if="isOpen"
        class="fixed inset-0 z-50 overflow-y-auto"
        @click="onBackdropClick"
    >
      <!-- Backdrop -->
      <div class="fixed inset-0 bg-gray-500/70 backdrop-blur-sm transition-opacity"></div>

      <!-- Dialog -->
      <div class="flex min-h-full items-center justify-center p-4">
        <div
            class="relative w-full max-w-xl transform overflow-hidden rounded-3xl bg-white p-8 shadow-xl transition-all"
            @click.stop
        >
          <!-- Header -->
          <div class="flex items-center justify-between mb-2.5 md:mb-10">
            <div class="flex items-center gap-3">
              <h3 class="text-2xl font-semibold text-gray-900">
                {{ projectInfo?.name || cover.protocol }} Coverage
              </h3>
              <div
                  class="px-4 py-1.5 text-sm rounded-full font-medium"
                  :class="statusClass"
              >
                {{ statusText }}
              </div>
            </div>
            <button
                @click="onClose"
                class="rounded-full p-2 hover:bg-gray-100 transition-colors"
            >
              <svg
                  class="h-5 w-5 text-gray-400"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
              >
                <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M6 18L18 6M6 6l12 12"
                />
              </svg>
            </button>
          </div>

          <!-- Project Info -->
          <div class="flex flex-col items-center mb-8">
            <img
                :src="projectInfo?.logo"
                :alt="`${projectInfo?.name || cover.protocol} logo`"
                class="w-24 h-24 object-contain mb-4"
            />
            <p class="text-gray-600 text-center">{{ projectInfo?.description }}</p>
          </div>

          <!-- Cover Details -->
          <div class="space-y-6">
            <!-- Cover Amount -->
            <div>
              <label class="block text-base text-gray-500 mb-2">Cover Amount</label>
              <div class="p-3 bg-gray-50 rounded-lg">
                <span class="text-2xl font-semibold text-gray-900">
                  {{ formatAmount(cover.coverAmount) }} BTC
                </span>
              </div>
            </div>

            <!-- Dates -->
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div>
                <label class="block text-base text-gray-500 mb-2">Start Date</label>
                <div class="p-3 bg-gray-50 rounded-lg">
                  <span class="text-base text-gray-900">{{ formatDate(cover.startDate) }}</span>
                </div>
              </div>
              <div>
                <label class="block text-base text-gray-500 mb-2">End Date</label>
                <div class="p-3 bg-gray-50 rounded-lg">
                  <span class="text-base text-gray-900">{{ formatDate(cover.endDate) }}</span>
                </div>
              </div>
            </div>

            <!-- Time Remaining -->
            <div v-if="isActive">
              <label class="block text-base text-gray-500 mb-2">Time Left</label>
              <div class="p-3 bg-gray-50 rounded-lg">
                <span
                    class="text-base font-medium"
                    :class="{'text-red-500': isExpiringSoon, 'text-green-600': !isExpiringSoon}"
                >
                  {{ timeRemaining }}
                </span>
              </div>
            </div>
          </div>

        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup>
import { computed } from 'vue';

const props = defineProps({
  cover: {
    type: Object,
    required: true,
    validator: (value) => {
      return [
        'user',
        'protocol',
        'startDate',
        'endDate',
        'coverAmount'
      ].every(key => key in value);
    }
  },
  projectInfo: {
    type: Object,
    default: () => ({})
  },
  isOpen: {
    type: Boolean,
    default: false
  }
});

const emit = defineEmits(['close', 'renew']);

// Computed properties
const isActive = computed(() => {
  console.log(props.cover);
  return props.cover.endDate > Date.now();
});

const isExpiringSoon = computed(() => {
  const expireSoon = Date.now() + (5 * 24 * 60 * 60 * 1000);
  return props.cover.endDate <= expireSoon && isActive.value;
});

const statusText = computed(() => {
  if (!isActive.value) return 'Expired';
  return isExpiringSoon.value ? 'Expiring Soon' : 'Active';
});

const statusClass = computed(() => {
  if (!isActive.value) return 'bg-red-100 text-red-800';
  return isExpiringSoon.value
      ? 'bg-yellow-100 text-yellow-800'
      : 'bg-green-100 text-green-800';
});

const timeRemaining = computed(() => {
  if (!isActive.value) return '';

  const diff = props.cover.endDate - Date.now();
  const days = Math.floor(diff / (1000 * 60 * 60 * 24));
  const hours = Math.floor((diff % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60));

  if (days > 1) {
    return `${days} days ${hours} hours`;
  }
  return `${hours} hours`;
});

// Helper functions
const formatDate = (timestamp) => {
  return new Date(timestamp).toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  });
};

const formatAmount = (amount) => {
   return amount;
  // return new Intl.NumberFormat('en-US', {
  //   minimumFractionDigits: 2,
  //   maximumFractionDigits: 2
  // }).format(ethers.utils.formatEther(amount.toString()));
};

const copyToClipboard = async (text) => {
  if (!text) return;
  try {
    await navigator.clipboard.writeText(text);
  } catch (err) {
    console.error('Failed to copy text: ', err);
  }
};

// Event handlers
const onClose = () => {
  emit('close');
};

const onBackdropClick = () => {
  onClose();
};
</script>