<template>
  <div class="min-h-[85vh] bg-gray-50">
    <div class="max-w-6xl mx-auto px-4 py-8">
      <!-- Main Container -->
      <div class="bg-white rounded-xl p-6 mb-8 border border-gray-100 transition-all duration-300">
        <!-- Header with Dashboard Section -->
        <div class="grid grid-cols-1 lg:grid-cols-2 gap-8 mb-6">
          <!-- Left: Title and APR Display -->
          <div class="flex flex-col space-y-6">
            <div>
              <h1 class="text-2xl md:text-3xl font-bold text-gray-900 flex items-center gap-3 mb-2">
                <svg class="w-8 h-8 text-yellow-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                    d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                Insurance Pool
              </h1>
              <p class="text-gray-500 text-lg text-left">Stake your BTC to earn rewards</p>
            </div>

            <!-- APR Display -->
            <div
              class="bg-gradient-to-r from-yellow-50 to-yellow-100 p-10 rounded-xl border border-yellow-200 transform transition-all duration-300 hover:shadow-md">
              <div class="flex items-center gap-3 mb-3">
                <svg class="w-6 h-6 text-yellow-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                    d="M13 7h8m0 0v8m0-8l-8 8-4-4-6 6" />
                </svg>
                <span class="text-lg font-medium text-gray-700">Current APR</span>
              </div>
              <div class="text-5xl font-bold text-yellow-600 flex items-baseline">
                {{ poolAPR }}<span class="text-2xl ml-1">%</span>
              </div>
            </div>
          </div>
          <!-- Right: Your Overview Cards -->
          <div class="flex flex-col">
            <h2 class="text-xl font-semibold text-gray-900 mb-4 flex items-center gap-2">
              Pool Overview
              <span v-if="isLoading" class="text-sm text-gray-500">(Loading...)</span>
            </h2>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4 flex-grow">
              <div
                class="bg-gray-50 p-5 rounded-xl border border-gray-200 hover:shadow-sm transition-all duration-300 flex flex-col">
                <div class="text-sm text-gray-600 mb-1 flex items-center gap-2">
                  Your Total Stake
                </div>
                <div class="text-2xl font-semibold text-gray-900 mt-1">{{ userTotalStakedAmount }} <span
                    class="text-lg font-medium text-gray-700">BTC</span></div>
              </div>

              <div
                class="bg-gray-50 p-5 rounded-xl border border-gray-200 hover:shadow-sm transition-all duration-300 flex flex-col">
                <div class="text-sm text-gray-600 mb-1 flex items-center gap-2">
                  Pool TVL
                </div>
                <div class="text-2xl font-semibold text-gray-900 mt-1">{{ totalStakedAmount }} <span
                    class="text-lg font-medium text-gray-700">BTC</span></div>
              </div>

              <div
                class="bg-gradient-to-r from-gray-50 to-yellow-50 p-5 rounded-xl border border-yellow-100 md:col-span-2 hover:shadow-sm transition-all duration-300">
                <div class="text-sm text-gray-700 mb-1 flex items-center gap-2">
                  Available Rewards
                </div>
                <div class="flex items-center justify-between">
                  <div class="text-2xl font-semibold text-gray-900 mt-1">{{ earnedRewards }} <span
                      class="text-lg font-medium text-gray-700">BTC</span></div>
                  <button @click="claimRewards"
                    :disabled="!earnedRewards || earnedRewards === '0.00' || isClaimingRewards || unstakingDepositId"
                    class="btn-secondary px-4 py-2 rounded-lg text-sm disabled:opacity-50 disabled:cursor-not-allowed transition-all duration-300">
                    <span v-if="isClaimingRewards">Processing...</span>
                    <span v-else>Claim</span>
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>


        <!-- Action Buttons -->
        <div class="flex justify-center gap-4 mt-2">
          <button @click="openNewPositionDialog"
            class="flex items-center justify-center btn-primary px-8 py-3 rounded-lg shadow-sm hover:shadow transition-all duration-300 font-medium">
            <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
            </svg>
            New Staking Position
          </button>

          <!-- Show ONLY for pool manager -->
          <button v-if="isPoolManager" @click="navigateToPoolConfig"
            class="flex items-center justify-center btn-primary px-8 py-3 rounded-lg shadow-sm hover:shadow transition-all duration-300 font-medium">
            <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
            Configure Pool
          </button>
        </div>
      </div>

      <!-- Positions Table Section -->
      <div class="bg-white rounded-xl p-0 border border-gray-100 transition-all duration-300">
        <div class="p-5 border-b border-gray-200 flex justify-between items-center">
          <h2 class="text-xl font-semibold text-gray-900 flex items-center gap-2">
            <svg class="w-5 h-5 text-yellow-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
            </svg>
            Active Positions
          </h2>
          <div class="text-sm text-gray-500">{{ positions.length }} active position{{ positions.length !== 1 ? 's' : ''
            }}
          </div>
        </div>

        <!-- Enhanced Positions Table -->
        <div class="overflow-x-auto">
          <table class="w-full text-sm text-gray-600">
            <thead>
              <tr class="bg-gray-50 text-left">
                <th class="px-6 py-4 text-center text-xs font-semibold uppercase tracking-wider text-gray-900">ID</th>
                <th class="px-6 py-4 text-center text-xs font-semibold uppercase tracking-wider text-gray-900">Unlock
                  Date
                </th>
                <th class="px-6 py-4 text-right text-xs font-semibold uppercase tracking-wider text-gray-900">Amount
                </th>
                <th class="px-6 py-4 text-center text-xs font-semibold uppercase tracking-wider text-gray-900">Status
                </th>
                <th class="px-6 py-4 text-center text-xs font-semibold uppercase tracking-wider text-gray-900">Actions
                </th>
              </tr>
            </thead>
            <tbody class="divide-y divide-gray-200">
              <tr v-if="!isConnected">
                <td colspan="5" class="px-6 py-12 text-center text-gray-500">
                  <div class="flex flex-col items-center justify-center">
                    <svg class="w-12 h-12 text-gray-300 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                        d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
                    </svg>
                    <p>Failed to connect to canister</p>
                    <button @click="initializeICP" class="mt-2 px-4 py-2 bg-blue-500 text-white rounded-lg">Retry
                      Connection</button>
                  </div>
                </td>
              </tr>
              <tr v-else-if="isLoading">
                <td colspan="5" class="px-6 py-12 text-center text-gray-500">
                  <div class="flex flex-col items-center justify-center">
                    <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-yellow-500 mb-4"></div>
                    <p>Loading your positions...</p>
                  </div>
                </td>
              </tr>
              <tr v-else-if="positions.length === 0">
                <td colspan="5" class="px-6 py-12 text-center text-gray-500">
                  <div class="flex flex-col items-center justify-center">
                    <!-- Wallet not connected state -->
                    <div v-if="!userPrincipal" class="flex flex-col items-center justify-center">
                      <svg class="w-12 h-12 text-gray-300 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                          d="M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2-2v-5m16 0h-2.586a1 1 0 00-.707.293l-2.414 2.414a1 1 0 01-.707.293h-3.172a1 1 0 01-.707-.293l-2.414-2.414A1 1 0 006.586 13H4" />
                      </svg>
                      <p class="mb-4">Connect wallet to see your positions</p>
                    </div>

                    <!-- Wallet connected but no positions state -->
                    <div v-else class="flex flex-col items-center justify-center">
                      <svg class="w-12 h-12 text-gray-300 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                          d="M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2-2v-5m16 0h-2.586a1 1 0 00-.707.293l-2.414 2.414a1 1 0 01-.707.293h-3.172a1 1 0 01-.707-.293l-2.414-2.414A1 1 0 006.586 13H4" />
                      </svg>
                      <p>No active positions found for current principal</p>
                    </div>
                  </div>
                </td>
              </tr>
              <tr v-for="position in positions" :key="position.deposit_id"
                class="hover:bg-gray-50 transition-all duration-300">
                <td class="px-6 py-5">
                  <div class="flex justify-center items-center">
                    <span
                      class="inline-flex items-center justify-center min-w-[2.5rem] px-2.5 py-0.5 text-xs font-medium bg-gray-100 text-gray-800 rounded-full">
                      {{ position.deposit_id }}
                    </span>
                  </div>
                </td>
                <td class="px-6 py-5">
                  <div class="flex justify-center items-center text-gray-600">
                    {{ position.unlockDate }}
                  </div>
                </td>
                <td class="px-6 py-5 text-right font-medium whitespace-nowrap">
                  {{ formatAmount(position.amount) }}
                  <span class="ml-1 text-gray-500 font-normal">BTC</span>
                </td>
                <td class="px-6 py-5">
                  <div class="flex justify-center">
                    <span
                      class="inline-flex items-center px-3 py-1 rounded-full text-xs font-medium transition-colors duration-300"
                      :class="position.isUnlocked ? 'bg-green-100 text-green-800' : 'bg-yellow-100 text-yellow-800'">
                      <span class="w-2 h-2 rounded-full mr-1"
                        :class="position.isUnlocked ? 'bg-green-500' : 'bg-yellow-500'"></span>
                      {{ position.isUnlocked ? 'Unlocked' : 'Locked' }}
                    </span>
                  </div>
                </td>
                <td class="px-6 py-5 text-center">
                  <button @click="unstakePosition(position.deposit_id)"
                    :disabled="!position.isUnlocked || isClaimingRewards || unstakingDepositId"
                    class="btn-secondary px-4 py-2 rounded-lg disabled:opacity-50 disabled:cursor-not-allowed transition-all duration-300 hover:shadow-sm">
                    <span class="flex items-center">
                      <span v-if="unstakingDepositId === position.deposit_id">Processing...</span>
                      <span v-else>Unstake</span>
                    </span>
                  </button>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>


    <!-- New Position Dialog -->
    <NewPositionDialog v-if="isNewPositionDialogOpen" :is-open="isNewPositionDialogOpen" @close="closeNewPositionDialog"
      @position-created="handlePositionCreated" />

    <!-- Error Modal -->
    <div v-if="errorMessage" class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div class="bg-white rounded-lg p-6 max-w-md mx-4">
        <div class="flex items-center gap-3 mb-4">
          <svg class="w-6 h-6 text-red-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
              d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <h3 class="text-lg font-semibold text-gray-900">Error</h3>
        </div>
        <p class="text-gray-600 mb-4">{{ errorMessage }}</p>
        <div class="flex gap-2">
          <button @click="errorMessage = ''"
            class="flex-1 px-4 py-2 bg-gray-500 text-white rounded-lg hover:bg-gray-600 transition-colors">
            Close
          </button>
          <button @click="retryConnection"
            class="flex-1 px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors">
            Retry
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted, watch } from "vue";
import { useRouter } from "vue-router";
import { getCurrentNetwork, getCanisterIds } from "../constants/icp.js";
import { createBackendActor, createBackendActorWithPlug } from "../utils/icpAgent.js";
import { Principal } from "@dfinity/principal";
import NewPositionDialog from "../components/NewPositionDialog.vue";
import { useWeb3Store } from "../stores/web3Store";
import { handlePlugError } from "../utils/errorHandler.js";

