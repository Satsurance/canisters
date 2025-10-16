<!-- src/pages/Faucet.vue -->
<template>
  <div class="min-h-[85vh] bg-gray-50">
    <!-- Main Content -->
    <div class="max-w-6xl mx-auto px-4 py-8">
      <!-- Header with background -->
      <div class="bg-white rounded-lg shadow-sm p-6 mb-8">
        <div class="flex justify-between items-center mb-8">
          <div class="space-y-1">
            <h1 class="text-4xl font-semibold text-gray-900 flex items-center gap-3">
              <svg class="w-8 h-8 text-yellow-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              Test Faucet
            </h1>
          </div>
        </div>

        <!-- Instructions Steps -->
        <div class="bg-gray-50 rounded-lg p-6 mb-8">
          <h2 class="text-lg font-medium text-gray-900 mb-4">Follow these steps:</h2>
          <div class="space-y-4">
            <div class="flex items-start gap-3">
              <div class="flex-shrink-0 w-8 h-8 bg-yellow-100 rounded-full flex items-center justify-center">
                <span class="text-yellow-600 font-medium">1</span>
              </div>
              <div>
                <p class="text-gray-700">Install the <a href="https://plugwallet.ooo/" target="_blank"
                    class="text-yellow-600 hover:text-yellow-700 underline">Plug wallet</a> browser extension.</p>
              </div>
            </div>
            <div class="flex items-start gap-3">
              <div class="flex-shrink-0 w-8 h-8 bg-yellow-100 rounded-full flex items-center justify-center">
                <span class="text-yellow-600 font-medium">2</span>
              </div>
              <div>
                <p class="text-gray-700">Connect your Plug wallet using the button in the top right corner.</p>
              </div>
            </div>
            <div class="flex items-start gap-3">
              <div class="flex-shrink-0 w-8 h-8 bg-yellow-100 rounded-full flex items-center justify-center">
                <span class="text-yellow-600 font-medium">3</span>
              </div>
              <div>
                <p class="text-gray-700">Request test BTC tokens using the form below.</p>
              </div>
            </div>
            <div class="flex items-start gap-3">
              <div class="flex-shrink-0 w-8 h-8 bg-yellow-100 rounded-full flex items-center justify-center">
                <span class="text-yellow-600 font-medium">4</span>
              </div>
              <div>
                <p class="text-gray-700 italic "><strong>Go and test Satsurance functionality!</strong></p>
              </div>
            </div>
          </div>
        </div>

        <!-- Info Stats Card -->
        <div class="bg-white rounded-lg mb-8">
          <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
            <!-- Max Amount -->
            <div class="flex flex-col items-center justify-center p-6 bg-gray-50 rounded-lg">
              <div class="flex items-center gap-2 text-gray-600 mb-2">
                <svg class="w-5 h-5 text-yellow-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                    d="M13 7h8m0 0v8m0-8l-8 8-4-4-6 6" />
                </svg>
                <span>Maximum Request</span>
              </div>
              <div class="text-2xl font-medium">0.1 BTC</div>
            </div>

            <!-- Current Balance -->
            <div class="flex flex-col items-center justify-center p-6 bg-gray-50 rounded-lg">
              <div class="flex items-center gap-2 text-gray-600 mb-2">
                <svg class="w-5 h-5 text-yellow-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                    d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                <span>Your Balance</span>
              </div>
              <div class="text-2xl font-medium">
                <div>{{ currentBTCBalance }} BTC</div>
              </div>
            </div>

            <!-- Network -->
            <div class="flex flex-col items-center justify-center p-6 bg-gray-50 rounded-lg">
              <div class="flex items-center gap-2 text-gray-600 mb-2">
                <svg class="w-5 h-5 text-yellow-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                    d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9" />
                </svg>
                <span>Network</span>
              </div>
              <div class="text-2xl font-medium">{{ faucetDetails.networkName }}</div>
            </div>
          </div>
        </div>

        <!-- Faucet Interface -->
        <div class="bg-white rounded-lg p-6 py-12">
          <div class="max-w-lg mx-auto space-y-6">
            <div class="space-y-4">
              <!-- BTC Input -->
              <div class="flex flex-col">
                <label for="btc-amount" class="block mb-2 text-sm font-medium text-gray-900">
                  BTC Amount to Request
                </label>
                <div class="flex items-center space-x-4">
                  <input type="number" id="btc-amount" v-model="requestBTCAmount"
                    :disabled="isLoading || !web3Store.isConnected"
                    class="flex-1 bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-yellow-500 focus:border-yellow-500 focus:outline-none block w-full p-2.5"
                    placeholder="0.01" min="0.00001" max="0.1" required />
                  <button @click="requestBTCTokens" :disabled="(isLoading || !web3Store.isConnected)" :class="[
                    'px-8 py-2.5 rounded-lg',
                    (isLoading || !web3Store.isConnected) ? 'bg-gray-300 hover:border-slate-600 cursor-not-allowed' : 'btn-primary'
                  ]">
                    {{ isLoading && currentTokenType === 'BTC' ? 'Requesting...' : 'Request BTC' }}
                  </button>
                </div>
                <div v-if="!web3Store.isConnected" class="mt-2 text-sm text-gray-500">
                  Please connect your wallet first
                </div>
                <div v-if="btcError" class="mt-2 text-sm text-red-500">
                  {{ btcError }}
                </div>
              </div>

            </div>

            <div v-if="web3Store.isConnected" class="bg-yellow-50 rounded-lg p-4 text-yellow-700">
              <div class="flex items-start space-x-3">
                <svg class="w-5 h-5 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                    d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                <div>
                  <p class="font-medium text-left">Important Notes:</p>
                  <ul class="mt-1 ml-4 list-disc text-sm text-left">
                    <li>Maximum request amount is 0.1 BTC</li>
                    <li>Tokens are for testing purposes only</li>
                  </ul>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Transaction Status Modal -->
    <TransactionStatus :show="!!transactionStatus" :steps="transactionSteps" :tx-hash="currentTxHash"
      :error="transactionError" :block-explorer="ICP_MAINNET_BLOCK_EXPLORER" @close="resetTransaction"
      @retry="retryTransaction" />
  </div>
