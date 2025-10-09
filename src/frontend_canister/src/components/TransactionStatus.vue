<!-- src/components/TransactionStatus.vue -->
<template>
  <div v-if="show" class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
    <div class="bg-white rounded-lg p-6 max-w-md w-full mx-4">
      <!-- Steps Progress -->
      <div class="mb-6">
        <div v-for="(step, index) in steps" :key="step.id" class="flex items-center mb-6 last:mb-0">
          <!-- Step Indicator -->
          <div v-if="step.showNumber" :class="[
            'w-8 h-8 rounded-full flex items-center justify-center text-sm',
            getStepStatusClasses(step.status)
          ]">
            <template v-if="step.icon">
              <component :is="step.icon" class="w-4 h-4" />
            </template>
            <template v-else>
              {{ index + 1 }}
            </template>
          </div>

          <!-- Step Content -->
          <div class="ml-3 flex-1">
            <div class="font-medium">{{ step.title }}</div>
            <div class="text-sm text-gray-500">{{ step.description }}</div>
          </div>

          <!-- Step Status Indicator -->
          <div v-if="step.status === 'pending'"
            class="ml-2 animate-spin w-5 h-5 border-2 border-blue-600 border-t-transparent rounded-full" />
          <div v-else-if="step.status === 'success'" class="ml-2 text-green-600">
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
            </svg>
          </div>
        </div>
      </div>

      <!-- Error Display -->
      <div v-if="error" class="mt-4 p-3 bg-red-50 rounded-lg text-red-600 text-sm">
        {{ error }}
      </div>

      <!-- Transaction Hash Display -->
      <div v-if="txHash" class="mt-4 p-3 bg-gray-50 rounded-lg text-sm break-all">
        <div class="text-gray-500 mb-1">Dashboard Link:</div>
        <a :href="blockExplorer + '/transaction/' + txHash" target="_blank" class="text-blue-600 hover:text-blue-700">
          {{ txHash }}
        </a>
      </div>

      <!-- Action Buttons -->
      <div class="mt-6 space-y-3">
        <button v-if="hasFailedStep" @click="$emit('retry')"
          class="btn-primary w-full py-2 rounded-lg focus:outline-none">
          Retry Transaction
        </button>

        <button v-if="isCompleted || hasFailedStep" @click="close"
          class="btn-secondary w-full py-2 rounded-lg focus:outline-none">
          {{ isCompleted ? "Close" : "Cancel" }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed, watchEffect } from 'vue';

const props = defineProps({
  show: Boolean,
  steps: {
    type: Array,
    required: true,
    validator: (steps) =>
      steps.every(step =>
        step.id &&
        step.title &&
        ['pending', 'success', 'failed', 'idle'].includes(step.status)
      )
  },
  txHash: String,
  blockExplorer: String,
  error: String,
});

const emit = defineEmits(['close', 'retry']);

const close = () => {
  emit('close');
};

// Computed properties for status checks
const hasFailedStep = computed(() =>
  props.steps.some(step => step.status === 'failed')
);

const isCompleted = computed(() =>
  props.steps.every(step => step.status === 'success')
);

// Utility function for step status styling
const getStepStatusClasses = (status) => {
  return {
    'pending': 'bg-blue-100 text-blue-600',
    'success': 'bg-green-100 text-green-600',
    'failed': 'bg-red-100 text-red-600',
    'idle': 'bg-gray-100 text-gray-600'
  }[status] || 'bg-gray-100 text-gray-600';
};
</script>