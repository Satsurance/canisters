<template>
  <Transition name="fade">
    <div v-if="isOpen" class="fixed inset-0 z-50 overflow-y-auto" @click="$emit('close')">
      <div class="fixed inset-0 bg-gray-500/70 backdrop-blur-sm transition-opacity"></div>
      <div class="flex min-h-full items-center justify-center p-4">
        <div
          class="relative w-full max-w-xl transform overflow-hidden rounded-xl bg-white p-8 shadow-lg transition-all border border-gray-100"
          @click.stop>
          <!-- Dialog Header -->
          <div class="flex items-center justify-between mb-6">
            <h3 class="text-2xl font-bold text-gray-900 flex items-center gap-2">
              <svg class="w-6 h-6 text-yellow-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
              </svg>
              Create New Position
            </h3>
            <button @click="$emit('close')"
              class="rounded-full p-2 hover:bg-gray-100 transition-colors focus:outline-none focus:ring-2 focus:ring-gray-200">
              <svg class="h-5 w-5 text-gray-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
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
                <input type="number" id="amount" v-model="toStakeAmount" :disabled="isStakingDisabled"
                  class="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-yellow-500 focus:border-yellow-500 focus:outline-none block w-full p-3 pr-16 transition-colors duration-200 disabled:opacity-50 disabled:cursor-not-allowed"
                  placeholder="0.1" step="0.00000001" min="0" required />
                <div class="absolute inset-y-0 right-0 flex items-center pr-4">
                  <span class="text-gray-500 font-medium">BTC</span>
                </div>
              </div>
              <div class="mt-1 space-y-1">
                <p class="text-sm text-gray-500">Minimum stake amount: 0.01 BTC</p>
                <p v-if="maxStakeableAmount && Number(maxStakeableAmount) > 0" class="text-sm text-gray-500">
                  Maximum stake amount: {{ maxStakeableAmount }} BTC
                </p>
                <p v-if="validationMessage" class="text-sm text-red-600">
                  {{ validationMessage }}
                </p>
              </div>
            </div>

            <!-- Episode Selection -->
            <div>
              <label class="block mb-3 text-sm font-semibold text-gray-900 flex items-center gap-2">
                Select Lock Duration
              </label>
              <div class="space-y-2 max-h-48 overflow-y-auto">
                <div v-for="episode in availableEpisodes" :key="episode.number" class="flex items-center">
                  <input type="radio" :id="`episode-${episode.number}`" name="episodeToStake" :value="episode.number"
                    v-model="selectedEpisode" class="peer hidden" />
                  <label :for="`episode-${episode.number}`"
                    class="flex-1 flex items-center justify-between p-4 bg-white border-2 border-gray-200 rounded-lg cursor-pointer hover:border-yellow-200 hover:shadow-sm peer-checked:border-yellow-500 peer-checked:bg-yellow-50 transition-all duration-200">
                    <div class="flex flex-col">
                      <span class="text-lg font-medium text-gray-900">{{ episode.durationDays }} days</span>
                      <span class="text-sm text-gray-500">Lock duration</span>
                    </div>
                    <div class="text-right">
                      <span class="text-sm font-medium text-gray-900">{{ episode.unlockDate }}</span>
                      <div class="text-xs text-gray-500">Unlock Date</div>
                    </div>
                  </label>
                </div>
              </div>
            </div>

            <!-- Enhanced Staking Summary -->
            <div class="bg-gradient-to-r from-gray-50 to-yellow-50 rounded-xl p-5 border border-yellow-100">
              <h4 class="text-base font-semibold text-gray-900 mb-4 flex items-center gap-2">
                <svg class="w-5 h-5 text-yellow-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                    d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                Position Summary
              </h4>
              <div class="space-y-3">
                <div class="flex justify-between items-center">
                  <span class="text-sm text-gray-600 flex items-center gap-2">
                    <svg class="w-4 h-4 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                        d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                    </svg>
                    Amount to Stake
                  </span>
                  <span class="font-medium text-gray-900">{{ toStakeAmount || '0' }} BTC</span>
                </div>
                <div class="flex justify-between items-center">
                  <span class="text-sm text-gray-600 flex items-center gap-2">
                    <svg class="w-4 h-4 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                        d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
                    </svg>
                    Lock Duration
                  </span>
                  <span class="font-medium text-gray-900">{{ selectedEpisodeDuration || 'Not Selected' }}</span>
                </div>
                <div class="flex justify-between items-center">
                  <span class="text-sm text-gray-600 flex items-center gap-2">
                    <svg class="w-4 h-4 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                        d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 002 2z" />
                    </svg>
                    Unlock Date
                  </span>
                  <span class="font-medium text-gray-900">{{ selectedEpisodeUnlockDate || 'Not Selected' }}</span>
                </div>
              </div>
            </div>

            <!-- Submit Button -->
            <div class="pt-4">
              <button type="submit" :disabled="isSubmitting || !isValidAmount || !selectedEpisode || isStakingDisabled"
                class="w-full py-4 px-4 rounded-lg font-medium shadow-sm transition-all duration-300 flex items-center justify-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed"
                :class="[
                  isSubmitting || !isValidAmount || !selectedEpisode || isStakingDisabled
                    ? 'bg-gray-100 text-gray-400'
                    : 'bg-yellow-500 text-white hover:bg-yellow-600 hover:shadow'
                ]">
                <svg v-if="isSubmitting" class="w-5 h-5 animate-spin" xmlns="http://www.w3.org/2000/svg" fill="none"
                  viewBox="0 0 24 24">
                  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                  <path class="opacity-75" fill="currentColor"
                    d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z">
                  </path>
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
  <TransactionStatus :show="!!(firstTxStatus || secondTxStatus || transactionError)" :steps="transactionSteps"
    :tx-hash="currentTxHash" :error="transactionError" :block-explorer="ICP_MAINNET_BLOCK_EXPLORER"
    @close="resetTransaction" @retry="retryTransaction" />
