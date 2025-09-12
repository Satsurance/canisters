import { onMounted, onUnmounted } from 'vue';
import { useWeb3Store } from '../stores/web3Store';

export function useWeb3() {
    const web3Store = useWeb3Store();

    const checkMetaMaskInstalled = () => {
        return typeof window.ethereum !== 'undefined';
    };

    const connectWallet = async () => {
        if (!checkMetaMaskInstalled()) {
            throw new Error('MetaMask is not installed');
        }
        return web3Store.connectWallet();
    };

    // Auto-connect if previously connected
    onMounted(async () => {
        if (checkMetaMaskInstalled() && window.ethereum.selectedAddress) {
            try {
                await connectWallet();
            } catch (error) {
                console.error('Error auto-connecting:', error);
            }
        }
    });

    return {
        connectWallet,
        checkMetaMaskInstalled,
        isConnected: () => web3Store.isConnected,
        account: () => web3Store.account,
        chainId: () => web3Store.chainId
    };
}