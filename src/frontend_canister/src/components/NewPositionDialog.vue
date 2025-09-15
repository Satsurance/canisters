<template>
  <Transition name="fade">
    <div v-if="isOpen" class="fixed inset-0 z-50 overflow-y-auto" @click="$emit('close')">
      <div class="fixed inset-0 bg-gray-500/70 backdrop-blur-sm transition-opacity"></div>
      <div class="flex min-h-full items-center justify-center p-4">
        <div
            class="relative w-full max-w-xl transform overflow-hidden rounded-xl bg-white p-8 shadow-lg transition-all border border-gray-100"
            @click.stop
        >
          <!-- Dialog Header -->
          <div class="flex items-center justify-between mb-6">
            <h3 class="text-2xl font-bold text-gray-900 flex items-center gap-2">
              <svg class="w-6 h-6 text-yellow-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6"/>
              </svg>
              Create New Position
            </h3>
            <button
                @click="$emit('close')"
                class="rounded-full p-2 hover:bg-gray-100 transition-colors focus:outline-none focus:ring-2 focus:ring-gray-200"
            >
              <svg class="h-5 w-5 text-gray-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
              </svg>
            </button>
          </div>

          <!-- Position Form -->
          <form @submit.prevent="handleCreatePosition" class="space-y-6">
            <!-- Amount Input -->
            <div>
              <label for="amount" class="block mb-2 text-sm font-semibold text-gray-900 flex items-center gap-2">
                BTC Amount to Stake
              </label>
              <div class="relative">
                <input
                    type="number"
                    id="amount"
                    v-model="toStakeAmount"
                    class="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-yellow-500 focus:border-yellow-500 focus:outline-none block w-full p-3 pr-16 transition-colors duration-200"
                    placeholder="0.1"
                    step="0.00000001"
                    min="0"
                    required
                />
                <div class="absolute inset-y-0 right-0 flex items-center pr-4">
                  <span class="text-gray-500 font-medium">BTC</span>
                </div>
              </div>
              <p class="mt-1 text-sm text-gray-500">Minimum stake amount: 0.01 BTC</p>
            </div>

            <!-- Lock Period Selection -->
            <div>
              <label class="block mb-3 text-sm font-semibold text-gray-900 flex items-center gap-2">
                Lock Period
              </label>
              <div class="grid grid-cols-2 gap-3">
                <!-- Instant (0 days) -->
                <div>
                  <input
                      type="radio"
                      id="lock0"
                      name="lockPeriod"
                      :value="0"
                      v-model="selectedLockPeriod"
                      class="peer hidden"
                  />
                  <label
                      for="lock0"
                      class="flex flex-col items-center justify-center p-4 bg-white border-2 border-gray-200 rounded-lg cursor-pointer hover:border-yellow-200 hover:shadow-sm peer-checked:border-yellow-500 peer-checked:bg-yellow-50 transition-all duration-200"
                  >
                    <span class="text-lg font-medium">Instant</span>
                    <span class="text-sm text-gray-500">Test Mode</span>
                  </label>
                </div>

                <!-- 90 days -->
                <div>
                  <input
                      type="radio"
                      id="lock90"
                      name="lockPeriod"
                      :value="90"
                      v-model="selectedLockPeriod"
                      class="peer hidden"
                  />
                  <label
                      for="lock90"
                      class="flex flex-col items-center justify-center p-4 bg-white border-2 border-gray-200 rounded-lg cursor-pointer hover:border-yellow-200 hover:shadow-sm peer-checked:border-yellow-500 peer-checked:bg-yellow-50 transition-all duration-200"
                  >
                    <span class="text-lg font-medium">90 Days</span>
                    <span class="text-sm text-gray-500">Basic</span>
                  </label>
                </div>

                <!-- 180 days -->
                <div>
                  <input
                      type="radio"
                      id="lock180"
                      name="lockPeriod"
                      :value="180"
                      v-model="selectedLockPeriod"
                      class="peer hidden"
                  />
                  <label
                      for="lock180"
                      class="flex flex-col items-center justify-center p-4 bg-white border-2 border-gray-200 rounded-lg cursor-pointer hover:border-yellow-200 hover:shadow-sm peer-checked:border-yellow-500 peer-checked:bg-yellow-50 transition-all duration-200"
                  >
                    <span class="text-lg font-medium">180 Days</span>
                    <span class="text-sm text-gray-500">Enhanced</span>
                  </label>
                </div>

                <!-- 360 days -->
                <div>
                  <input
                      type="radio"
                      id="lock360"
                      name="lockPeriod"
                      :value="360"
                      v-model="selectedLockPeriod"
                      class="peer hidden"
                  />
                  <label
                      for="lock360"
                      class="flex flex-col items-center justify-center p-4 bg-white border-2 border-gray-200 rounded-lg cursor-pointer hover:border-yellow-200 hover:shadow-sm peer-checked:border-yellow-500 peer-checked:bg-yellow-50 transition-all duration-200"
                  >
                    <span class="text-lg font-medium">360 Days</span>
                    <span class="text-sm text-gray-500">Maximum</span>
                  </label>
                </div>
              </div>
            </div>

            <!-- Enhanced Staking Summary -->
            <div class="bg-gradient-to-r from-gray-50 to-yellow-50 rounded-xl p-5 border border-yellow-100">
              <h4 class="text-base font-semibold text-gray-900 mb-4 flex items-center gap-2">
                <svg class="w-5 h-5 text-yellow-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
                </svg>
                Position Summary
              </h4>
              <div class="space-y-3">
                <div class="flex justify-between items-center">
                  <span class="text-sm text-gray-600 flex items-center gap-2">
                    <svg class="w-4 h-4 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
                    </svg>
                    Amount to Stake
                  </span>
                  <span class="font-medium text-gray-900">{{ toStakeAmount || '0' }} BTC</span>
                </div>
                <div class="flex justify-between items-center">
                  <span class="text-sm text-gray-600 flex items-center gap-2">
                    <svg class="w-4 h-4 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"/>
                    </svg>
                    Lock Period
                  </span>
                  <span class="font-medium text-gray-900">{{ selectedLockPeriod || '0' }} Days</span>
                </div>
                <div class="flex justify-between items-center">
                  <span class="text-sm text-gray-600 flex items-center gap-2">
                    <svg class="w-4 h-4 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"/>
                    </svg>
                    Unlock Date
                  </span>
                  <span class="font-medium text-gray-900">{{ selectedLockPeriod ? formatDate(new Date(Date.now() + selectedLockPeriod * 24 * 60 * 60 * 1000)) : 'Instant' }}</span>
                </div>
              </div>
            </div>

            <!-- Submit Button -->
            <div class="pt-4">
              <button
                  type="submit"
                  :disabled="isSubmitting || !isValidAmount"
                  class="w-full py-4 px-4 rounded-lg font-medium shadow-sm transition-all duration-300 flex items-center justify-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed"
                  :class="[
                  isSubmitting || !isValidAmount
                    ? 'bg-gray-100 text-gray-400'
                    : 'bg-yellow-500 text-white hover:bg-yellow-600 hover:shadow'
                ]"
              >
                <svg v-if="isSubmitting" class="w-5 h-5 animate-spin" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                  <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
                {{ isSubmitting ? 'Creating Position...' : 'Create Position' }}
              </button>
            </div>
          </form>
        </div>
      </div>
    </div>
  </Transition>

  <!-- Transaction Status Modal -->
  <TransactionStatus
      :show="!!(firstTxStatus || secondTxStatus)"
      :steps="transactionSteps"
      :tx-hash="currentTxHash"
      :error="transactionError"
      :block-explorer="web3Store.chainId ? SUPPORTED_NETWORKS[web3Store.chainId].blockExplorerUrls[0] : ''"
      @close="resetTransaction"
      @retry="retryTransaction"
  />
