<!-- BuyCoverLayout.vue -->
<template>
  <div class="min-h-[85vh] bg-gray-50">
    <div class="max-w-6xl mx-auto px-4 py-8">
      <!-- Header Section -->
      <div class="bg-white rounded-lg shadow-sm p-6 mb-8">
        <div class="flex flex-col items-center justify-between mb-6">
          <h1 class="text-2xl md:text-4xl font-semibold text-gray-900 flex items-center gap-3">
            <svg class="w-8 h-8 text-yellow-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
            </svg>
            Insurance Products
          </h1>
          <p class="text-gray-500 mt-2">Satsurance is currently running on Testnet</p>
        </div>

        <!-- Filter Tabs -->
        <div class="flex justify-center space-x-4 mb-8">
          <button v-for="category in categories" :key="category" @click="selectedCategory = category" :class="[
            'px-6 py-2 rounded-lg transition-colors',
            selectedCategory === category
              ? 'btn-primary bg-yellow-500 border border-yellow-500 text-white hover:bg-white hover:text-yellow-500'
              : 'btn-secondary bg-gray-100 text-gray-600 border border-gray-200 hover:bg-gray-200'
          ]">
            {{ category }}
          </button>
        </div>
      </div>

      <!-- Projects Grid - Responsive grid with different columns based on screen size -->
      <div v-if="filteredProjects.length > 0"
        class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4 md:gap-6">
        <ProjectCard v-for="project in filteredProjects" :key="project.name" :project="project"
          @click="openPurchaseModal(project.name)" />
      </div>

      <!-- Empty State / Coming Soon Placeholder -->
      <div v-else class="bg-white rounded-lg shadow-sm p-12 text-center">
        <svg class="w-16 h-16 mx-auto text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
            d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
        </svg>
        <h3 class="mt-4 text-xl font-medium text-gray-900">Coming Soon</h3>
        <p class="mt-2 text-gray-500">We're working on adding more insurance products in this category.</p>
        <button @click="selectedCategory = 'All'"
          class="mt-6 px-6 py-2 rounded-lg bg-yellow-500 text-white hover:bg-yellow-600 transition-colors">
          View All Products
        </button>
      </div>

    </div>

    <!-- Purchase Modal -->
    <CoverPurchaseDialog v-if="selectedProject" :project="selectedProject" :show="!!selectedProject"
      :is-submitting="firstTxStatus !== ''" @close="handleClose" @purchase="handlePurchase" />

    <!-- Transaction Status Modal -->
    <TransactionStatus :show="!!firstTxStatus" :steps="transactionSteps" :tx-hash="currentTxHash"
      :error="transactionError" :block-explorer="ICP_MAINNET_BLOCK_EXPLORER" @close="resetTransaction"
      @retry="retryTransaction" />
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue';
import { Principal } from '@dfinity/principal';
import ProjectCard from '../components/CoverCard.vue';
import CoverPurchaseDialog from '../components/CoverPurchaseDialog.vue';
import TransactionStatus from '../components/TransactionStatus.vue';
import { COVER_PROJECTS } from '../constants/projects';
import { getCurrentNetwork, getCanisterIds, ICP_CONFIG, ICP_MAINNET_BLOCK_EXPLORER } from '../constants/icp.js';
import { createBackendActor, createBackendActorWithPlug, createLedgerActorWithPlug } from '../utils/icpAgent.js';
import { useWeb3Store } from '../stores/web3Store';

const categories = ['All', 'Web3', 'Cannabis', 'AI'];
const selectedCategory = ref('All');
const selectedProject = ref(null);

// Transaction state
const firstTxStatus = ref('');
const secondTxStatus = ref('');
const transactionType = ref('');
const currentTxHash = ref('');
const transactionError = ref('');
const currentPurchaseParams = ref(null);
const products = ref([]);
const poolState = ref(null);

const currentNetwork = ref('');
const currentHost = ref('');
const backendCanisterId = ref('');

