# MyCoversLayout.vue
<template>
  <div class="min-h-[85vh] bg-gray-50">
    <div class="max-w-6xl mx-auto px-4 py-8">
      <!-- Header Section -->
      <div class="bg-white rounded-lg shadow-sm p-6 mb-8">
        <div class="flex flex-col items-center justify-between mb-6">
          <h1 class="text-2xl md:text-4xl font-semibold text-gray-900 flex items-center gap-3">
            <svg
                class="w-8 h-8 text-yellow-500"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
            >
              <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z"
              />
            </svg>
            My Purchases
          </h1>
        </div>

        <!-- Filter Tabs -->
        <div class="flex justify-center space-x-4 mb-8">
          <button
              v-for="status in filterStatuses"
              :key="status"
              @click="selectedStatus = status"
              :class="[
              'px-6 py-2 rounded-lg transition-colors',
              selectedStatus === status
                ? 'btn-primary bg-yellow-500 border border-yellow-500 text-white hover:bg-white hover:text-yellow-500'
                : 'btn-secondary bg-gray-100 text-gray-600 border border-gray-200 hover:bg-gray-200'
            ]"
          >
            {{ status }}
          </button>
        </div>

        <!-- Wallet Not Connected State -->
        <div v-if="!web3Store.isConnected" class="text-center py-12">
          <p class="text-gray-500">Please connect your wallet to view your covers.</p>
        </div>

        <!-- Loading State -->
        <div v-else-if="loading" class="flex justify-center items-center py-12">
          <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-yellow-500"></div>
        </div>

        <!-- Error State -->
        <div v-else-if="error" class="text-center py-12">
          <p class="text-red-500">{{ error }}</p>
          <button
              @click="loadUserCovers"
              class="mt-4 px-4 py-2 bg-yellow-500 text-white rounded-lg hover:bg-yellow-600"
          >
            Retry
          </button>
        </div>

        <!-- Empty States -->
        <div v-else-if="!userCovers.length || !filteredCovers.length" class="text-center py-12">
          <p class="text-gray-500">
            {{ emptyStateMessage }}
          </p>
          <div class="mt-4 space-x-4">
            <router-link
                v-if="!userCovers.length"
                to="/cover_buy"
                class="inline-block px-4 py-2 rounded-lg btn-primary bg-yellow-500 text-white hover:bg-yellow-600"
            >
              Buy Cover
            </router-link>
            <button
                v-if="userCovers.length && !filteredCovers.length"
                @click="selectedStatus = 'All'"
                class="inline-block px-4 py-2 rounded-lg btn-secondary bg-gray-100 text-gray-600 border border-gray-200 hover:bg-gray-200"
            >
              Clear Filter
            </button>
          </div>
        </div>

        <!-- Covers Grid -->
        <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
          <UserCoverCard
              v-for="cover in filteredCovers"
              :key="`${cover.protocol}-${cover.startDate}`"
              :cover="cover"
              :project-info="projectsInfo[cover.protocol]"
              @click="openCoverDetails(cover)"
              class="cursor-pointer"
          />
        </div>
      </div>
    </div>

    <!-- Cover Details Modal -->
    <UserCoverDetails
        v-if="selectedCover"
        :cover="selectedCover"
        :project-info="projectsInfo[selectedCover.protocol]"
        :is-open="!!selectedCover"
        @close="closeCoverDetails"
    />
  </div>
</template>

<script setup>
import { ref, computed, watch, onMounted } from 'vue';
import { useWeb3Store } from '../stores/web3Store';
import { getCurrentNetwork, getCanisterIds, ICP_CONFIG } from '../constants/icp.js';
import { createBackendActorWithPlug, createLedgerActorWithPlug } from '../utils/icpAgent.js';
import UserCoverCard from '../components/UserCoverCard.vue';
import UserCoverDetails from '../components/UserCoverDetails.vue';
import { COVER_PROJECTS } from '../constants/projects';

