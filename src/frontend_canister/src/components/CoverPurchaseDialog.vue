<template>
  <Transition name="fade">
    <div v-if="show" class="fixed inset-0 z-50 overflow-y-auto" @click="$emit('close')">
      <div class="fixed inset-0 bg-gray-500/70 backdrop-blur-sm transition-opacity"></div>
      <div class="flex min-h-full items-center justify-center p-4">
        <div
          class="relative w-full max-w-xl transform overflow-hidden rounded-3xl bg-white p-4 sm:p-6 md:p-8 shadow-xl transition-all"
          @click.stop>
          <!-- Modal Header -->
          <div class="flex items-center justify-between mb-4 sm:mb-6 md:mb-8">
            <div class="flex items-center space-x-4">
              <h3 class="text-xl sm:text-2xl font-semibold text-gray-900">
                {{ project.name }} cover
              </h3>
            </div>
            <button @click="$emit('close')" class="rounded-full p-2 hover:bg-gray-100 transition-colors">
              <svg class="h-5 w-5 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>

          <!-- Cover Form -->
          <form @submit.prevent="handlePurchase" class="space-y-6">
            <!-- Cover Amount -->
            <div>
              <label for="cover-amount" class="block mb-2 text-sm font-medium text-gray-900">
                Cover Amount (BTC)
              </label>
              <div class="space-y-1">
                <input type="number" id="cover-amount" v-model="coverAmount" @input="handleCoverAmountInput"
                  placeholder="0.0001" step="any"
                  class="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-yellow-500 focus:border-yellow-500 focus:outline-none block w-full p-2.5 transition-colors duration-200"
                  :class="{ 'border-red-300': coverAmountError }" :aria-invalid="!!coverAmountError"
                  :aria-describedby="coverAmountError ? 'cover-amount-error' : undefined"
                  :disabled="isSubmitting || !web3Store.isConnected" required />
                <!-- Error message container with fixed height -->
                <div class="h-5 overflow-hidden">
                  <Transition enter-active-class="transition-all duration-200 ease-out"
                    enter-from-class="opacity-0 -translate-y-1" enter-to-class="opacity-100 translate-y-0"
                    leave-active-class="transition-all duration-200 ease-in"
                    leave-from-class="opacity-100 translate-y-0" leave-to-class="opacity-0 -translate-y-1">
                    <p v-if="coverAmountError" id="cover-amount-error" class="text-sm text-red-600">
                      {{ coverAmountError }}
                    </p>
                  </Transition>
                </div>
                <p class="text-sm text-gray-500">
                  Available coverage: {{ project.maxCover }} BTC
                </p>
              </div>
            </div>

            <!-- Cover Duration Controls -->
            <div class="space-y-4">
              <label for="cover-duration" class="block text-sm font-medium text-gray-900">
                Cover Duration
              </label>

              <!-- Duration Input -->
              <div class="space-y-1">
                <div
                  class="flex items-center bg-gray-50 border border-gray-300 rounded-lg p-2 focus-within:ring-2 focus-within:ring-yellow-500 focus-within:border-yellow-500 transition-colors duration-200"
                  :class="{ 'border-red-300': durationError }">
                  <input type="number" id="cover-duration-input" v-model="duration" @input="handleDurationInput"
                    step="1" class="w-20 bg-transparent border-none focus:outline-none text-gray-900 text-sm p-1"
                    :aria-invalid="!!durationError" :aria-describedby="durationError ? 'duration-error' : undefined"
                    :disabled="isSubmitting || !web3Store.isConnected" />
                  <span class="text-sm text-gray-600 ml-1">days</span>
                </div>

                <!-- Error message container with fixed height -->
                <div class="h-5 overflow-hidden">
                  <Transition enter-active-class="transition-all duration-200 ease-out"
                    enter-from-class="opacity-0 -translate-y-1" enter-to-class="opacity-100 translate-y-0"
                    leave-active-class="transition-all duration-200 ease-in"
                    leave-from-class="opacity-100 translate-y-0" leave-to-class="opacity-0 -translate-y-1">
                    <p v-if="durationError" id="duration-error" class="text-sm text-red-600">
                      {{ durationError }}
                    </p>
                  </Transition>
                </div>
              </div>

              <!-- Duration Slider with Marks -->
              <div class="space-y-2">
                <input type="range" id="cover-duration-slider" v-model="duration" @input="handleDurationInput" :min="31"
                  :max="project.maxCoverageDuration / 86400" step="1"
                  class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-yellow-500"
                  :disabled="isSubmitting || !web3Store.isConnected" />
                <div class="relative h-4 text-xs text-gray-500">
                  <span class="absolute left-0 -translate-x-1/2 text-[9px] sm:text-xs" style="left: 2%">31d</span>
                  <span class="absolute -translate-x-1/2 hidden sm:inline text-xs" style="left: 19.5%">{{ (Math.floor(31
                    + ((project.maxCoverageDuration / 86400) - 31) * 38 / 200)) }}d</span>
                  <span class="absolute -translate-x-1/2 text-[9px] sm:text-xs" style="left: 48%">{{ (Math.floor((31 +
                    project.maxCoverageDuration / 86400) / 2)) }}d</span>
                  <span class="absolute -translate-x-1/2 hidden sm:inline text-xs" style="left: 72.72%">{{
                    (Math.floor(31
                      + (((project.maxCoverageDuration / 86400) - 31) * 8) / 11)) }}d</span>
                  <span class="absolute -translate-x-1/2 text-[9px] sm:text-xs" style="left: 97%">{{
                    (Math.floor(project.maxCoverageDuration / 86400)) }}d</span>
                </div>
              </div>
            </div>

            <!-- Premium Calculation -->
            <div class="bg-gray-50 rounded-lg p-4 space-y-2">
              <div class="flex justify-between text-sm">
                <span class="text-gray-600">Premium Rate:</span>
                <span class="font-medium">{{ project.rate }}% per year</span>
              </div>
              <div class="flex justify-between text-sm">
                <span class="text-gray-600">Cover Duration:</span>
                <span class="font-medium">{{ duration }} days</span>
              </div>
              <div class="flex justify-between text-sm font-semibold pt-2 border-t">
                <span class="text-gray-900">Total Premium:</span>
                <span class="text-yellow-600">
                  {{ formatPremium }}
                </span>
              </div>
            </div>

            <!-- Submit Button -->
            <div class="flex justify-end pt-4">
              <button type="submit" :disabled="isSubmitting || hasErrors || !web3Store.isConnected"
                class="w-full py-3 rounded-lg transition-colors" :class="[
                  web3Store.isConnected
                    ? 'btn-primary disabled:bg-gray-300 disabled:border-gray-300 disabled:cursor-not-allowed'
                    : 'bg-gray-100 border-gray-100 hover:border-gray-100 cursor-not-allowed text-gray-500'
                ]">
                {{
                  !web3Store.isConnected
                    ? 'Wallet is not connected'
                    : isSubmitting
                      ? 'Processing...'
                      : 'Purchase Cover'
                }}
              </button>
            </div>
          </form>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup>