</template>

<script setup>
import { ref, computed, onMounted } from 'vue';
import { useWeb3Store } from '../stores/web3Store';
import { ICP_CONFIG, getCurrentNetwork, getCanisterIds, ICP_MAINNET_BLOCK_EXPLORER } from '../constants/icp.js';
import { createBackendActorWithPlug, createLedgerActorWithPlug } from '../utils/icpAgent.js';
import { formatDate } from '../utils.js';
import TransactionStatus from '../components/TransactionStatus.vue';
import { CodeSquare } from 'lucide-vue-next';
import { Principal } from '@dfinity/principal';
import { handlePlugError } from '../utils/errorHandler.js';

const props = defineProps({
  isOpen: {
    type: Boolean,
    required: true
  },

  poolContract: {
    type: Object,
    default: null,
  },
  maxStakeableAmount: {
    type: [String, Number],
    default: null
  }
});

const emit = defineEmits(['close', 'positionCreated']);

// Constants from the contract
const EPISODE_DURATION = Math.floor((91 * 24 * 60 * 60) / 3); // 91 days / 3 in seconds
const MAX_ACTIVE_EPISODES = 24;

// State
const web3Store = useWeb3Store();
const toStakeAmount = ref(null);
const selectedEpisode = ref(null);
const currentEpisode = ref(0);
const availableEpisodes = ref([]);

// Transaction state
const firstTxStatus = ref("");
const secondTxStatus = ref("");
const transactionType = ref("");
const currentTxHash = ref("");
const transactionError = ref("");
const isSubmitting = ref(false);

// Computed
const isValidAmount = computed(() => {
  if (!toStakeAmount.value || toStakeAmount.value < 0.01) {
    return false;
  }

  // Check against max stakeable amount if provided
  if (props.maxStakeableAmount && Number(props.maxStakeableAmount) > 0) {
    return toStakeAmount.value <= Number(props.maxStakeableAmount);
  }

  return true;
});

const selectedEpisodeUnlockDate = computed(() => {
  if (!selectedEpisode.value) return null;
  const episode = availableEpisodes.value.find(ep => ep.number === selectedEpisode.value);
  return episode ? episode.unlockDate : null;
});