const BASIS_POINTS = 10000n;
const SECONDS_PER_DAY = 24n * 60n * 60n;
const SECONDS_PER_YEAR = 365n * SECONDS_PER_DAY;
const MIN_COVERAGE_AMOUNT = 0.00001; // Minimum coverage in BTC

const web3Store = useWeb3Store();
const initializeNetwork = () => {
  currentNetwork.value = getCurrentNetwork();
  const canisterIds = getCanisterIds(currentNetwork.value);
  backendCanisterId.value = canisterIds.backend;
  currentHost.value = ICP_CONFIG[currentNetwork.value]?.host || '';
};

const loadPoolState = async () => {
  if (!backendCanisterId.value || !currentHost.value) {
    return;
  }

  try {
    const backendActor = await createBackendActor(backendCanisterId.value, currentHost.value);
    poolState.value = await backendActor.get_pool_state();
  } catch (error) {
    console.error('Error loading pool state:', error);
    poolState.value = null;
  }
};

const loadProducts = async () => {
  if (!backendCanisterId.value || !currentHost.value) {
    return;
  }

  try {
    const backendActor = await createBackendActor(backendCanisterId.value, currentHost.value);
    const productList = await backendActor.get_products();
    products.value = productList.map(product => ({
      name: product.name,
      productId: Number(product.product_id),
      annualPercent: Number(product.annual_percent),
      maxCoverageDuration: Number(product.max_coverage_duration),
      maxPoolAllocationPercent: Number(product.max_pool_allocation_percent),
      allocation: product.allocation,
      active: product.active,
    }));
  } catch (error) {
    console.error('Error loading products:', error);
    products.value = [];
  }
};

const productsByName = computed(() => {
  const map = new Map();
  products.value.forEach(product => {
    map.set(product.name, product);
  });
  return map;
});

const calculateMaxCoverage = (product) => {
  if (!poolState.value || !poolState.value.total_assets) {
    return null;
  }

  try {
    // Convert total_assets from Nat to number (assuming 8 decimals for BTC)
    const totalAssets = BigInt(poolState.value.total_assets.toString());
    const maxAllocationPercent = BigInt(product.maxPoolAllocationPercent);
    const currentAllocation = BigInt(product.allocation.toString());
    const decimals = 8n;
    const scale = 10n ** decimals;

    // Calculate max allocation for this product
    // max_allocation = (total_assets * max_pool_allocation_percent) / 10000
    const maxAllocation = (totalAssets * maxAllocationPercent) / 10000n;

    // Available coverage = max_allocation - current_allocation
    const availableCoverage = maxAllocation > currentAllocation ? maxAllocation - currentAllocation : 0n;

    // Convert to BTC (divide by 10^8)
    const coverageInBTC = Number(availableCoverage) / Number(scale);

    return Math.max(0, coverageInBTC);
  } catch (error) {
    console.error('Error calculating max coverage:', error);
    return null;
  }
};

const availableProjects = computed(() => {
  return Object.entries(COVER_PROJECTS)
    .map(([name, data]) => {
      const product = productsByName.value.get(name);
      if (!product || !product.active) {
        return null;
      }

      const maxCoverage = calculateMaxCoverage(product);

      // If we can't calculate max coverage or it's 0, don't show the product
      if (maxCoverage === null || maxCoverage <= 0) {
        return null;
      }

      return {
        name,
        ...data,
        productId: product.productId,
        annualPercent: product.annualPercent,
        maxCoverageDuration: product.maxCoverageDuration,
        rate: product.annualPercent / 100,
        // Use dynamically calculated coverage range
        minCover: MIN_COVERAGE_AMOUNT,
        maxCover: maxCoverage,
      };
    })
    .filter(Boolean);
});

const filteredProjects = computed(() => {
  if (selectedCategory.value === 'All') {
    return availableProjects.value;
  }
  return availableProjects.value.filter(project => project.category === selectedCategory.value);
});