import { ref, computed } from 'vue';
import { useWeb3Store } from '../stores/web3Store';

const props = defineProps({
  project: {
    type: Object,
    required: true
  },
  show: {
    type: Boolean,
    required: true
  },
  isSubmitting: {
    type: Boolean,
    required: true
  }
});

const emit = defineEmits(['close', 'purchase']);
const web3Store = useWeb3Store();

const coverAmount = ref('');
const duration = ref(31);
const coverAmountError = ref('');
const durationError = ref('');

const hasErrors = computed(() => {
  return !!coverAmountError.value || !!durationError.value;
});

const calculatePremium = computed(() => {
  if (!coverAmount.value || isNaN(coverAmount.value)) return 0;

  // Convert all numbers to strings and use high precision arithmetic
  const amount = Number(coverAmount.value);
  const rate = Number(props.project.rate);
  const days = Number(duration.value);

  // Calculate with maximum precision: (amount * rate * days) / (100 * 365)
  const yearlyPremium = (amount * rate) / 100;
  const durationInYears = days / 365;
  const premium = yearlyPremium * durationInYears;

  return premium;
});

const formatPremium = computed(() => {
  const premium = calculatePremium.value;

  // Handle different ranges of numbers with appropriate precision
  if (premium === 0) return '0 BTC';
  if (premium < 0.00000001) return premium.toExponential(8) + ' BTC';
  if (premium < 0.0001) return premium.toFixed(8) + ' BTC';
  if (premium < 0.01) return premium.toFixed(6) + ' BTC';
  return premium.toFixed(4) + ' BTC';
});

const handleCoverAmountInput = (event) => {
  const value = event.target.value;

  // Allow empty input for better UX
  if (value === '') {
    coverAmount.value = '';
    coverAmountError.value = '';
    return;
  }

  // Parse the input value
  const numValue = Number(value);

  // Validate the input
  if (isNaN(numValue)) {
    coverAmountError.value = 'Please enter a valid number';
    return;
  }

  // Round to 8 decimal places for BTC
  coverAmount.value = parseFloat(numValue.toFixed(8));

  // Validate against max cover
  if (numValue > props.project.maxCover) {
    coverAmountError.value = `Cover amount cannot exceed ${props.project.maxCover} BTC`;
  } else {
    coverAmountError.value = '';
  }
};

const handleDurationInput = () => {
  const numValue = Number(duration.value);
  const maxDurationDays = Math.floor(props.project.maxCoverageDuration / 86400);

  // Clear error first
  durationError.value = '';

  // Validate against max duration
  if (numValue > maxDurationDays) {
    durationError.value = `Duration cannot exceed ${maxDurationDays} days`;
  }
};

const validateCoverAmount = (value) => {
  const numValue = Number(value);
  if (isNaN(numValue)) {
    coverAmountError.value = 'Please enter a valid number';
    return false;
  }
  if (numValue < props.project.minCover) {
    coverAmountError.value = `Cover amount must be at least ${props.project.minCover} BTC`;
    return false;
  }
  if (numValue > props.project.maxCover) {
    coverAmountError.value = `Cover amount cannot exceed ${props.project.maxCover} BTC`;
    return false;
  }
  coverAmountError.value = '';
  return true;
};

const validateDuration = (value) => {
  const numValue = Number(value);
  const maxDurationDays = Math.floor(props.project.maxCoverageDuration / 86400);
  if (numValue < 31) {
    durationError.value = 'Duration must be at least 31 day';
    return false;
  } else if (numValue > maxDurationDays) {
    durationError.value = `Duration cannot exceed ${maxDurationDays} days`;
    return false;
  } else if (!Number.isInteger(numValue)) {
    durationError.value = 'Duration must be a whole number';
    return false;
  }
  durationError.value = '';
  return true;
};

const handlePurchase = async () => {
  // Clear previous errors
  coverAmountError.value = '';
  durationError.value = '';

  // Run validations
  const isValidAmount = validateCoverAmount(coverAmount.value);
  const isValidDuration = validateDuration(duration.value);

  // Only emit if all validations pass
  if (isValidAmount && isValidDuration) {
    emit('purchase', {
      coverAmount: coverAmount.value,
      duration: duration.value,
      premium: calculatePremium.value
    });
  }
};
</script>