const selectedEpisodeDuration = computed(() => {
  if (!selectedEpisode.value) return null;
  const episode = availableEpisodes.value.find(ep => ep.number === selectedEpisode.value);
  return episode ? `${episode.durationDays} days` : null;
});

const validationMessage = computed(() => {
  // Check if staking is disabled due to 0 max stakeable amount
  if (props.maxStakeableAmount && Number(props.maxStakeableAmount) <= 0) {
    return 'Pool has reached maximum capacity. No new stakes allowed.';
  }

  if (!toStakeAmount.value) return '';

  if (toStakeAmount.value < 0.01) {
    return 'Amount must be at least 0.01 BTC';
  }

  if (props.maxStakeableAmount && Number(props.maxStakeableAmount) > 0 && toStakeAmount.value > Number(props.maxStakeableAmount)) {
    return `Amount exceeds maximum stakeable limit of ${props.maxStakeableAmount} BTC`;
  }

  return '';
});

const isStakingDisabled = computed(() => {
  return props.maxStakeableAmount && Number(props.maxStakeableAmount) <= 0;
});

const transactionSteps = computed(() => {
  return [
    {
      id: 'transfer',
      title: 'Transfer BTC',
      description: 'Send BTC tokens to deposit subaccount',
      status: firstTxStatus.value,
      showNumber: true
    },
    {
      id: 'stake',
      title: 'Create Position',
      description: 'Finalize deposit on canister for selected episode',
      status: secondTxStatus.value,
      showNumber: true
    }
  ];
});

// Episode calculation functions
const getCurrentEpisode = () => {
  return Math.floor(Date.now() / 1000 / EPISODE_DURATION);
};

const getEpisodeFinishTime = (episodeId) => {
  return (episodeId + 1) * EPISODE_DURATION;
};

const calculateAvailableEpisodes = () => {
  const current = getCurrentEpisode();
  const episodes = [];

  for (let i = current; i < current + MAX_ACTIVE_EPISODES; i++) {
    // Check if episode satisfies the modulo 3 == 2 rule
    if (i % 3 === 2) {
      const finishTime = getEpisodeFinishTime(i);
      const unlockDate = new Date(finishTime * 1000);
      const durationDays = Math.ceil((finishTime * 1000 - Date.now()) / (1000 * 60 * 60 * 24));

      episodes.push({
        number: i,
        unlockDate: formatDate(unlockDate),
        durationDays: durationDays
      });
    }
  }

  return episodes;
};