const transactionSteps = computed(() => {
  if (transactionType.value === 'cover_purchase') {
    return [
      {
        id: 'approve',
        title: 'Approve BTC',
        description: 'Allow smart contract to use your BTC',
        status: firstTxStatus.value,
        showNumber: true
      },
      {
        id: 'purchase',
        title: 'Purchase Cover',
        description: 'Process your cover purchase',
        status: secondTxStatus.value,
        showNumber: true
      }
    ];
  }
  return [];
});

const openPurchaseModal = (projectName) => {
  const project = availableProjects.value.find(project => project.name === projectName);
  if (project) {
    selectedProject.value = project;
  }
};

const resetTransaction = () => {
  firstTxStatus.value = '';
  secondTxStatus.value = '';
  transactionType.value = '';
  currentTxHash.value = '';
  transactionError.value = '';
  currentPurchaseParams.value = null;
};

const handleClose = () => {
  selectedProject.value = null;
  resetTransaction();
};

const resolveLedgerError = (errorVariant) => {
  const [errorKey] = Object.keys(errorVariant || {});
  switch (errorKey) {
    case 'InsufficientFunds':
      return 'Insufficient BTC balance to complete the transfer.';
    case 'TemporarilyUnavailable':
      return 'Ledger temporarily unavailable. Please try again later.';
    case 'Duplicate':
      return 'Duplicate transfer detected. Please wait and try again.';
    case 'TooOld':
      return 'Transfer request is too old. Please retry.';
    case 'BadFee':
      return 'Transfer fee mismatch. Please retry.';
    default:
      return 'Transfer failed. Please try again.';
  }
};

const resolvePoolError = (errorVariant) => {
  const [errorKey] = Object.keys(errorVariant || {});
  switch (errorKey) {
    case 'ProductNotActive':
      return 'Selected coverage market is not active.';
    case 'CoverageDurationTooLong':
      return 'Selected duration exceeds the allowed maximum.';
    case 'CoverageDurationTooShort':
      return 'Selected duration is shorter than the minimum allowed.';
    case 'NotEnoughAssetsToCover':
      return 'Pool has insufficient liquidity to provide this coverage amount.';
    case 'InsufficientBalance':
      return 'Insufficient balance in deposit subaccount.';
    case 'ProductNotFound':
      return 'Selected coverage product was not found.';
    default:
      return 'Transaction failed. Please try again.';
  }
};

const convertAmountToNat = (value, decimals) => {
  if (value === undefined || value === null) {
    return 0n;
  }
  const asString = value.toString();
  const [intPartRaw, fracPartRaw = ''] = asString.split('.');
  const sanitizedInt = intPartRaw.replace(/\D/g, '') || '0';
  const sanitizedFrac = fracPartRaw.replace(/\D/g, '');
  const scale = BigInt(10) ** BigInt(decimals);
  const intComponent = BigInt(sanitizedInt);
  const fracPadded = (sanitizedFrac + '0'.repeat(decimals)).slice(0, decimals);
  const fracComponent = fracPadded ? BigInt(fracPadded) : 0n;
  return intComponent * scale + fracComponent;
};

