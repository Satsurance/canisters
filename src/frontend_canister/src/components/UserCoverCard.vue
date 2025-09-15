<template>
  <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-6 hover:shadow-md transition-shadow">
    <div class="flex flex-col h-full">
      <!-- Project Name and Status -->
      <div class="flex items-center justify-between mb-4">
        <h3 class="text-lg font-semibold text-gray-900">{{ projectInfo?.name || cover.protocol }}</h3>
        <span
            class="px-2.5 py-0.5 text-xs font-medium rounded-full"
            :class="statusClass"
        >
          {{ isActive ? 'Active' : 'Expired' }}
        </span>
      </div>

      <!-- Logo -->
      <div class="flex justify-center items-center py-4 sm:py-6">
        <img
            :src="projectInfo?.logo"
            :alt="`${projectInfo?.name || cover.protocol} logo`"
            class="w-24 h-24 sm:w-28 md:w-32 sm:h-28 md:h-32 object-contain"
        />
      </div>

      <!-- Cover Info -->
      <div class="mt-auto space-y-3">
        <p class="text-gray-600 mb-4 text-sm">{{ projectInfo?.description }}</p>

        <div class="flex justify-between text-sm">
          <span class="text-gray-500">Cover Amount:</span>
          <span class="font-medium">{{ cover.coverAmount }} BTC</span>
        </div>

        <div class="flex justify-between text-sm">
          <span class="text-gray-500">Start Date:</span>
          <span class="font-medium">{{ formatDate(cover.startDate) }}</span>
        </div>

        <div class="flex justify-between text-sm">
          <span class="text-gray-500">End Date:</span>
          <span class="font-medium">{{ formatDate(cover.endDate) }}</span>
        </div>

        <div v-if="isActive" class="flex justify-between text-sm">
          <span class="text-gray-500">Time Left:</span>
          <span
              :class="{'text-red-500': isExpiringSoon, 'text-green-600': !isExpiringSoon}"
              class="font-medium"
          >
            {{ timeRemaining }}
          </span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue';

const props = defineProps({
  cover: {
    type: Object,
    required: true
  },
  projectInfo: {
    type: Object,
    default: () => ({})
  }
});

const isActive = computed(() => props.cover.endDate > Date.now());

const isExpiringSoon = computed(() => {
  const expireSoon = Date.now() + (5 * 24 * 60 * 60 * 1000);
  return props.cover.endDate <= expireSoon && isActive.value;
});

const statusClass = computed(() => ({
  'bg-green-100 text-green-800': isActive.value,
  'bg-red-100 text-red-800': !isActive.value
}));

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

const formatDate = (timestamp) => {
  return new Date(timestamp).toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric'
  });
};

defineEmits(['renew']);
</script>