// Router
const router = useRouter();

// State
const positions = ref([]);
const totalStakedAmount = ref('0.00');
const userTotalStakedAmount = ref('0.00');
const earnedRewards = ref('0.00');
const poolAPR = ref('0.00');
const isNewPositionDialogOpen = ref(false);
const isLoading = ref(false);
const isConnected = ref(false);
const isClaimingRewards = ref(false);
const unstakingDepositId = ref(null);
const errorMessage = ref('');
const debugInfo = ref(null);
const isPoolManager = ref(false);
// BigInt-safe serializer for debug panel
const toPlain = (value) => {
  if (typeof value === 'bigint') return value.toString();
  if (Array.isArray(value)) return value.map((v) => toPlain(v));
  if (value && typeof value === 'object') {
    const obj = {};
    for (const [k, v] of Object.entries(value)) obj[k] = toPlain(v);
    return obj;
  }
  return value;
};

// Network info
const currentNetwork = ref('');
const currentHost = ref('');
const backendCanisterId = ref('');
let backendActor = null;

// Connected wallet principal
const web3Store = useWeb3Store();
const userPrincipal = computed(() => web3Store.account);

// Initialize network configuration
const initializeNetwork = () => {
  currentNetwork.value = getCurrentNetwork();
  const canisterIds = getCanisterIds();
  backendCanisterId.value = canisterIds.backend;

  console.log('Network config:', {
    currentNetwork: currentNetwork.value,
    canisterIds: canisterIds,
    backendCanisterId: backendCanisterId.value
  });

  if (currentNetwork.value === 'local') {
    currentHost.value = 'http://127.0.0.1:4943';
  } else {
    currentHost.value = 'https://icp0.io';
  }
};