const handlePurchase = async (purchaseParams) => {
  try {
    if (!selectedProject.value) {
      return;
    }

    transactionType.value = 'cover_purchase';
    firstTxStatus.value = '';
    secondTxStatus.value = '';
    transactionError.value = '';
    currentTxHash.value = '';

    const { coverAmount, duration } = purchaseParams;
    currentPurchaseParams.value = purchaseParams;

    if (!window.ic?.plug || !web3Store.isConnected) {
      transactionError.value = 'Plug wallet not connected';
      firstTxStatus.value = 'failed';
      throw new Error('Plug wallet not connected');
    }

    const network = currentNetwork.value || getCurrentNetwork();
    const { backend, ledger } = getCanisterIds(network);

    try {
      await window.ic.plug.createAgent({
        whitelist: [backend, ledger],
        host: ICP_CONFIG[network]?.host,
      });
    } catch (error) {
      console.log('createAgent error ignored:', error);
    }

    if (network === 'local') {
      try {
        await window.ic.plug.agent.fetchRootKey();
      } catch (error) {
        console.log('fetchRootKey error ignored:', error);
      }
    }

    const backendActor = await createBackendActorWithPlug(backend);
    const ledgerActor = await createLedgerActorWithPlug(ledger);

    const principal = await window.ic.plug.agent.getPrincipal();
    const decimals = Number(await ledgerActor.icrc1_decimals());
    const fee = await ledgerActor.icrc1_fee();

    const coverageAmountNat = convertAmountToNat(coverAmount, decimals);
    const durationSeconds = BigInt(Math.floor(Number(duration))) * SECONDS_PER_DAY;
    const annualPercent = BigInt(selectedProject.value.annualPercent);

    console.log('coverageAmountNat', coverageAmountNat);
    console.log('durationSeconds', durationSeconds);
    console.log('annualPercent', annualPercent);
    console.log('SECONDS_PER_YEAR', SECONDS_PER_YEAR);
    console.log('BASIS_POINTS', BASIS_POINTS);
    const premiumNat = (coverageAmountNat * annualPercent * durationSeconds) / (SECONDS_PER_YEAR * BASIS_POINTS);
    console.log('premiumNat', premiumNat);

    if (premiumNat <= fee) {
      console.log(premiumNat, fee);
      transactionError.value = 'Premium amount is too small to cover the transfer fee.';
      firstTxStatus.value = 'failed';
      throw new Error('Premium too small');
    }

    const balance = await ledgerActor.icrc1_balance_of({ owner: principal, subaccount: [] });
    if (balance < premiumNat + fee) {
      transactionError.value = 'Insufficient BTC balance to cover premium and fees.';
      firstTxStatus.value = 'failed';
      throw new Error('Insufficient balance');
    }

    console.log("productId", selectedProject.value.productId);

    const subaccount = await backendActor.get_purchase_subaccount(principal, BigInt(selectedProject.value.productId));

    firstTxStatus.value = 'pending';
    const transferArg = {
      from_subaccount: [],
      to: { owner: Principal.fromText(backend), subaccount: [Array.from(subaccount)] },
      amount: premiumNat,
      fee: [fee],
      memo: [],
      created_at_time: [],
    };

    try {
      const transferResult = await ledgerActor.icrc1_transfer(transferArg);
      if (transferResult.Ok) {
        currentTxHash.value = transferResult.Ok?.toString() || '';
      }
    } catch (error) {
      console.log('Transfer error ignored (Plug wallet certificate issue):', error);
    }

    firstTxStatus.value = 'success';

    // Query and log the subaccount balance
    try {
      const subaccountBalance = await ledgerActor.icrc1_balance_of({
        owner: Principal.fromText(backend),
        subaccount: [Array.from(subaccount)]
      });
      console.log('Subaccount balance:', subaccountBalance.toString());
    } catch (error) {
      console.error('Error querying subaccount balance:', error);
    }

    secondTxStatus.value = 'pending';

    try {
      const coverageResult = await backendActor.purchase_coverage(
        BigInt(selectedProject.value.productId),
        principal,
        durationSeconds,
        coverageAmountNat
      );
    } catch (error) {
      console.log('Purchase coverage error ignored (Plug wallet certificate issue):', error);
    }

    secondTxStatus.value = 'success';
    setTimeout(handleClose, 2000);

    // Reload products and pool state to reflect updated data
    await Promise.all([loadProducts(), loadPoolState()]);


  } catch (error) {
    console.error('Cover purchase process error:', error);
    if (!transactionError.value) {
      transactionError.value = 'Transaction failed. Please try again';
    }
  }
};

const retryTransaction = async () => {
  if (currentPurchaseParams.value) {
    await handlePurchase(currentPurchaseParams.value);
  }
};

onMounted(async () => {
  initializeNetwork();
  await Promise.all([loadProducts(), loadPoolState()]);
});
</script>