</template>

<script setup>
import { ref, computed } from 'vue';
import { ethers } from 'ethers';
import { useWeb3Store } from '../stores/web3Store';
import {getContractAddress, SUPPORTED_NETWORKS} from '../constants/contracts.js';
import erc20ABI from '../assets/abis/erc20.json';
import { formatDate } from '../utils.js';
import TransactionStatus from '../components/TransactionStatus.vue';

const props = defineProps({
  isOpen: {
    type: Boolean,
    required: true
  },
  poolContract: {
    type: Object,
    required: true
  }
});

const emit = defineEmits(['close', 'positionCreated']);

// State
const web3Store = useWeb3Store();
const toStakeAmount = ref(null);
const selectedLockPeriod = ref(90);

// Transaction state
const firstTxStatus = ref("");
const secondTxStatus = ref("");
const transactionType = ref("");
const currentTxHash = ref("");
const transactionError = ref("");
const isSubmitting = ref(false);

// Computed
const isValidAmount = computed(() => {
  return toStakeAmount.value && toStakeAmount.value >= 0.01;
});

const transactionSteps = computed(() => {
  return [
    {
      id: 'approve',
      title: 'Approve BTC',
      description: 'Allow smart contract to use your BTC tokens',
      status: firstTxStatus.value,
      showNumber: true
    },
    {
      id: 'stake',
      title: 'Create Position',
      description: 'Stake your BTC tokens',
      status: secondTxStatus.value,
      showNumber: true
    }
  ];
});