// Thin wrapper on actor methods
const makeCanisterCall = async (method, args = []) => {
  if (!backendActor) throw new Error('Actor not initialized');
  switch (method) {
    case 'get_pool_state':
      return await backendActor.get_pool_state();
    case 'get_pool_reward_rate':
      return await backendActor.get_pool_reward_rate();
    case 'get_current_episode_id':
      return await backendActor.get_current_episode_id();
    case 'get_user_deposits':
      return await backendActor.get_user_deposits(
        typeof args[0] === 'string' ? Principal.fromText(args[0]) : args[0]
      );
    case 'get_deposits_rewards':
      return await backendActor.get_deposits_rewards(args[0]);
    default:
      throw new Error(`Unknown method ${method}`);
  }
};

const formatAmount = (amountE8s) => {
  if (!amountE8s) return '0.00';

  const amount = Number(amountE8s) / 100000000;
  return amount.toFixed(4);
};

// Get current episode
const getCurrentEpisode = () => {
  const episodeDuration = 91 * 24 * 60 * 60 / 3; // ~30.33 days in seconds
  const currentTime = Math.floor(Date.now() / 1000);
  return Math.floor(currentTime / episodeDuration);
};
// Calculate unlock date from episode
const getUnlockDate = (episode) => {
  const episodeNumber = Number(episode.toString());
  const episodeDuration = 91 * 24 * 60 * 60 / 3; // ~30.33 days in seconds
  const unlockTimestamp = (episodeNumber + 1) * episodeDuration * 1000; // Convert to milliseconds
  const date = new Date(unlockTimestamp);
  return date.toLocaleDateString('en-US', { year: 'numeric', month: 'short', day: 'numeric' });
};
// Initialize ICP connection
const initializeICP = async () => {
  try {
    console.log('Initializing ICP connection...');
    initializeNetwork();

    // Init actor directly; agent will fetch root key internally on local
    console.log('Creating backend actor with:', {
      canisterId: backendCanisterId.value,
      host: currentHost.value
    });

    backendActor = await createBackendActor(backendCanisterId.value, currentHost.value);

    // Mark as connected if backend canister id is present
    if (backendCanisterId.value && backendActor) {
      isConnected.value = true;
      console.log('ICP connection successful');
    } else {
      throw new Error('Backend canister ID not set');
    }

  } catch (error) {
    console.error('Failed to initialize ICP:', error);
    isConnected.value = false;
    errorMessage.value = `Failed to connect to canister: ${error.message}`;
  }
};

