import { defineStore } from 'pinia';

export const useWeb3Store = defineStore('web3', {
    state: () => ({
        account: null,
        chainId: null,
        isConnected: false,
        plugAgent: null,
    }),

    actions: {
        // Connect to Plug wallet
        connect(connectionData) {
            this.account = connectionData.account;
            this.chainId = connectionData.chainId;
            this.isConnected = connectionData.isConnected;
            this.plugAgent = window.ic?.plug?.agent || null;
            
            // Persist connection state in localStorage
            localStorage.setItem('plugWalletConnected', 'true');
            localStorage.setItem('plugWalletAccount', connectionData.account);
            localStorage.setItem('plugWalletChainId', connectionData.chainId);
        },

        disconnect() {
            this.account = null;
            this.chainId = null;
            this.isConnected = false;
            this.plugAgent = null;
            
            // Clear persisted connection state
            localStorage.removeItem('plugWalletConnected');
            localStorage.removeItem('plugWalletAccount');
            localStorage.removeItem('plugWalletChainId');
        },

        // Restore connection state from localStorage
        restoreConnection() {
            const wasConnected = localStorage.getItem('plugWalletConnected');
            const account = localStorage.getItem('plugWalletAccount');
            const chainId = localStorage.getItem('plugWalletChainId');
            
            console.log('Restore attempt:', { wasConnected, account, chainId });
            
            if (wasConnected && account && chainId) {
                this.account = account;
                this.chainId = chainId;
                this.isConnected = true;
                this.plugAgent = window.ic?.plug?.agent || null;
                console.log('Connection restored from localStorage');
                return true;
            }
            console.log('No stored connection found');
            return false;
        }
    }
});