// Methods
const resetTransaction = () => {
  firstTxStatus.value = "";
  secondTxStatus.value = "";
  transactionType.value = "";
  currentTxHash.value = "";
  transactionError.value = "";
  isSubmitting.value = false;
};

const retryTransaction = () => {
  if (transactionType.value === "create_position") {
    handleCreatePosition();
  }
};

const handleStakeProcess = async (amountInWei) => {
  try {
    const signer = web3Store.provider.getSigner();
    const btcContract = new ethers.Contract(
        getContractAddress('BTC_TOKEN', web3Store.chainId),
        erc20ABI,
        signer
    );

    // Check allowance
    const currentAllowance = await btcContract.allowance(
        web3Store.account,
        props.poolContract.address
    );

    // Handle approval if needed
    if (currentAllowance.lt(amountInWei)) {
      transactionType.value = "create_position";
      firstTxStatus.value = "pending";

      try {
        const approveTx = await btcContract.approve(
            props.poolContract.address,
            amountInWei,
            {
              from: web3Store.account,
            }
        );
        currentTxHash.value = approveTx.hash;

        await approveTx.wait();
        firstTxStatus.value = "success";
      } catch (error) {
        firstTxStatus.value = "failed";
        transactionError.value =
            error.code === 4001
                ? "Transaction rejected by user"
                : "Failed to approve tokens";
        throw error;
      }
    }

    // Handle staking
    secondTxStatus.value = "pending";
    const stakeTx = await props.poolContract.joinPool(
        amountInWei,
        selectedLockPeriod.value * 60 * 60 * 24,
        {
          from: web3Store.account,
        }
    );
    currentTxHash.value = stakeTx.hash;

    await stakeTx.wait();
    secondTxStatus.value = "success";

    emit('positionCreated');
    emit('close');

    // Auto-close on success after delay
    setTimeout(resetTransaction, 3000);
  } catch (error) {
    console.error("Position creation error:", error);
    throw error;
  }
};

const handleCreatePosition = async () => {
  if (!isValidAmount.value) {
    return;
  }

  try {
    isSubmitting.value = true;
    const amountInWei = ethers.utils.parseEther(toStakeAmount.value.toString());

    // Check BTC balance first
    const btcContract = new ethers.Contract(
        getContractAddress('BTC_TOKEN', web3Store.chainId),
        erc20ABI,
        web3Store.provider
    );
    const balance = await btcContract.balanceOf(web3Store.account);

    if (balance.lt(amountInWei)) {
      transactionError.value = `Insufficient BTC balance. You have ${ethers.utils.formatEther(
          balance
      )} BTC but trying to stake ${toStakeAmount.value} BTC`;
      return;
    }

    await handleStakeProcess(amountInWei);
  } catch (error) {
    console.error('Failed to create position:', error);

    if (firstTxStatus.value !== "failed") {
      secondTxStatus.value = "failed";
      transactionError.value =
          error.code === 4001
              ? "Transaction rejected by user"
              : error.code === -32603
                  ? "Insufficient balance or internal error"
                  : "Transaction failed. Please try again";
    }
  }
};
</script>