// Initialize episodes on component mount
onMounted(() => {
  currentEpisode.value = getCurrentEpisode();
  availableEpisodes.value = calculateAvailableEpisodes();

  // Select the first available episode by default
  if (availableEpisodes.value.length > 0) {
    selectedEpisode.value = availableEpisodes.value[0].number;
  }
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

const handleStakeProcess = async (amountNat) => {
  try {
    transactionType.value = "create_position";

    // Ensure Plug connection and agent
    if (!window.ic || !window.ic.plug || !web3Store.isConnected) {
      throw new Error('Plug wallet not connected');
    }

    const currentNetwork = getCurrentNetwork();
    const { backend, ledger } = getCanisterIds(currentNetwork);

    // Ensure Plug agent is initialized for this host and canisters
    try {
      await window.ic.plug.createAgent({
        whitelist: [backend, ledger],
        host: ICP_CONFIG[currentNetwork].host,
      });

      // Small delay to ensure agent is ready
      await new Promise(resolve => setTimeout(resolve, 100));
    } catch (error) {
      handlePlugError(error);
    }

    const backendActor = await createBackendActorWithPlug(backend);
    const ledgerActor = await createLedgerActorWithPlug(ledger);

    // Compute deposit subaccount for user and episode
    const principal = await window.ic.plug.agent.getPrincipal();
    console.log('principal', principal);
    const subaccount = await backendActor.get_deposit_subaccount(principal, BigInt(selectedEpisode.value));
    console.log('subaccount', subaccount);
    // Determine decimals to scale input
    const decimals = Number(await ledgerActor.icrc1_decimals());

    // Step 1: transfer tokens to deposit subaccount using Plug's requestTransfer
    firstTxStatus.value = "pending";

    const fee = await ledgerActor.icrc1_fee();

    if (amountNat <= fee) {
      firstTxStatus.value = "failed";
      transactionError.value = `Amount must be greater than fee`;
      throw new Error('Amount too small');
    }

    const transferArg = {
      from_subaccount: [],
      to: { owner: Principal.fromText(backend), subaccount: [subaccount] },
      amount: amountNat,
      fee: [fee],
      memo: [],
      created_at_time: [],
    };


    // Use ledger actor with Plug signing
    console.log('transferArg', transferArg);
    try {
      const transferResult = await ledgerActor.icrc1_transfer(transferArg);
      if ('Err' in transferResult) {
        firstTxStatus.value = "failed";
        transactionError.value = `Transfer failed: ${JSON.stringify(transferResult.Err)}`;
        throw new Error('Transfer failed');
      }
    } catch (error) {
      handlePlugError(error);
    }

    firstTxStatus.value = "success";

    // Step 2: call deposit on backend
    secondTxStatus.value = "pending";

    try {
      const depRes = await backendActor.deposit(principal, BigInt(selectedEpisode.value));

      if ('Err' in depRes) {
        secondTxStatus.value = "failed";
        transactionError.value = `Deposit failed: ${JSON.stringify(depRes.Err)}`;
        throw new Error('Deposit failed');
      }
    }
    catch (error) {
      handlePlugError(error);
    }


    secondTxStatus.value = "success";

    emit('positionCreated');
    emit('close');
    setTimeout(resetTransaction, 3000);
  } catch (error) {
    console.log('error', error);
    throw error;
  }
};

const handleCreatePosition = async () => {
  if (!isValidAmount.value || !selectedEpisode.value) {
    return;
  }

  try {
    isSubmitting.value = true;

    if (!window.ic || !window.ic.plug || !web3Store.isConnected) {
      transactionError.value = 'Plug wallet not connected';
      return;
    }

    const currentNetwork = getCurrentNetwork();
    const { backend, ledger } = getCanisterIds(currentNetwork);
    console.log('backend', backend);
    console.log('ledger', ledger);

    try {
      await window.ic.plug.createAgent({
        whitelist: [backend, ledger],
        host: ICP_CONFIG[currentNetwork].host,
      });
    } catch (error) {
      handlePlugError(error);
    }

    const ledgerActor = await createLedgerActorWithPlug(ledger);
    console.log('ledgerActor', ledgerActor);
    // Query decimals and balance
    const decimals = Number(await ledgerActor.icrc1_decimals());
    console.log('decimals', decimals);
    const principal = await window.ic.plug.agent.getPrincipal();
    console.log('principal', principal);
    const balance = await ledgerActor.icrc1_balance_of({ owner: principal, subaccount: [] });
    console.log('balance', balance);
    const fee = await ledgerActor.icrc1_fee();
    console.log('fee', fee);


    // Convert human amount to Nat using decimals (no floating point)
    const scale = BigInt(10) ** BigInt(decimals);
    const [intPartRaw, fracPartRaw = ''] = toStakeAmount.value.toString().split('.');
    const intPart = intPartRaw.replace(/[^0-9]/g, '') || '0';
    const fracPadded = (fracPartRaw.replace(/[^0-9]/g, '') + '0'.repeat(decimals)).slice(0, decimals);
    const amountNat = BigInt(intPart) * scale + BigInt(fracPadded || '0');


    if (balance < amountNat + fee) {
      transactionError.value = `Insufficient BTC balance.`;
      return;
    }

    await handleStakeProcess(amountNat);
  } catch (error) {

    if (firstTxStatus.value !== "failed") {
      secondTxStatus.value = "failed";
      transactionError.value = 'Transaction failed. Please try again';
    }
  }
};
</script>