// Load pool state from canister
const loadPoolState = async () => {
  try {
    console.log('Loading pool state...');

    // Call get_pool_state
    const poolStateResult = await makeCanisterCall('get_pool_state');
    console.log('Pool state result:', poolStateResult);

    if (poolStateResult && poolStateResult.total_assets !== undefined) {
      totalStakedAmount.value = formatAmount(poolStateResult.total_assets);
    }

    // Call get_pool_reward_rate  
    const rewardRateResult = await makeCanisterCall('get_pool_reward_rate');
    console.log('Reward rate result:', rewardRateResult);

    // Calculate approximate APR
    if (poolStateResult && rewardRateResult) {
      console.log('poolStateResult', poolStateResult);
      console.log('rewardRateResult', rewardRateResult);
      const totalAssets = Number(poolStateResult.total_assets);
      // const rewardRate = Number(rewardRateResult) || 0;

      // Simple APR calculation (annual reward rate / total assets * 100)
      // const annualRewards = rewardRate * 365 * 24 * 3600;
      const apr = Number(rewardRateResult * 365n * 24n * 3600n * 100n / 1_000_000_000_000_000_000n) / Number(poolStateResult.total_assets);
      poolAPR.value = apr.toFixed(2);
    }

  } catch (error) {
    console.error('Error loading pool state:', error);
    // Set defaults on error
    totalStakedAmount.value = '0.00';
    poolAPR.value = '0.00';
  }
};

