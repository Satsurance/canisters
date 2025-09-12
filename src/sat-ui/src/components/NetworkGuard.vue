//src/components/NetworkGuard.vue
<template>
  <div class="relative">
    <!-- Overlay for unsupported network -->
    <div
        v-if="web3Store.isConnected && !isNetworkSupported"
        class="fixed inset-0 bg-gray-900 bg-opacity-50 backdrop-blur-sm z-50 flex items-center justify-center"
    >
      <div class="bg-white p-8 rounded-xl shadow-xl max-w-md w-full mx-4">
        <div class="flex items-center justify-center mb-4">
          <div class="bg-red-100 p-3 rounded-full">
            <svg
                xmlns="http://www.w3.org/2000/svg"
                fill="none"
                viewBox="0 0 24 24"
                stroke-width="1.5"
                stroke="currentColor"
                class="size-6 text-red-500"
            >
              <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126ZM12 15.75h.007v.008H12v-.008Z"
              />
            </svg>
          </div>
        </div>

        <h3 class="text-xl font-bold text-center mb-2">Unsupported Network</h3>
        <p class="text-gray-600 text-center mb-6">
          Please switch to a supported network to use Satsurance:
        </p>

        <div class="space-y-3">
          <button
              v-for="network in supportedNetworks"
              :key="network.chainId"
              @click="() => switchNetwork(network)"
              class="w-full py-3 px-4 rounded-lg border border-gray-200 hover:border-yellow-500 hover:text-yellow-500 focus:outline-none focus:border-yellow-500 focus:text-yellow-500 flex items-center justify-between"
          >
            <span>{{ network.name }}</span>
            <svg
                xmlns="http://www.w3.org/2000/svg"
                fill="none"
                viewBox="0 0 24 24"
                stroke-width="1.5"
                stroke="currentColor"
                class="size-5"
            >
              <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  d="M8.25 4.5l7.5 7.5-7.5 7.5"
              />
            </svg>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue';
import { useWeb3Store } from '../stores/web3Store';
import { NETWORKS, SUPPORTED_NETWORKS } from '../constants/contracts';

const web3Store = useWeb3Store();
const supportedNetworks = Object.values(SUPPORTED_NETWORKS);

const isNetworkSupported = computed(() => {
  return Object.values(NETWORKS).includes(web3Store.chainId);
});

const switchNetwork = async (network) => {
  try {
    console.log({ chainId: `0x${network.chainId.toString(16)}` });
    // Try switching to existing network
    await window.ethereum.request({
      method: 'wallet_switchEthereumChain',
      params: [{ chainId: `0x${network.chainId.toString(16)}` }],
    });
  } catch (error) {
    // If network doesn't exist, add it
    if (error.code === 4902 || error.code === -32603) {
      try {
        console.log({
          method: 'wallet_addEthereumChain',
              params: [{
            chainId: `0x${network.chainId.toString(16)}`,
            chainName: network.name,
            nativeCurrency: network.nativeCurrency,
            rpcUrls: network.rpcUrls,
            blockExplorerUrls: network.blockExplorerUrls,
          }],
        });
        await window.ethereum.request({
          method: 'wallet_addEthereumChain',
          params: [{
            chainId: `0x${network.chainId.toString(16)}`,
            chainName: network.name,
            nativeCurrency: network.nativeCurrency,
            rpcUrls: network.rpcUrls,
            blockExplorerUrls: network.blockExplorerUrls,
          }],
        });
      } catch (addError) {
        console.error('Error adding network:', addError);
      }
    } else {
      console.error('Error switching network:', error);
    }
  }
};
</script>