// Store and State
const web3Store = useWeb3Store();
const filterStatuses = ['All', 'Active', 'Expired'];
const selectedStatus = ref('Active');
const userCovers = ref([]);
const loading = ref(true);
const error = ref(null);
const selectedCover = ref(null);
const projectsInfo = COVER_PROJECTS;
const currentNetwork = ref('');

// Computed Properties
const emptyStateMessage = computed(() => {
  if (!userCovers.value.length) {
    return 'No covers found';
  }
  if (!filteredCovers.value.length) {
    return `No ${selectedStatus.value.toLowerCase()} covers found`;
  }
  return '';
});

const initializeNetwork = () => {
  currentNetwork.value = getCurrentNetwork();
};

const formatNatToDecimal = (value, decimals) => {
  const amount = BigInt(value ?? 0);
  const scale = 10n ** BigInt(decimals);
  const integer = amount / scale;
  const fraction = amount % scale;
  const fractionStr = fraction.toString().padStart(decimals, '0').replace(/0+$/, '');
  return fractionStr ? `${integer.toString()}.${fractionStr}` : integer.toString();
};

// Load user covers from pool canister
const loadUserCovers = async () => {
  try {
    loading.value = true;
    error.value = null;

    if (!web3Store.isConnected || !window.ic?.plug) {
      userCovers.value = [];
      loading.value = false;
      return;
    }

    const network = currentNetwork.value || getCurrentNetwork();
    const { backend, ledger } = getCanisterIds(network);

    await window.ic.plug.createAgent({
      whitelist: [backend, ledger],
      host: ICP_CONFIG[network]?.host,
    });

    if (network === 'local') {
      try {
        await window.ic.plug.agent.fetchRootKey();
      } catch (agentError) {
        console.warn('Failed to fetch root key:', agentError);
      }
    }

    const backendActor = await createBackendActorWithPlug(backend);
    const ledgerActor = await createLedgerActorWithPlug(ledger);
    const principal = await window.ic.plug.agent.getPrincipal();

    const [productList, decimals, covers] = await Promise.all([
      backendActor.get_products(),
      ledgerActor.icrc1_decimals(),
      backendActor.get_coverages(principal),
    ]);

    const decimalsNumber = Number(decimals);
    const productMap = new Map();
    productList.forEach(product => {
      productMap.set(Number(product.product_id), product.name);
    });

    userCovers.value = covers.map(cover => {
      const productId = Number(cover.product_id);
      const productName = productMap.get(productId) || `Product ${productId}`;

      return {
        id: Number(cover.coverage_id),
        user: principal.toText(),
        protocol: productName,
        productId,
        startDate: Number(cover.start_time) * 1000,
        endDate: Number(cover.end_time) * 1000,
        coverAmount: formatNatToDecimal(cover.coverage_amount, decimalsNumber),
        premiumAmount: formatNatToDecimal(cover.premium_amount, decimalsNumber),
      };
    });
  } catch (e) {
    console.error('Error loading covers:', e);
    error.value = 'Failed to load covers. Please try again.';
  } finally {
    loading.value = false;
  }
};

// Computed properties
const filteredCovers = computed(() => {
  const now = Date.now();

  return userCovers.value.filter(cover => {
    if (selectedStatus.value === 'Active') {
      return cover.endDate > now;
    }
    if (selectedStatus.value === 'Expired') {
      return cover.endDate <= now;
    }
    return true;
  });
});

const openCoverDetails = (cover) => {
  selectedCover.value = cover;
};

const closeCoverDetails = () => {
  selectedCover.value = null;
};

onMounted(async () => {
  initializeNetwork();
  if (web3Store.isConnected) {
    await loadUserCovers();
  } else {
    loading.value = false;
  }
});

watch(
  () => [web3Store.isConnected, web3Store.account],
  async ([isConnected]) => {
    if (isConnected) {
      await loadUserCovers();
    } else {
      userCovers.value = [];
      loading.value = false;
    }
  }
);
</script>