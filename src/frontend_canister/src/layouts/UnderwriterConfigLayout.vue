<template>
  <div class="min-h-[85vh] bg-gray-50">
    <div class="max-w-6xl mx-auto px-4 py-8">
      <!-- Header Section -->
      <div class="bg-white rounded-lg shadow-sm p-6 mb-8">
        <div class="flex flex-col md:flex-row justify-between items-center">
          <div class="flex flex-col items-center md:items-start text-center md:text-left space-y-2">
            <h1 class="text-2xl md:text-4xl font-semibold text-gray-900 flex items-center gap-3">
              <button @click="$router.push('/stake')"
                class="p-2 bg-gray-100 hover:bg-gray-200 text-gray-700 hover:text-gray-900 rounded-lg transition-all duration-300 shadow-sm hover:shadow-md border border-gray-200 hover:border-gray-300"
                title="Back to Pool Staking">
                <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
                </svg>
              </button>
              <svg class="w-8 h-8 text-yellow-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
              </svg>
              Underwriter Dashboard
            </h1>
            <p class="text-gray-500">Manage pool settings, products, and monitor performance</p>
          </div>
        </div>
      </div>

      <!-- Access Control Check -->
      <div v-if="!isPoolManager && userPrincipal" class="bg-red-50 border border-red-200 rounded-lg p-6 mb-8">
        <div class="flex items-center space-x-3">
          <svg class="w-6 h-6 text-red-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
              d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
          </svg>
          <div>
            <h3 class="text-red-800 font-semibold">Access Denied</h3>
            <p class="text-red-600">Only the underwriter (executor) can access this dashboard.</p>
          </div>
        </div>
      </div>

      <!-- Dashboard Content -->
      <div v-if="isPoolManager || !userPrincipal" class="space-y-8">
        <!-- Pool Statistics -->
        <div class="bg-white rounded-lg shadow-sm p-6">
          <h2 class="text-xl font-semibold text-gray-900 mb-6 flex items-center gap-2">
            <svg class="w-5 h-5 text-yellow-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
            </svg>
            Pool Statistics
          </h2>

          <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
            <div class="bg-blue-50 p-6 rounded-lg border border-blue-200">
              <div class="text-sm text-blue-700 mb-1">Total Assets Staked</div>
              <div class="text-2xl font-bold text-blue-600">{{ formatBTC(totalAssetsStaked) }}</div>
            </div>
            <div class="bg-green-50 p-6 rounded-lg border border-green-200">
              <div class="text-sm text-green-700 mb-1">End of Current Episode</div>
              <div class="text-2xl font-bold text-green-600">{{ currentEpisodeEndDate }}</div>
            </div>
            <div class="bg-purple-50 p-6 rounded-lg border border-purple-200">
              <div class="text-sm text-purple-700 mb-1">Pool APR</div>
              <div class="text-2xl font-bold text-purple-600">{{ poolAPR }}%</div>
            </div>
            <div class="bg-orange-50 p-6 rounded-lg border border-orange-200">
              <div class="text-sm text-orange-700 mb-1">Cover Allocation</div>
              <div class="text-2xl font-bold text-orange-600">{{ formatBTC(totalCoverAllocation) }}</div>
            </div>
          </div>
        </div>

        <!-- Rewards Management (UI Only - Not Interactive) -->
        <div class="bg-white rounded-lg shadow-sm p-6 opacity-60">
          <div class="flex items-center gap-2 mb-6">
            <h2 class="text-xl font-semibold text-gray-900 flex items-center gap-2">
              <svg class="w-5 h-5 text-yellow-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              Rewards Management
            </h2>
            <span class="text-xs bg-gray-200 text-gray-600 px-2 py-1 rounded">Coming Soon</span>
          </div>

          <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div class="space-y-4">
              <div>
                <label class="block text-sm font-medium text-gray-700">Available Rewards</label>
                <div class="text-2xl font-bold text-gray-400">0.000000 BTC</div>
              </div>
              <button disabled class="btn-primary px-4 py-2 rounded-lg opacity-50 cursor-not-allowed w-full">
                Collect Rewards
              </button>
            </div>

            <div class="space-y-4">
              <div class="flex items-center justify-center">
                <div class="text-center">
                  <label class="block text-sm font-medium text-gray-700">Current Underwriter Fee</label>
                  <div class="text-lg font-semibold text-gray-400">0.00%</div>
                </div>
              </div>

              <div class="flex items-center gap-3">
                <label class="text-sm font-medium text-gray-700 whitespace-nowrap">Update Fee (%):</label>
                <input type="number" min="0" max="10" step="0.1" class="form-input w-20 opacity-50 cursor-not-allowed"
                  placeholder="10" disabled />
                <button disabled class="btn-primary px-4 py-2 rounded-lg opacity-50 cursor-not-allowed">
                  Update
                </button>
              </div>
            </div>
          </div>
        </div>

        <!-- Pool Settings (UI Only - Not Interactive) -->
        <div class="bg-white rounded-lg shadow-sm p-6 opacity-60">
          <div class="flex items-center gap-2 mb-6">
            <h2 class="text-xl font-semibold text-gray-900 flex items-center gap-2">
              <svg class="w-5 h-5 text-yellow-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 100 4m0-4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 100 4m0-4v2m0-6V4" />
              </svg>
              Pool Settings
            </h2>
            <span class="text-xs bg-gray-200 text-gray-600 px-2 py-1 rounded">Coming Soon</span>
          </div>

          <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div class="space-y-4">
              <div class="flex items-center justify-between">
                <div>
                  <label class="block text-sm font-medium text-gray-700">New Deposits</label>
                  <p class="text-sm text-gray-500">Allow new users to deposit into the pool</p>
                </div>
                <div class="flex items-center">
                  <input type="checkbox" disabled
                    class="h-5 w-5 text-yellow-600 border-gray-300 rounded opacity-50 cursor-not-allowed" />
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Products Management (INTERACTIVE) -->
        <div class="bg-white rounded-lg shadow-sm p-6">
          <div class="flex justify-between items-center mb-6">
            <h2 class="text-xl font-semibold text-gray-900 flex items-center gap-2">
              <svg class="w-5 h-5 text-yellow-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4" />
              </svg>
              Insurance Products
            </h2>
            <button @click="openCreateProductModal" :disabled="!isPoolManager"
              class="btn-primary px-4 py-2 rounded-lg flex items-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
              </svg>
              Create Product
            </button>
          </div>

          <!-- Products List -->
          <div v-if="isLoadingProducts" class="text-center py-8">
            <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-yellow-500 mx-auto mb-4"></div>
            <p class="text-gray-500">Loading products...</p>
          </div>

          <div v-else-if="products.length > 0" class="space-y-4">
            <div v-for="product in products" :key="product.product_id"
              class="border border-gray-200 rounded-lg p-4 hover:border-yellow-300 hover:bg-yellow-50 transition-colors cursor-pointer"
              @click="openEditProductModal(product)">
              <div class="flex-1">
                <div class="flex items-center gap-3 mb-2">
                  <h3 class="font-semibold text-gray-900">{{ product.name }}</h3>
                  <span :class="[
                    'px-2 py-1 text-xs rounded-full',
                    product.active ? 'bg-green-100 text-green-700' : 'bg-red-100 text-red-700'
                  ]">
                    {{ product.active ? 'Active' : 'Inactive' }}
                  </span>
                </div>
                <div class="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
                  <div>
                    <span class="text-gray-500">Annual Rate:</span>
                    <span class="font-medium text-gray-900 ml-1">{{ (Number(product.annual_percent) / 100).toFixed(2)
                    }}%</span>
                  </div>
                  <div>
                    <span class="text-gray-500">Max Duration:</span>
                    <span class="font-medium text-gray-900 ml-1">{{ formatDuration(product.max_coverage_duration)
                    }}</span>
                  </div>
                  <div>
                    <span class="text-gray-500">Pool Allocation:</span>
                    <span class="font-medium text-gray-900 ml-1">{{ (Number(product.max_pool_allocation_percent) /
                      100).toFixed(2) }}%</span>
                  </div>
                  <div>
                    <span class="text-gray-500">Current Allocation:</span>
                    <span class="font-medium text-gray-900 ml-1">{{ formatBTC(product.allocation) }}</span>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <div v-else class="text-center py-8 text-gray-500">
            No products created yet. Create your first product to start offering insurance coverage.
          </div>
        </div>
      </div>
    </div>

    <!-- Create Product Modal -->
    <div v-if="isCreateProductModalOpen" class="fixed inset-0 z-50 overflow-y-auto" @click="closeCreateProductModal">
      <div class="fixed inset-0 bg-gray-500/70 backdrop-blur-sm"></div>
      <div class="flex min-h-full items-center justify-center p-4">
        <div class="relative w-full max-w-lg transform overflow-hidden rounded-xl bg-white p-6 shadow-xl" @click.stop>
          <div class="flex items-center justify-between mb-6">
            <h3 class="text-lg font-semibold text-gray-900">Create New Product</h3>
            <button @click="closeCreateProductModal" class="rounded-full p-2 hover:bg-gray-100 transition-colors">
              <svg class="h-5 w-5 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>

          <form @submit.prevent="createProduct" class="space-y-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">Product Name</label>
              <input v-model="productForm.name" type="text" class="form-input w-full" placeholder="Enter product name"
                required />
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">Annual Premium Rate (%)</label>
              <input v-model.number="productForm.annualPercent" type="number" min="0.01" max="100" step="0.01"
                class="form-input w-full" placeholder="e.g., 5.5" required />
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">Max Coverage Duration (days)</label>
              <input v-model.number="productForm.maxCoverageDuration" type="number" min="1" max="365"
                class="form-input w-full" placeholder="e.g., 90" required />
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">Max Pool Allocation (%)</label>
              <input v-model.number="productForm.maxPoolAllocationPercent" type="number" min="0.01" max="100"
                step="0.01" class="form-input w-full" placeholder="e.g., 25" required />
            </div>

            <div class="flex space-x-3 pt-4">
              <button type="button" @click="closeCreateProductModal" class="btn-secondary flex-1 px-4 py-2 rounded-lg">
                Cancel
              </button>
              <button type="submit" :disabled="createProductLoading"
                class="btn-primary flex-1 px-4 py-2 rounded-lg disabled:opacity-50">
                {{ createProductLoading ? 'Creating...' : 'Create Product' }}
              </button>
            </div>
          </form>
        </div>
      </div>
    </div>

    <!-- Edit Product Modal -->
    <div v-if="isEditProductModalOpen" class="fixed inset-0 z-50 overflow-y-auto" @click="closeEditProductModal">
      <div class="fixed inset-0 bg-gray-500/70 backdrop-blur-sm"></div>
      <div class="flex min-h-full items-center justify-center p-4">
        <div class="relative w-full max-w-lg transform overflow-hidden rounded-xl bg-white p-6 shadow-xl" @click.stop>
          <div class="flex items-center justify-between mb-6">
            <h3 class="text-lg font-semibold text-gray-900">Edit Product</h3>
            <button @click="closeEditProductModal" class="rounded-full p-2 hover:bg-gray-100 transition-colors">
              <svg class="h-5 w-5 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>

          <form @submit.prevent="updateProduct" class="space-y-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">Annual Premium Rate (%)</label>
              <input v-model.number="editProductForm.annualPercent" type="number" min="0.01" max="100" step="0.01"
                class="form-input w-full" required />
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">Max Coverage Duration (days)</label>
              <input v-model.number="editProductForm.maxCoverageDuration" type="number" min="1" max="365"
                class="form-input w-full" required />
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">Max Pool Allocation (%)</label>
              <input v-model.number="editProductForm.maxPoolAllocationPercent" type="number" min="0.01" max="100"
                step="0.01" class="form-input w-full" required />
            </div>

            <div class="flex items-center justify-between">
              <label class="block text-sm font-medium text-gray-700">Active Status</label>
              <div class="flex items-center">
                <input type="checkbox" v-model="editProductForm.active"
                  class="h-5 w-5 text-yellow-600 focus:ring-yellow-500 border-gray-300 rounded" />
              </div>
            </div>

            <div class="flex space-x-3 pt-4">
              <button type="button" @click="closeEditProductModal" class="btn-secondary flex-1 px-4 py-2 rounded-lg">
                Cancel
              </button>
              <button type="submit" :disabled="updateProductLoading"
                class="btn-primary flex-1 px-4 py-2 rounded-lg disabled:opacity-50">
                {{ updateProductLoading ? 'Updating...' : 'Update Product' }}
              </button>
            </div>
          </form>
        </div>
      </div>
    </div>

    <!-- Transaction Status Modal -->
    <TransactionStatus :show="showTransactionStatus" :steps="transactionSteps" :tx-hash="currentTxHash"
      :error="transactionError" :block-explorer="ICP_MAINNET_BLOCK_EXPLORER" @close="resetTransaction" />
  </div>