// Load user positions from canister
const loadUserPositions = async () => {
  try {
    isLoading.value = true;

    // Only load user positions if wallet is connected
    if (!userPrincipal.value) {
      console.log('No wallet connected, skipping user positions load');
      positions.value = [];
      userTotalStakedAmount.value = '0.00';
      earnedRewards.value = '0.00';
      return;
    }

    console.log('Loading user positions for principal:', userPrincipal.value);

    // Call get_user_deposits
    const userDepositsResult = await makeCanisterCall('get_user_deposits', [userPrincipal.value]);
    console.log('User deposits result:', userDepositsResult);

    if (Array.isArray(userDepositsResult)) {
      const currentEpisode = getCurrentEpisode();

      positions.value = userDepositsResult.map(deposit => ({
        deposit_id: deposit.deposit_id,
        episode: deposit.episode,
        shares: deposit.shares,
        amount: deposit.amount,
        unlockDate: getUnlockDate(deposit.episode),
        isUnlocked: deposit.episode < currentEpisode
      }));

      // Calculate total staked amount
      const totalStaked = userDepositsResult.reduce((sum, deposit) => sum + Number(deposit.amount), 0);
      userTotalStakedAmount.value = formatAmount(totalStaked);

      // Get rewards if we have deposits
      if (userDepositsResult.length > 0) {
        const depositIds = userDepositsResult.map(d => d.deposit_id);
        const rewardsResult = await makeCanisterCall('get_deposits_rewards', [depositIds]);
        console.log('Rewards result:', rewardsResult);

        if (rewardsResult !== undefined) {
          const amount = Number(rewardsResult) / 100000000;
          earnedRewards.value = amount.toFixed(8);
        }
      } else {
        earnedRewards.value = '0.00';
      }
    } else {
      positions.value = [];
      userTotalStakedAmount.value = '0.00';
      earnedRewards.value = '0.00';
    }

    // Update debug info (BigInt-safe)
    debugInfo.value = toPlain({
      network: currentNetwork.value,
      canisterId: backendCanisterId.value,
      host: currentHost.value,
      userPrincipal: userPrincipal.value,
      currentEpisode: getCurrentEpisode(),
      userDeposits: userDepositsResult,
      lastUpdated: new Date().toISOString()
    });

  } catch (error) {
    console.error('Error loading user positions:', error);
    positions.value = [];
    userTotalStakedAmount.value = '0.00';
    earnedRewards.value = '0.00';

    // Still update debug info with error
    debugInfo.value = toPlain({
      error: error.message,
      network: currentNetwork.value,
      canisterId: backendCanisterId.value,
      host: currentHost.value,
      lastUpdated: new Date().toISOString()
    });
  } finally {
    isLoading.value = false;
  }
};

// Claim rewards (this would be an update call, not query)
const claimRewards = async () => {
  try {
    isClaimingRewards.value = true;

    const depositIds = positions.value.map(p => p.deposit_id);

    if (depositIds.length === 0) {
      throw new Error('No positions found to claim rewards from');
    }

    console.log('Claiming rewards for deposits:', depositIds);

    const backendActor = await createBackendActorWithPlug(backendCanisterId.value);
    try {
      await backendActor.withdraw_rewards(depositIds);
    } catch (error) {
      handlePlugError(error);
    }
    await loadAllData();

  } catch (error) {
    console.error('Error claiming rewards:', error);
    errorMessage.value = 'Failed to claim rewards: ' + error.message;
  } finally {
    isClaimingRewards.value = false;
  }
};

// Unstake position (this would be an update call, not query)
const unstakePosition = async (depositId) => {
  try {
    unstakingDepositId.value = depositId;

    console.log('Unstaking position:', depositId);

    // This would be an update call to withdraw
    // For now, we'll just simulate it
    alert(`Would unstake deposit ID: ${depositId}\n\nThis requires an update call which needs proper authentication.`);

  } catch (error) {
    console.error('Error unstaking position:', error);
    errorMessage.value = 'Failed to unstake position: ' + error.message;
  } finally {
    unstakingDepositId.value = null;
  }
};

// Dialog methods
const openNewPositionDialog = () => {
  isNewPositionDialogOpen.value = true;
};

const closeNewPositionDialog = () => {
  isNewPositionDialogOpen.value = false;
};

const handlePositionCreated = async () => {
  await loadUserPositions();
  await loadPoolState();
};

// Check if current user is the pool manager
const checkIfPoolManager = async () => {
  try {
    if (!userPrincipal.value || !backendActor) {
      isPoolManager.value = false;
      return;
    }

    // Call the backend to get the pool manager principal
    const poolManagerPrincipal = await backendActor.get_pool_manager_principal();
    const poolManagerPrincipalString = poolManagerPrincipal.toText();

    // Check if the connected user is the pool manager
    isPoolManager.value = userPrincipal.value === poolManagerPrincipalString;

    console.log('Pool manager check:', {
      userPrincipal: userPrincipal.value,
      poolManagerPrincipal: poolManagerPrincipalString,
      isPoolManager: isPoolManager.value
    });
  } catch (error) {
    console.error('Error checking pool manager:', error);
    isPoolManager.value = false;
  }
};

// Navigate to pool configuration page
const navigateToPoolConfig = () => {
  router.push('/underwriter');
};

// Retry connection
const retryConnection = async () => {
  errorMessage.value = '';
  await loadAllData();
};

// Load all data
const loadAllData = async () => {
  await loadPoolState();
  await loadUserPositions();
  await checkIfPoolManager();
};

// Initialize on component mount
onMounted(async () => {
  await initializeICP();
  await loadAllData();
});

// Watch for account changes and reload data
watch(() => web3Store.account, async (newAccount, oldAccount) => {
  console.log('Account changed from', oldAccount, 'to', newAccount);
  await loadAllData();
});
</script>