</template>

<script setup>
import { ref, computed, watch } from 'vue';
import { useWeb3Store } from '../stores/web3Store';
import { ICP_MAINNET_BLOCK_EXPLORER, ICP_NETWORKS, ICP_CONFIG, getCurrentNetwork, getCanisterIds } from '../constants/icp.js';
import { createLedgerActorWithPlug } from '../utils/icpAgent.js';
import TransactionStatus from '../components/TransactionStatus.vue';

// Store setup
const web3Store = useWeb3Store();

// State
const currentBTCBalance = ref(0);
const requestBTCAmount = ref(null);
const isLoading = ref(false);
const currentTokenType = ref('');
const btcError = ref('');

// Transaction state
const transactionStatus = ref('');
const transactionType = ref('');
const currentTxHash = ref('');
const transactionError = ref('');

const transactionSteps = computed(() => {
  if (transactionType.value === 'faucet_request') {
    return [
      {
        id: 'faucet',
        title: `Request ${currentTokenType.value}`,
        description: `Receiving faucet token.`,
        status: transactionStatus.value,
        showNumber: false
      }
    ];
  }
  return [];
});

const faucetDetails = computed(() => {
  if (web3Store.chainId === ICP_NETWORKS.LOCAL) {
    return {
      networkName: "ICP Local (dfx)",
      docsPage: "https://internetcomputer.org/docs/current/developer-docs/getting-started/install/",
      description: "Local development network running on your machine"
    }
  } else if (web3Store.chainId === ICP_NETWORKS.MAINNET) {
    return {
      networkName: "ICP Mainnet",
      docsPage: "https://internetcomputer.org/docs",
      description: "Internet Computer Protocol mainnet"
    }
  } else {
    return {
      networkName: "Wallet not connected",
      docsPage: "https://internetcomputer.org/docs",
      description: "Please connect your Plug wallet"
    }
  }
});

// Methods
const loadBalances = async () => {
  try {
    if (!web3Store.isConnected) {
      currentBTCBalance.value = 0;
      return;
    }

    // Ensure Plug connection
    if (!window.ic || !window.ic.plug) {
      console.error('Plug wallet not available');
      return;
    }

    const currentNetwork = getCurrentNetwork();
    const { ledger } = getCanisterIds(currentNetwork);

    const ledgerActor = await createLedgerActorWithPlug(ledger);
    const principal = await window.ic.plug.agent.getPrincipal();

    // Get balance
    const balance = await ledgerActor.icrc1_balance_of({
      owner: principal,
      subaccount: []
    });
    console.log('balance', balance);

    // Get decimals for proper formatting
    // const decimals = Number(await ledgerActor.icrc1_decimals());
    const decimals = 8;
    const scale = BigInt(10) ** BigInt(decimals);

    currentBTCBalance.value = (Number(balance) / Number(scale)).toFixed(8);
  } catch (error) {
    console.error('Error loading balances:', error);
  }
};

const validateBTCAmount = (amount) => {
  if (!amount) {
    btcError.value = 'Please enter an amount';
    return false;
  }
  const numAmount = Number(amount);
  if (numAmount < 0.00001 || numAmount > 0.1) {
    btcError.value = 'Amount must be between 0.00001 and 0.1 BTC';
    return false;
  }
  btcError.value = '';
  return true;
};

const requestBTCTokens = async () => {
  if (!validateBTCAmount(requestBTCAmount.value)) return;
  currentTokenType.value = 'BTC';
  await requestTokens('BTC', requestBTCAmount.value);
};

const requestTokens = async (tokenType, amount) => {
  try {
    isLoading.value = true;
    transactionType.value = 'faucet_request';
    transactionStatus.value = 'pending';

    const response = await fetch(`${__API_BASE_URL__}/api/faucet/request`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        address: web3Store.account,
        chainId: 1000000, // ICP network id fixed
        amount,
        tokenType,
      })
    });

    const data = await response.json();

    if (!response.ok) {
      throw new Error(data.error || 'Failed to request tokens');
    }

    // currentTxHash.value = data.txHash || data.blockIndex || '';
    transactionStatus.value = 'success';

    // Reset form and reload balance
    requestBTCAmount.value = null;
    await loadBalances();

    // Auto-close success message
    setTimeout(resetTransaction, 3000);
  } catch (error) {
    console.error('Error requesting tokens:', error);
    transactionStatus.value = 'failed';
    transactionError.value = error.message || 'Failed to request tokens';
  } finally {
    isLoading.value = false;
  }
};

const retryTransaction = () => {
  requestBTCTokens();
};

const resetTransaction = () => {
  transactionStatus.value = '';
  transactionType.value = '';
  currentTxHash.value = '';
  transactionError.value = '';
  currentTokenType.value = '';
  btcError.value = '';
};

// Watchers
watch(
  () => [web3Store.isConnected, web3Store.account],
  async ([newAccount, oldAccount, isConnected]) => {
    console.log('Account changed from', oldAccount, 'to', newAccount);
    console.log('isConnected', web3Store.isConnected);
    if (web3Store.isConnected) {
      await loadBalances();
    } else {
      currentBTCBalance.value = 0;
    }
  },
  { immediate: true }
);
</script>