</template>

<script setup>
import { ref, computed, onMounted, watch } from 'vue';
import { useWeb3Store } from '../stores/web3Store';
import { getCurrentNetwork, getCanisterIds, ICP_MAINNET_BLOCK_EXPLORER } from '../constants/icp.js';
import { createBackendActor, createBackendActorWithPlug } from '../utils/icpAgent.js';
import TransactionStatus from '../components/TransactionStatus.vue';

// Store
const web3Store = useWeb3Store();

// State
const isPoolManager = ref(false);
const currentNetwork = ref('');
const backendCanisterId = ref('');
const currentHost = ref('');
let backendActor = null;

// Pool Statistics
const totalAssetsStaked = ref(0);
const poolAPR = ref('0.00');
const totalCoverAllocation = ref(0);

// Products
const products = ref([]);
const isLoadingProducts = ref(false);
const isCreateProductModalOpen = ref(false);
const isEditProductModalOpen = ref(false);
const createProductLoading = ref(false);
const updateProductLoading = ref(false);
const editingProductId = ref(null);

// Transaction state
const transactionStatus = ref('');
const transactionType = ref('');
const currentTxHash = ref('');
const transactionError = ref('');
const showTransactionStatus = computed(() => !!transactionStatus.value);

// Forms
const productForm = ref({
  name: '',
  annualPercent: null,
  maxCoverageDuration: null,
  maxPoolAllocationPercent: null
});

