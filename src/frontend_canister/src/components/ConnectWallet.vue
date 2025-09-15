<template>
  <div class="flex items-center">
    <button
        v-if="!web3Store.isConnected"
        @click="handleConnect"
        class="btn-primary flex flex-row items-center px-4 py-2 rounded-lg"
    >
      <span class="mr-2">Connect Wallet</span>
      <svg
          xmlns="http://www.w3.org/2000/svg"
          fill="none"
          viewBox="0 0 24 24"
          stroke-width="1.5"
          stroke="currentColor"
          class="size-6"
      >
        <path
            stroke-linecap="round"
            stroke-linejoin="round"
            d="M21 12a2.25 2.25 0 0 0-2.25-2.25H15a3 3 0 1 1-6 0H5.25A2.25 2.25 0 0 0 3 12m18 0v6a2.25 2.25 0 0 1-2.25 2.25H5.25A2.25 2.25 0 0 1 3 18v-6m18 0V9M3 12V9m18 0a2.25 2.25 0 0 0-2.25-2.25H5.25A2.25 2.25 0 0 0 3 9m18 0V6a2.25 2.25 0 0 0-2.25-2.25H5.25A2.25 2.25 0 0 0 3 6v3"
        />
      </svg>
    </button>
    <div v-else>
      <!-- Desktop View -->
      <div class="hidden lg:flex items-center space-x-4">
        <div class="flex items-center space-x-2 border border-gray-200 rounded-lg px-3 py-2">
          <svg
              xmlns="http://www.w3.org/2000/svg"
              fill="none"
              viewBox="0 0 24 24"
              stroke-width="1.5"
              stroke="currentColor"
              class="size-5 text-yellow-500"
          >
            <path
                stroke-linecap="round"
                stroke-linejoin="round"
                d="M21 12a2.25 2.25 0 0 0-2.25-2.25H15a3 3 0 1 1-6 0H5.25A2.25 2.25 0 0 0 3 12m18 0v6a2.25 2.25 0 0 1-2.25 2.25H5.25A2.25 2.25 0 0 1 3 18v-6m18 0V9M3 12V9m18 0a2.25 2.25 0 0 0-2.25-2.25H5.25A2.25 2.25 0 0 0 3 9m18 0V6a2.25 2.25 0 0 0-2.25-2.25H5.25A2.25 2.25 0 0 0 3 6v3"
            />
          </svg>
          <div class="flex flex-col">
            <span class="text-sm text-gray-600">{{ formatAddress(web3Store.account) }}</span>
            <span class="text-xs text-gray-500">{{ networkName }}</span>
          </div>
        </div>
        <button
            @click="handleDisconnect"
            class="text-sm text-red-400 hover:text-red-500 px-3 py-1 rounded border border-red-400 hover:border-red-500"
        >
          Disconnect
        </button>
      </div>

      <!-- Mobile View -->
      <div class="flex flex-col lg:hidden">
        <p class="font-bold mb-2">Wallet</p>
        <p>{{ formatAddress(web3Store.account) }}</p>
        <p>{{ networkName }}</p>
        <button
            @click="handleDisconnect"
            class="border-red-400 bg-white text-red-400 mt-2 focus:outline-none hover:border-red-400"
        >
          Disconnect
        </button>
      </div>
    </div>
  </div>
  <NetworkGuard>
  </NetworkGuard>
</template>

<script setup>
import { computed } from "vue";
import { useWeb3 } from "../composables/useWeb3";
import { useWeb3Store } from "../stores/web3Store";
import NetworkGuard from "./NetworkGuard.vue";

const web3 = useWeb3();
const web3Store = useWeb3Store();

const networkName = computed(() => {
  if (!web3Store.isConnected) return "Not connected";
  switch (web3Store.chainId) {
    case 200810:
      return "Bitlayer Testnet";
    case 763373:
      return "Ink Sepolia";
    case 808813:
      return "BOB Sepolia";
    case 31337:
      return "Local Hardhat";
    default:
      return "Unsupported Network";
  }
});

const handleConnect = async () => {
  try {
    await web3.connectWallet();
  } catch (error) {
    console.error("Failed to connect wallet:", error);
  }
};

const handleDisconnect = () => {
  web3Store.disconnect();
};

const formatAddress = (address) => {
  if (!address) return "";
  return `${address.slice(0, 6)}...${address.slice(-4)}`;
};
</script>