import { defineStore } from 'pinia';
import { getCurrentNetwork, getCanisterIds, ICP_CONFIG } from '../constants/icp';

export const useWeb3Store = defineStore('web3', {
    state: () => ({
        account: null,
        chainId: null,
        provider: null,
        isConnected: false,
    }),

    actions: {
        async connectWallet() {
            try {
                // Check if Plug is installed
                if (!window.ic || !window.ic.plug) {
                    alert('Plug wallet is not installed. Please install Plug from https://plugwallet.ooo/');
                    throw new Error('Plug wallet is not installed');
                }
                // Get network configuration
                const currentNetwork = getCurrentNetwork();
                const canisters = getCanisterIds(currentNetwork);
                const networkConfig = ICP_CONFIG[currentNetwork];

                // Check if already connected (avoids prompting user again)
                let isConnected = await window.ic.plug.isConnected();

                // Only request connection if not already connected
                if (!isConnected) {
                    isConnected = await window.ic.plug.requestConnect({
                        whitelist: Object.values(canisters),
                        host: networkConfig.host
                    });
                }

                if (isConnected) {
                    // Verify that required canisters are whitelisted
                    await window.ic.plug.createAgent({
                        whitelist: Object.values(canisters),
                        host: networkConfig.host,
                        onConnectionUpdate: async () => {
                            console.log('onConnectionUpdate');
                        }
                    });

                    // Get the principal ID
                    const principal = await window.ic.plug.agent.getPrincipal();

                    this.account = principal.toString();
                    this.chainId = currentNetwork;
                    this.plugAgent = window.ic.plug.agent;
                    this.isConnected = true;
                }
            } catch (error) {
                console.error('Error connecting wallet:', error);
                alert('Failed to connect to Plug wallet. Please try again.');
                throw error;
            }
        },
        disconnect() {
            // Disconnect from Plug wallet if connected
            if (window.ic?.plug) {
                window.ic.plug.disconnect().catch(error => {
                    console.error('Error disconnecting from Plug:', error);
                });
            }

            // Reset state
            this.account = null;
            this.chainId = null;
            this.provider = null;
            this.isConnected = false;
        }
    }
});