const editProductForm = ref({
  annualPercent: 0,
  maxCoverageDuration: 0,
  maxPoolAllocationPercent: 0,
  active: true
});

// Computed
const userPrincipal = computed(() => web3Store.account);

const currentEpisodeEndDate = computed(() => {
  const currentEpisode = getCurrentEpisode();
  return getUnlockDate(currentEpisode);
});

const transactionSteps = computed(() => {
  if (transactionType.value === 'create_product') {
    return [
      {
        id: 'create',
        title: 'Create Product',
        description: 'Creating new insurance product',
        status: transactionStatus.value,
        showNumber: true
      }
    ];
  } else if (transactionType.value === 'update_product') {
    return [
      {
        id: 'update',
        title: 'Update Product',
        description: 'Updating insurance product',
        status: transactionStatus.value,
        showNumber: true
      }
    ];
  }
  return [];
});

// Methods
const formatBTC = (amount) => {
  if (!amount) return '0.000000 BTC';
  const btcAmount = Number(amount) / 100000000;
  return btcAmount.toFixed(6) + ' BTC';
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

const formatDuration = (seconds) => {
  const days = Math.floor(Number(seconds) / (24 * 60 * 60));
  return `${days} day${days !== 1 ? 's' : ''}`;
};

const resetTransaction = () => {
  transactionStatus.value = '';
  transactionType.value = '';
  currentTxHash.value = '';
  transactionError.value = '';
};

// Initialize network configuration
const initializeNetwork = () => {
  currentNetwork.value = getCurrentNetwork();
  const canisterIds = getCanisterIds();
  backendCanisterId.value = canisterIds.backend;

  if (currentNetwork.value === 'local') {
    currentHost.value = 'http://127.0.0.1:4943';
  } else {
    currentHost.value = 'https://icp0.io';
  }
};

// Initialize ICP connection
const initializeICP = async () => {
  try {
    console.log('Initializing ICP connection...');
    initializeNetwork();

    backendActor = await createBackendActor(backendCanisterId.value, currentHost.value);
    console.log('ICP connection successful');
  } catch (error) {
    console.error('Failed to initialize ICP:', error);
  }
};

// Check if current user is the underwriter
const checkIfPoolManager = async () => {
  try {
    if (!userPrincipal.value || !backendActor) {
      isPoolManager.value = false;
      return;
    }

    const poolManagerPrincipal = await backendActor.get_pool_manager_principal();
    const poolManagerPrincipalString = poolManagerPrincipal.toText();

    isPoolManager.value = userPrincipal.value === poolManagerPrincipalString;

    console.log('Underwriter check:', {
      userPrincipal: userPrincipal.value,
      poolManagerPrincipal: poolManagerPrincipalString,
      isPoolManager: isPoolManager.value
    });
  } catch (error) {
    console.error('Error checking underwriter:', error);
    isPoolManager.value = false;
  }
};

// Load pool statistics
const loadPoolStatistics = async () => {
  if (!backendActor) return;

  try {
    // Get pool state
    const poolState = await backendActor.get_pool_state();
    totalAssetsStaked.value = poolState.total_assets;

    // Get pool reward rate for APR calculation
    const rewardRate = await backendActor.get_pool_reward_rate();
    const apr = Number(rewardRate * 365n * 24n * 3600n * 100n / 1_000_000_000_000_000_000n) / Number(poolState.total_assets);
    poolAPR.value = apr.toFixed(2);

    // Get total cover allocation
    const coverAllocation = await backendActor.get_total_cover_allocation();
    totalCoverAllocation.value = coverAllocation;

    console.log('Pool statistics loaded:', {
      totalAssetsStaked: totalAssetsStaked.value,
      poolAPR: poolAPR.value,
      totalCoverAllocation: totalCoverAllocation.value
    });
  } catch (error) {
    console.error('Error loading pool statistics:', error);
  }
};

// Load products
const loadProducts = async () => {
  if (!backendActor) return;

  try {
    isLoadingProducts.value = true;
    const productList = await backendActor.get_products();

    products.value = productList.map(product => ({
      product_id: product.product_id,
      name: product.name,
      annual_percent: product.annual_percent,
      max_coverage_duration: product.max_coverage_duration,
      max_pool_allocation_percent: product.max_pool_allocation_percent,
      allocation: product.allocation,
      last_allocation_update: product.last_allocation_update,
      active: product.active
    }));

    console.log('Products loaded:', products.value);
  } catch (error) {
    console.error('Error loading products:', error);
  } finally {
    isLoadingProducts.value = false;
  }
};

// Create product
const createProduct = async () => {
  if (!backendActor) return;

  try {
    createProductLoading.value = true;
    transactionType.value = 'create_product';
    transactionStatus.value = 'pending';
    transactionError.value = '';

    const annualPercentInBasisPoints = Math.floor(productForm.value.annualPercent * 100);
    const maxCoverageDurationInSeconds = BigInt(productForm.value.maxCoverageDuration * 24 * 60 * 60);
    const maxPoolAllocationPercentInBasisPoints = BigInt(Math.floor(productForm.value.maxPoolAllocationPercent * 100));

    // Use Plug wallet for signing the transaction
    const plugBackendActor = await createBackendActorWithPlug(backendCanisterId.value);
    try {
      const result = await plugBackendActor.create_product(
        productForm.value.name,
        BigInt(annualPercentInBasisPoints),
        maxCoverageDurationInSeconds,
        maxPoolAllocationPercentInBasisPoints
      );

      // if ('Err' in result) {
      //   throw new Error(Object.keys(result.Err)[0]);
      // }
    } catch (error) {
      console.error('Ignoring error:', error);
    }

    transactionStatus.value = 'success';
    closeCreateProductModal();
    await loadProducts();
  } catch (error) {
    console.error('Error creating product:', error);
    transactionError.value = 'Failed to create product: ' + error.message;
    transactionStatus.value = 'failed';
  } finally {
    createProductLoading.value = false;
  }
};

// Update product
const updateProduct = async () => {
  if (!backendActor) return;

  try {
    updateProductLoading.value = true;
    transactionType.value = 'update_product';
    transactionStatus.value = 'pending';
    transactionError.value = '';

    const annualPercentInBasisPoints = BigInt(Math.floor(editProductForm.value.annualPercent * 100));
    const maxCoverageDurationInSeconds = BigInt(editProductForm.value.maxCoverageDuration * 24 * 60 * 60);
    const maxPoolAllocationPercentInBasisPoints = BigInt(Math.floor(editProductForm.value.maxPoolAllocationPercent * 100));

    // Use Plug wallet for signing the transaction
    const plugBackendActor = await createBackendActorWithPlug(backendCanisterId.value);
    try {
      const result = await plugBackendActor.set_product(
        editingProductId.value,
        annualPercentInBasisPoints,
        maxCoverageDurationInSeconds,
        maxPoolAllocationPercentInBasisPoints,
        editProductForm.value.active
      );

      // if ('Err' in result) {
      //   throw new Error(Object.keys(result.Err)[0]);
      // }
    } catch (error) {
      console.error('Ignoring error:', error);
    }
    transactionStatus.value = 'success';
    closeEditProductModal();
    await loadProducts();
  } catch (error) {
    console.error('Error updating product:', error);
    transactionError.value = 'Failed to update product: ' + error.message;
    transactionStatus.value = 'failed';
  } finally {
    updateProductLoading.value = false;
  }
};

// Modal controls
const openCreateProductModal = () => {
  isCreateProductModalOpen.value = true;
  productForm.value = {
    name: '',
    annualPercent: null,
    maxCoverageDuration: null,
    maxPoolAllocationPercent: null
  };
};

const closeCreateProductModal = () => {
  isCreateProductModalOpen.value = false;
};

const openEditProductModal = (product) => {
  if (!isPoolManager.value) return;

  editingProductId.value = product.product_id;
  editProductForm.value = {
    annualPercent: Number(product.annual_percent) / 100,
    maxCoverageDuration: Number(product.max_coverage_duration) / (24 * 60 * 60),
    maxPoolAllocationPercent: Number(product.max_pool_allocation_percent) / 100,
    active: product.active
  };
  isEditProductModalOpen.value = true;
};

const closeEditProductModal = () => {
  isEditProductModalOpen.value = false;
  editingProductId.value = null;
};

// Load all data
const loadAllData = async () => {
  await checkIfPoolManager();

  if (isPoolManager.value) {
    await loadPoolStatistics();
    await loadProducts();
  }
};

// Watch for account changes
watch(() => web3Store.account, async (newAccount) => {
  console.log('Account changed to:', newAccount);
  if (backendActor) {
    await loadAllData();
  }
});

// Initialize on mount
onMounted(async () => {
  await initializeICP();
  await loadAllData();
});
</script>

<style scoped>
.form-input {
  @apply w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-yellow-500 focus:border-yellow-500;
}

.btn-primary {
  @apply bg-yellow-500 text-white hover:bg-yellow-600 transition-colors;
}

.btn-secondary {
  @apply bg-gray-100 text-gray-700 hover:bg-gray-200 border border-gray-300 transition-colors;
}
</style>